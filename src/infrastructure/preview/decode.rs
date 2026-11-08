use std::collections::{HashSet, VecDeque};
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

use futures_channel::mpsc::UnboundedSender;

use crate::contracts::preview::{BufPool, PREVIEW_HEIGHT, PREVIEW_WIDTH, Upload, UploadQueue};
use crate::infrastructure::runtime::Wake;

const WAKE_COALESCE: Duration = Duration::from_millis(40);
const MAX_SOURCE_EDGE: u32 = 4096;
const MAX_SOURCE_PIXELS: u64 = 8 * 1024 * 1024;
const MAX_ENCODED_BYTES: u64 = 32 * 1024 * 1024;
const MAX_DECODE_BYTES: usize = 32 * 1024 * 1024;
const WEBP_EXTENSION: &str = "webp";
const THUMB_EXTENSIONS: [&str; 3] = [WEBP_EXTENSION, "jpg", "jpeg"];
const NEAR_CONTAINER_BYTES: usize = near_container_bytes(PREVIEW_WIDTH, PREVIEW_HEIGHT);

const fn near_container_bytes(mut width: u32, mut height: u32) -> usize {
    let mut bytes = 12usize;
    loop {
        let aligned_width = (width + 3) & !3;
        let aligned_height = (height + 3) & !3;
        bytes += 8 + aligned_width as usize * aligned_height as usize;
        let next_width = width / 2;
        let next_height = height / 2;
        if next_width < 16 || next_height < 16 {
            return bytes;
        }
        width = next_width;
        height = next_height;
    }
}

#[derive(Debug)]
pub struct Job {
    pub store_idx: usize,
    pub path: String,
    pub fallback: Option<String>,
    pub tier: u32,
    pub layer: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub compressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeDone {
    pub store_idx: usize,
    pub tier: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeFailed {
    pub store_idx: usize,
    pub tier: u32,
    pub error: String,
}

pub struct DrainedDecodes {
    pub done: Vec<DecodeDone>,
    pub failed: Vec<DecodeFailed>,
}

struct Decoded {
    data: Vec<u8>,
    blocks: bool,
    recycle: Option<Arc<BufPool>>,
}

impl Drop for Decoded {
    fn drop(&mut self) {
        if let Some(pool) = self.recycle.take() {
            pool.put(std::mem::take(&mut self.data));
        }
    }
}

struct Queues {
    visible: VecDeque<Job>,
    backfill: VecDeque<Job>,
    closed: bool,
}

struct PoolState {
    queues: Mutex<Queues>,
    cv: Condvar,
    uploads: UploadQueue,
    done: Mutex<Vec<DecodeDone>>,
    failed: Mutex<Vec<DecodeFailed>>,
    cancelled: Mutex<Vec<usize>>,
    wanted: Mutex<HashSet<usize>>,
    stats: Mutex<(f64, f32, u64)>,
    last_wake: Mutex<Instant>,
    dirty: AtomicBool,
    in_flight: AtomicUsize,
    tx: UnboundedSender<Wake>,
    read_pool: Arc<BufPool>,
}

#[derive(Clone)]
pub struct DecodePool {
    state: Arc<PoolState>,
    _shutdown: Arc<PoolShutdown>,
}

struct PoolShutdown(Arc<PoolState>);

impl Drop for PoolShutdown {
    fn drop(&mut self) {
        {
            let mut queues = self.0.queues.lock().unwrap_or_else(PoisonError::into_inner);
            queues.closed = true;
            queues.visible.clear();
            queues.backfill.clear();
        }
        self.0.cv.notify_all();
        self.0.read_pool.close();
    }
}

impl DecodePool {
    pub fn start(uploads: UploadQueue, tx: UnboundedSender<Wake>, workers: usize) -> Self {
        let now = Instant::now();
        let state = Arc::new(PoolState {
            queues: Mutex::new(Queues {
                visible: VecDeque::new(),
                backfill: VecDeque::new(),
                closed: false,
            }),
            cv: Condvar::new(),
            uploads,
            done: Mutex::new(Vec::new()),
            failed: Mutex::new(Vec::new()),
            cancelled: Mutex::new(Vec::new()),
            wanted: Mutex::new(HashSet::new()),
            stats: Mutex::new((0.0, 0.0, 0)),
            last_wake: Mutex::new(now.checked_sub(WAKE_COALESCE + WAKE_COALESCE).unwrap_or(now)),
            dirty: AtomicBool::new(false),
            in_flight: AtomicUsize::new(0),
            tx,
            read_pool: Arc::new(BufPool::new_bounded(NEAR_CONTAINER_BYTES, workers)),
        });
        // Workers own state, but only clients keep the pool open. Install the
        // shutdown guard before spawning so partial startup also releases workers.
        let pool = Self { _shutdown: Arc::new(PoolShutdown(state.clone())), state: state.clone() };
        for idx in 0..workers {
            let worker_state = state.clone();
            std::thread::Builder::new()
                .name(format!("decode-{idx}"))
                .spawn(move || worker_loop(&worker_state))
                .expect("spawn decode worker");
        }
        pool
    }

    pub fn enqueue(&self, job: Job, visible: bool) {
        let mut queues = self.state.queues.lock().unwrap_or_else(PoisonError::into_inner);
        if visible {
            queues.visible.push_back(job);
        } else {
            queues.backfill.push_back(job);
        }
        drop(queues);
        self.state.cv.notify_one();
    }

    pub fn drain_done(&self) -> DrainedDecodes {
        DrainedDecodes {
            done: std::mem::take(
                &mut *self.state.done.lock().unwrap_or_else(PoisonError::into_inner),
            ),
            failed: std::mem::take(
                &mut *self.state.failed.lock().unwrap_or_else(PoisonError::into_inner),
            ),
        }
    }

    pub fn drain_cancelled(&self) -> Vec<usize> {
        std::mem::take(&mut *self.state.cancelled.lock().unwrap_or_else(PoisonError::into_inner))
    }

    pub fn set_wanted(&self, wanted: HashSet<usize>) {
        *self.state.wanted.lock().unwrap_or_else(PoisonError::into_inner) = wanted;
    }

    pub fn is_idle(&self) -> bool {
        if self.state.in_flight.load(Ordering::Acquire) != 0 {
            return false;
        }
        let queues = self.state.queues.lock().unwrap_or_else(PoisonError::into_inner);
        queues.visible.is_empty() && queues.backfill.is_empty()
    }

    pub fn drain_stats(&self) -> (u64, f32, f32) {
        let (sum_ms, max_ms, count) =
            std::mem::take(&mut *self.state.stats.lock().unwrap_or_else(PoisonError::into_inner));
        let avg = if count > 0 { (sum_ms / count as f64) as f32 } else { 0.0 };
        (count, avg, max_ms)
    }
}

fn worker_loop(state: &PoolState) {
    while let Some(job) = next_job(state) {
        state.in_flight.fetch_add(1, Ordering::AcqRel);
        if job.tier == 1
            && !state.wanted.lock().unwrap_or_else(PoisonError::into_inner).contains(&job.store_idx)
        {
            state.in_flight.fetch_sub(1, Ordering::AcqRel);
            state.cancelled.lock().unwrap_or_else(PoisonError::into_inner).push(job.store_idx);
            state.dirty.store(true, Ordering::Relaxed);
            wake(state);
            continue;
        }
        let t0 = Instant::now();
        let mut result = decode_path(&job.path, &job, &state.read_pool);
        if result.is_err()
            && let Some(fallback) = &job.fallback
        {
            result = decode_path(fallback, &job, &state.read_pool);
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        {
            let mut stats = state.stats.lock().unwrap_or_else(PoisonError::into_inner);
            stats.0 += ms;
            stats.1 = stats.1.max(ms as f32);
            stats.2 += 1;
        }
        if job.tier == 1
            && !state.wanted.lock().unwrap_or_else(PoisonError::into_inner).contains(&job.store_idx)
        {
            drop(result);
            state.cancelled.lock().unwrap_or_else(PoisonError::into_inner).push(job.store_idx);
        } else {
            publish(state, &job, result);
        }
        state.in_flight.fetch_sub(1, Ordering::AcqRel);
        state.dirty.store(true, Ordering::Relaxed);
        wake(state);
    }
}

fn next_job(state: &PoolState) -> Option<Job> {
    let mut queues = state.queues.lock().unwrap_or_else(PoisonError::into_inner);
    loop {
        if queues.closed {
            return None;
        }
        if let Some(job) = queues.visible.pop_back() {
            return Some(job);
        }
        if let Some(job) = queues.backfill.pop_front() {
            return Some(job);
        }
        force_wake(state);
        queues = state.cv.wait(queues).unwrap_or_else(PoisonError::into_inner);
    }
}

fn publish(state: &PoolState, job: &Job, result: Result<Decoded, String>) {
    match result {
        Ok(mut decoded) => {
            let (data, compressed, recycle) = if job.tier == 2 && job.compressed && !decoded.blocks
            {
                (bc1_encode(&decoded.data, job.w, job.h), true, None)
            } else {
                (
                    std::mem::take(&mut decoded.data),
                    job.compressed && decoded.blocks,
                    decoded.recycle.take(),
                )
            };
            state.uploads.lock().unwrap_or_else(PoisonError::into_inner).push(Upload {
                tier: job.tier,
                layer: job.layer,
                x: job.x,
                y: job.y,
                w: job.w,
                h: job.h,
                compressed,
                data,
                recycle,
            });
            state
                .done
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(DecodeDone { store_idx: job.store_idx, tier: job.tier });
        }
        Err(err) => {
            state.failed.lock().unwrap_or_else(PoisonError::into_inner).push(DecodeFailed {
                store_idx: job.store_idx,
                tier: job.tier,
                error: format!("{}: {err}", job.path),
            });
        }
    }
}

fn wake(state: &PoolState) {
    let now = Instant::now();
    let mut last = state.last_wake.lock().unwrap_or_else(PoisonError::into_inner);
    if now.duration_since(*last) > WAKE_COALESCE {
        *last = now;
        state.dirty.store(false, Ordering::Relaxed);
        let _ = state.tx.unbounded_send(Wake::Decoded);
    }
}

fn force_wake(state: &PoolState) {
    if !state.dirty.swap(false, Ordering::Relaxed) {
        return;
    }
    *state.last_wake.lock().unwrap_or_else(PoisonError::into_inner) = Instant::now();
    let _ = state.tx.unbounded_send(Wake::Decoded);
}

fn far_block_len(job: &Job) -> usize {
    (job.w / 4) as usize * (job.h / 4) as usize * 8
}

fn preview_dimensions_safe(width: u32, height: u32) -> bool {
    width > 0
        && height > 0
        && width <= MAX_SOURCE_EDGE
        && height <= MAX_SOURCE_EDGE
        && u64::from(width)
            .checked_mul(u64::from(height))
            .is_some_and(|pixels| pixels <= MAX_SOURCE_PIXELS)
}

pub fn validate_near_container(data: &[u8]) -> Result<(), String> {
    if data.len() < 12 || &data[0..4] != b"SKB1" {
        return Err(String::from("near container: bad magic"));
    }
    if data[4] != 0 {
        return Err(String::from("near container: unsupported format"));
    }
    let levels = data[5] as usize;
    let table_end = 12 + levels * 8;
    if levels == 0 || data.len() < table_end {
        return Err(String::from("near container: truncated header"));
    }
    let mut width = u16::from_le_bytes([data[6], data[7]]) as u32;
    let mut height = u16::from_le_bytes([data[8], data[9]]) as u32;
    if width == 0 || height == 0 {
        return Err(String::from("near container: empty dimensions"));
    }
    if data.len() != near_container_bytes(width, height) {
        return Err(String::from("near container: unexpected size"));
    }
    let mut off = table_end;
    for idx in 0..levels {
        let pos = 12 + idx * 8;
        let level_width = u16::from_le_bytes([data[pos], data[pos + 1]]) as u32;
        let level_height = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as u32;
        let len = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            as usize;
        let expected = ((width + 3) & !3) as usize * ((height + 3) & !3) as usize;
        if level_width != width || level_height != height || len != expected {
            return Err(String::from("near container: invalid mip layout"));
        }
        off = off.checked_add(len).ok_or("near container: length overflow")?;
        width /= 2;
        height /= 2;
    }
    if off != data.len() {
        return Err(String::from("near container: size mismatch"));
    }
    Ok(())
}

fn decode_path(path: &str, job: &Job, read_pool: &Arc<BufPool>) -> Result<Decoded, String> {
    let metadata = std::fs::metadata(path).map_err(|err| err.to_string())?;
    if !metadata.is_file() {
        return Err(String::from("preview is not a regular file"));
    }
    if metadata.len() > MAX_ENCODED_BYTES {
        return Err(format!(
            "encoded preview is {} bytes (limit is {MAX_ENCODED_BYTES})",
            metadata.len()
        ));
    }
    if !preview_dimensions_safe(job.w, job.h) {
        return Err(format!("unsafe target dimensions {}x{}", job.w, job.h));
    }
    let target_pixels = u64::from(job.w) * u64::from(job.h);
    let target_bytes = target_pixels
        .checked_mul(4)
        .and_then(|bytes| usize::try_from(bytes).ok())
        .filter(|bytes| *bytes <= MAX_DECODE_BYTES)
        .ok_or_else(|| format!("target allocation exceeds {MAX_DECODE_BYTES} bytes"))?;

    if path.ends_with(".bc7") {
        if (job.w, job.h) != (PREVIEW_WIDTH, PREVIEW_HEIGHT) {
            return Err(format!("near target must be {PREVIEW_WIDTH}x{PREVIEW_HEIGHT}"));
        }
        if metadata.len() != NEAR_CONTAINER_BYTES as u64 {
            return Err(format!(
                "near container is {} bytes (expected {NEAR_CONTAINER_BYTES})",
                metadata.len()
            ));
        }
        let mut data = read_pool.take_bounded().ok_or("preview decoder closed")?;
        let read = std::fs::File::open(path).and_then(|mut file| {
            file.read_exact(&mut data)?;
            let mut extra = [0u8; 1];
            if file.read(&mut extra)? != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "near container grew while reading",
                ));
            }
            Ok(())
        });
        if let Err(err) = read {
            read_pool.put(data);
            return Err(err.to_string());
        }
        if let Err(err) = validate_near_container(&data) {
            read_pool.put(data);
            return Err(err);
        }
        return Ok(Decoded { data, blocks: true, recycle: Some(read_pool.clone()) });
    }
    if path.ends_with(".bc1") {
        let expected = far_block_len(job);
        let data = std::fs::read(path).map_err(|err| err.to_string())?;
        if data.len() != expected {
            return Err(format!("far block {} bytes, want {expected}", data.len()));
        }
        return Ok(Decoded { data, blocks: true, recycle: None });
    }

    let (sw, sh, rgba) =
        if has_extension(path, WEBP_EXTENSION) { decode_webp(path)? } else { decode_image(path)? };
    debug_assert_eq!(target_bytes, job.w as usize * job.h as usize * 4);
    let rgba = resize_thumbnail_area(&rgba, sw, sh, job.w, job.h);
    Ok(Decoded { data: rgba, blocks: false, recycle: None })
}

fn decode_webp(path: &str) -> Result<(u32, u32, Vec<u8>), String> {
    let file = std::fs::File::open(path).map_err(|err| err.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut decoder = image_webp::WebPDecoder::new(reader).map_err(|err| err.to_string())?;
    let (sw, sh) = decoder.dimensions();
    if !preview_dimensions_safe(sw, sh) {
        return Err(format!("unsafe source dimensions {sw}x{sh}"));
    }
    let source_pixels = u64::from(sw) * u64::from(sh);
    let bpp = if decoder.has_alpha() { 4usize } else { 3usize };
    let source_bytes = source_pixels
        .checked_mul(bpp as u64)
        .and_then(|bytes| usize::try_from(bytes).ok())
        .filter(|bytes| *bytes <= MAX_DECODE_BYTES)
        .ok_or_else(|| format!("source allocation exceeds {MAX_DECODE_BYTES} bytes"))?;
    if decoder.output_buffer_size() != Some(source_bytes) {
        return Err(String::from("inconsistent WebP output size"));
    }
    let mut src = vec![0u8; source_bytes];
    decoder.read_image(&mut src).map_err(|err| err.to_string())?;
    let rgba = if bpp == 3 {
        let mut out = vec![255u8; source_pixels as usize * 4];
        for (idx, px) in src.chunks_exact(3).enumerate() {
            out[idx * 4] = px[0];
            out[idx * 4 + 1] = px[1];
            out[idx * 4 + 2] = px[2];
        }
        out
    } else {
        src
    };
    Ok((sw, sh, rgba))
}

fn decode_image(path: &str) -> Result<(u32, u32, Vec<u8>), String> {
    let mut reader = image::ImageReader::open(path)
        .and_then(image::ImageReader::with_guessed_format)
        .map_err(|err| err.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_SOURCE_EDGE);
    limits.max_image_height = Some(MAX_SOURCE_EDGE);
    limits.max_alloc = Some(MAX_DECODE_BYTES as u64);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|err| err.to_string())?;
    let (sw, sh) = (decoded.width(), decoded.height());
    if !preview_dimensions_safe(sw, sh) {
        return Err(format!("unsafe source dimensions {sw}x{sh}"));
    }
    Ok((sw, sh, decoded.into_rgba8().into_raw()))
}

#[cfg(test)]
fn resize_area(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    resize_area_region(src, sw, (0, 0, sw, sh), (dw, dh))
}

fn resize_thumbnail_area(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let crop = cover_source_rect(sw, sh, PREVIEW_WIDTH, PREVIEW_HEIGHT);
    resize_area_region(src, sw, crop, (dw, dh))
}

fn cover_source_rect(sw: u32, sh: u32, aspect_w: u32, aspect_h: u32) -> (u32, u32, u32, u32) {
    if u64::from(sw) * u64::from(aspect_h) > u64::from(sh) * u64::from(aspect_w) {
        let crop_w = ((u64::from(sh) * u64::from(aspect_w) + u64::from(aspect_h) / 2)
            / u64::from(aspect_h))
        .clamp(1, u64::from(sw)) as u32;
        ((sw - crop_w) / 2, 0, crop_w, sh)
    } else {
        let crop_h = ((u64::from(sw) * u64::from(aspect_h) + u64::from(aspect_w) / 2)
            / u64::from(aspect_w))
        .clamp(1, u64::from(sh)) as u32;
        (0, (sh - crop_h) / 2, sw, crop_h)
    }
}

fn resize_area_region(
    src: &[u8],
    source_stride: u32,
    (sx, sy, sw, sh): (u32, u32, u32, u32),
    (dw, dh): (u32, u32),
) -> Vec<u8> {
    let mut out = vec![0u8; dw as usize * dh as usize * 4];
    let bx = (sw as f32 / dw as f32).max(1.0) as u32;
    let by = (sh as f32 / dh as f32).max(1.0) as u32;
    for dy in 0..dh {
        let source_y = sy + (dy as u64 * sh as u64 / dh as u64) as u32;
        for dx in 0..dw {
            let source_x = sx + (dx as u64 * sw as u64 / dw as u64) as u32;
            let mut acc = [0u32; 4];
            let mut count = 0u32;
            for oy in 0..by {
                let sample_y = (source_y + oy).min(sy + sh - 1);
                for ox in 0..bx {
                    let sample_x = (source_x + ox).min(sx + sw - 1);
                    let off = (sample_y as usize * source_stride as usize + sample_x as usize) * 4;
                    acc[0] += src[off] as u32;
                    acc[1] += src[off + 1] as u32;
                    acc[2] += src[off + 2] as u32;
                    acc[3] += src[off + 3] as u32;
                    count += 1;
                }
            }
            let off = (dy as usize * dw as usize + dx as usize) * 4;
            out[off] = (acc[0] / count) as u8;
            out[off + 1] = (acc[1] / count) as u8;
            out[off + 2] = (acc[2] / count) as u8;
            out[off + 3] = (acc[3] / count) as u8;
        }
    }
    out
}

pub fn bc1_encode(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let params =
        texpresso::Params { algorithm: texpresso::Algorithm::RangeFit, ..Default::default() };
    let mut out = vec![0u8; texpresso::Format::Bc1.compressed_size(w, h)];
    texpresso::Format::Bc1.compress(rgba, w, h, params, &mut out);
    out
}

pub fn is_thumb(path: &str) -> bool {
    THUMB_EXTENSIONS.iter().any(|ext| has_extension(path, ext))
}

fn has_extension(path: &str, ext: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, found)| found.eq_ignore_ascii_case(ext))
}

mod tests;
