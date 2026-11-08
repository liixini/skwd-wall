#![cfg(test)]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::{
    DecodeDone, DecodePool, Job, MAX_ENCODED_BYTES, NEAR_CONTAINER_BYTES, decode_path,
    far_block_len, is_thumb, near_container_bytes, preview_dimensions_safe, resize_area,
    resize_thumbnail_area, validate_near_container,
};
use crate::contracts::preview::BufPool;

#[test]
fn near_container_validation() {
    let mut valid = b"SKB1".to_vec();
    valid.extend_from_slice(&[0u8, 1, 4, 0, 4, 0, 0, 0]);
    valid.extend_from_slice(&[4, 0, 4, 0, 16, 0, 0, 0]);
    valid.extend_from_slice(&[0u8; 16]);
    assert!(validate_near_container(&valid).is_ok());

    let mut trunc = b"SKB1".to_vec();
    trunc.extend_from_slice(&[0u8, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(trunc.len(), 12);
    assert!(validate_near_container(&trunc).is_err());

    assert!(validate_near_container(&[0u8; 32]).is_err());
    let mut extra = valid.clone();
    extra.push(0);
    assert!(validate_near_container(&extra).is_err());
    assert!(validate_near_container(&[]).is_err());
}

fn far_job() -> Job {
    Job {
        store_idx: 0,
        path: String::new(),
        fallback: None,
        tier: 2,
        layer: 0,
        x: 0,
        y: 0,
        w: 160,
        h: 88,
        compressed: true,
    }
}

fn near_job() -> Job {
    Job { tier: 1, w: 640, h: 360, ..far_job() }
}

fn read_pool(cap: usize) -> Arc<BufPool> {
    Arc::new(BufPool::new_bounded(NEAR_CONTAINER_BYTES, cap))
}

#[test]
fn far_block_len_bc1() {
    assert_eq!(far_block_len(&far_job()), 7040);
}

#[test]
fn preview_dimensions_bounds() {
    assert!(preview_dimensions_safe(640, 360));
    assert!(preview_dimensions_safe(3840, 2160));
    assert!(!preview_dimensions_safe(0, 360));
    assert!(!preview_dimensions_safe(4096, 4096));
    assert!(!preview_dimensions_safe(4097, 1));
    assert!(!preview_dimensions_safe(1, 4097));
}

#[test]
fn decode_far_block() {
    let temp = std::env::temp_dir().join(format!("skwd_block_{}.bc1", std::process::id()));
    let path = temp.to_str().unwrap();
    std::fs::write(&temp, vec![7u8; 7040]).unwrap();
    let decoded = decode_path(path, &far_job(), &read_pool(1)).unwrap();
    assert!(decoded.blocks);
    assert!(decoded.recycle.is_none());
    assert_eq!(decoded.data.len(), 7040);

    std::fs::write(&temp, vec![0u8; 100]).unwrap();
    assert!(decode_path(path, &far_job(), &read_pool(1)).is_err());
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn decode_jpeg_thumb() {
    let temp = std::env::temp_dir().join(format!("skwd_thumb_{}.thumb.jpg", std::process::id()));
    image::RgbImage::from_pixel(320, 180, image::Rgb([200, 40, 10])).save(&temp).unwrap();
    let job = Job { compressed: false, ..far_job() };
    let decoded = decode_path(temp.to_str().unwrap(), &job, &read_pool(1)).unwrap();
    let _ = std::fs::remove_file(&temp);
    assert!(!decoded.blocks);
    assert_eq!(decoded.data.len(), 160 * 88 * 4);
    assert!(decoded.data[0] > 150 && decoded.data[3] == u8::MAX);
}

#[test]
fn decode_rejects_oversized() {
    let temp = std::env::temp_dir().join(format!("skwd_oversized_{}.bc1", std::process::id()));
    let file = std::fs::File::create(&temp).unwrap();
    file.set_len(MAX_ENCODED_BYTES + 1).unwrap();
    assert!(decode_path(temp.to_str().unwrap(), &far_job(), &read_pool(1)).is_err());

    file.set_len(7040).unwrap();
    let unsafe_job = Job { w: 0, ..far_job() };
    assert!(decode_path(temp.to_str().unwrap(), &unsafe_job, &read_pool(1)).is_err());
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn decode_near_magic() {
    let temp = std::env::temp_dir().join(format!("skwd_near_{}.bc7", std::process::id()));
    let path = temp.to_str().unwrap();
    let good = skb1_container();
    std::fs::write(&temp, &good).unwrap();
    let pool = read_pool(1);
    let decoded = decode_path(path, &near_job(), &pool).unwrap();
    assert!(decoded.blocks);
    assert!(decoded.recycle.is_some());
    assert_eq!(&decoded.data[0..4], b"SKB1");

    std::fs::write(&temp, vec![9u8; 64]).unwrap();
    drop(decoded);
    assert!(decode_path(path, &near_job(), &pool).is_err());
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn near_pool_recycles() {
    let temp = std::env::temp_dir().join(format!("skwd_near_pool_{}.bc7", std::process::id()));
    let path = temp.to_str().unwrap();
    let pool = read_pool(1);
    let seed = pool.take_bounded().unwrap();
    let pointer = seed.as_ptr();
    pool.put(seed);

    std::fs::write(&temp, vec![9u8; NEAR_CONTAINER_BYTES]).unwrap();
    assert!(decode_path(path, &near_job(), &pool).is_err());
    let recovered = pool.take_bounded().unwrap();
    assert_eq!(recovered.as_ptr(), pointer);
    pool.put(recovered);

    std::fs::write(&temp, skb1_container()).unwrap();
    let mut decoded = decode_path(path, &near_job(), &pool).unwrap();
    let upload = crate::contracts::preview::Upload {
        tier: 1,
        layer: 0,
        x: 0,
        y: 0,
        w: 640,
        h: 360,
        compressed: decoded.blocks,
        data: std::mem::take(&mut decoded.data),
        recycle: decoded.recycle.take(),
    };
    drop(upload);
    let recovered = pool.take_bounded().unwrap();
    assert_eq!(recovered.as_ptr(), pointer);
    pool.put(recovered);

    let cancel_pool = read_pool(1);
    let data = cancel_pool.take_bounded().unwrap();
    let pointer = data.as_ptr();
    drop(super::Decoded { data, blocks: true, recycle: Some(cancel_pool.clone()) });
    let recovered = cancel_pool.take_bounded().unwrap();
    assert_eq!(recovered.as_ptr(), pointer);

    std::fs::write(&temp, vec![0u8; NEAR_CONTAINER_BYTES - 1]).unwrap();
    assert!(decode_path(path, &near_job(), &pool).is_err());
    std::fs::write(&temp, vec![0u8; NEAR_CONTAINER_BYTES + 1]).unwrap();
    assert!(decode_path(path, &near_job(), &pool).is_err());
    let _ = std::fs::remove_file(&temp);
}

#[cfg(feature = "obs-heap")]
#[test]
fn warm_near_no_allocs() {
    let temp = std::env::temp_dir().join(format!("skwd_near_alloc_{}.bc7", std::process::id()));
    let path = temp.to_str().unwrap();
    std::fs::write(&temp, skb1_container()).unwrap();
    let pool = read_pool(1);
    drop(decode_path(path, &near_job(), &pool).unwrap());

    let before = crate::infrastructure::observability::allocation::thread_alloc_count();
    for _ in 0..64 {
        drop(decode_path(path, &near_job(), &pool).unwrap());
    }
    let after = crate::infrastructure::observability::allocation::thread_alloc_count();
    assert_eq!(after - before, 0);
    let _ = std::fs::remove_file(&temp);
}

#[cfg(all(feature = "obs-heap", target_os = "linux"))]
#[test]
#[ignore = "retained manual allocation/RSS evidence; run with SKWD_NEAR_POOL_BENCH_MODE"]
fn near_pool_scroll_measurement() {
    const FILES: usize = 64;
    const DEFAULT_ITERATIONS: usize = 4096;

    let mode = std::env::var("SKWD_NEAR_POOL_BENCH_MODE")
        .expect("set SKWD_NEAR_POOL_BENCH_MODE=baseline or candidate");
    assert!(matches!(mode.as_str(), "baseline" | "candidate"));
    let iterations = std::env::var("SKWD_NEAR_POOL_BENCH_ITERATIONS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_ITERATIONS);
    assert!(iterations >= FILES && iterations.is_multiple_of(FILES));

    let directory =
        std::env::temp_dir().join(format!("skwd-near-pool-bench-{}-{mode}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let paths = (0..FILES)
        .map(|index| {
            let path = directory.join(format!("{index:02}.bc7"));
            let mut container = skb1_container();
            *container.last_mut().unwrap() = index as u8;
            std::fs::write(&path, container).unwrap();
            path
        })
        .collect::<Vec<_>>();

    let pool = read_pool(4);
    if mode == "candidate" {
        let buffers = (0..4).map(|_| pool.take_bounded().unwrap()).collect::<Vec<_>>();
        for buffer in buffers {
            pool.put(buffer);
        }
    }
    // Exclude setup and allocator residue from the measured workload.
    // SAFETY: `malloc_trim` accepts any non-negative padding value and does not
    // receive or expose Rust-owned pointers. This Linux-only benchmark uses zero
    // solely to discard allocator residue before sampling process memory.
    unsafe { libc::malloc_trim(0) };
    let (rss_before_kib, pss_before_kib, _) = self_memory_kib();
    let allocations_before = crate::infrastructure::observability::allocation::thread_alloc_count();
    let started = Instant::now();
    let mut order_digest = 0xcbf2_9ce4_8422_2325u64;
    for index in 0..iterations {
        let path = &paths[index % FILES];
        let marker = if mode == "baseline" {
            // Mirror the successful pre-pool decode path, including its outer
            // metadata/shape checks before the fresh whole-file read.
            let metadata = std::fs::metadata(path).unwrap();
            assert!(metadata.is_file());
            assert_eq!(metadata.len(), NEAR_CONTAINER_BYTES as u64);
            assert!(preview_dimensions_safe(near_job().w, near_job().h));
            let data = std::fs::read(path).unwrap();
            validate_near_container(&data).unwrap();
            *data.last().unwrap()
        } else {
            let decoded = decode_path(path.to_str().unwrap(), &near_job(), &pool).unwrap();
            *decoded.data.last().unwrap()
        };
        order_digest ^= u64::from(marker);
        order_digest = order_digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let elapsed = started.elapsed();
    let allocations =
        crate::infrastructure::observability::allocation::thread_alloc_count() - allocations_before;
    // SAFETY: as above, no Rust pointer or lifetime crosses the C boundary;
    // trimming allocator residue makes the post-workload RSS/PSS sample useful.
    unsafe { libc::malloc_trim(0) };
    let (rss_after_kib, pss_after_kib, rss_high_water_kib) = self_memory_kib();

    if mode == "candidate" {
        assert_eq!(allocations, 0, "warm pooled reads must remain allocation-free");
    } else {
        assert!(allocations >= iterations, "fresh reads must expose the former allocation path");
    }
    assert_eq!(order_digest, 0xe775_f8dc_ee79_1325);
    println!(
        "{}",
        serde_json::json!({
            "mode": mode,
            "files": FILES,
            "iterations": iterations,
            "containerBytes": NEAR_CONTAINER_BYTES,
            "payloadAllocationBytes": if mode == "baseline" {
                iterations * NEAR_CONTAINER_BYTES
            } else {
                0
            },
            "allocations": allocations,
            "elapsedMs": elapsed.as_secs_f64() * 1000.0,
            "rssBeforeKiB": rss_before_kib,
            "rssAfterKiB": rss_after_kib,
            "pssBeforeKiB": pss_before_kib,
            "pssAfterKiB": pss_after_kib,
            "rssHighWaterKiB": rss_high_water_kib,
            "heapPeakBytes": crate::infrastructure::observability::allocation::peak_bytes(),
            "retainedPoolBoundBytes": 4 * NEAR_CONTAINER_BYTES,
            "orderDigest": format!("{order_digest:016x}"),
        })
    );

    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(all(feature = "obs-heap", target_os = "linux"))]
fn self_memory_kib() -> (u64, u64, u64) {
    let rollup = std::fs::read_to_string("/proc/self/smaps_rollup").unwrap();
    let value = |name: &str| {
        rollup
            .lines()
            .find_map(|line| {
                line.strip_prefix(name)
                    .and_then(|tail| tail.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
            .unwrap_or(0)
    };
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let high_water = status
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")
                .and_then(|tail| tail.split_whitespace().next())
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0);
    (value("Rss:"), value("Pss:"), high_water)
}

#[test]
fn thumb_suffix() {
    assert!(is_thumb("a.webp"));
    assert!(is_thumb("/path/to/THUMB.WEBP"));
    assert!(is_thumb("mixed.WeBp"));
    assert!(is_thumb("a.thumb.jpg"));
    assert!(is_thumb("b.JPEG"));
    assert!(is_thumb(".webp"));
    assert!(!is_thumb("webp"));
    assert!(!is_thumb(""));
    assert!(!is_thumb("x.web"));
    assert!(is_thumb("naïve_café.webp"));
    assert!(!is_thumb("café.png"));
}

#[test]
fn resize_area_2x2() {
    let src = [10u8, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160];
    assert_eq!(resize_area(&src, 2, 2, 1, 1), vec![70, 80, 90, 100]);
}

#[test]
fn resize_area_quadrants() {
    let mut src = vec![0u8; 4 * 4 * 4];
    for y in 0..4usize {
        for x in 0..4usize {
            let q = (y / 2) as u8 * 2 + (x / 2) as u8;
            let off = (y * 4 + x) * 4;
            src[off..off + 4].copy_from_slice(&[q * 10, q * 10 + 1, q * 10 + 2, 255]);
        }
    }
    let out = resize_area(&src, 4, 4, 2, 2);
    for q in 0..4u8 {
        let off = q as usize * 4;
        assert_eq!(&out[off..off + 4], &[q * 10, q * 10 + 1, q * 10 + 2, 255]);
    }
}

#[test]
fn resize_area_upscale() {
    let out = resize_area(&[5, 6, 7, 8], 1, 1, 3, 2);
    assert_eq!(out.len(), 3 * 2 * 4);
    for px in out.chunks_exact(4) {
        assert_eq!(px, &[5, 6, 7, 8]);
    }
}

#[test]
fn resize_area_non_divisible() {
    let src = vec![9u8; 5 * 3 * 4];
    let out = resize_area(&src, 5, 3, 2, 2);
    assert_eq!(out.len(), 2 * 2 * 4);
    assert!(out.iter().all(|&byte| byte == 9));
}

#[test]
fn thumbnail_resize_crops_square_without_squashing() {
    let mut src = vec![0u8; 4 * 4 * 4];
    for y in 0..4usize {
        for x in 0..4usize {
            let off = (y * 4 + x) * 4;
            src[off..off + 4].copy_from_slice(&[(y + 1) as u8, 0, 0, 255]);
        }
    }

    let out = resize_thumbnail_area(&src, 4, 4, 4, 2);
    assert!(out[..4 * 4].chunks_exact(4).all(|pixel| pixel == [2, 0, 0, 255]));
    assert!(out[4 * 4..].chunks_exact(4).all(|pixel| pixel == [3, 0, 0, 255]));
}

#[test]
fn thumbnail_resize_crops_wide_source_from_the_centre() {
    let mut src = vec![0u8; 8 * 2 * 4];
    for y in 0..2usize {
        for x in 0..8usize {
            let off = (y * 8 + x) * 4;
            src[off..off + 4].copy_from_slice(&[(x + 1) as u8, 0, 0, 255]);
        }
    }

    let out = resize_thumbnail_area(&src, 8, 2, 4, 2);
    let first_row = out[..4 * 4].chunks_exact(4).map(|pixel| pixel[0]).collect::<Vec<_>>();
    assert_eq!(first_row, vec![3, 4, 5, 6]);
}

fn far_job_for(idx: usize, path: &str) -> Job {
    Job { store_idx: idx, path: path.to_string(), ..far_job() }
}

type WakeRx = futures_channel::mpsc::UnboundedReceiver<crate::infrastructure::runtime::Wake>;

fn test_pool(workers: usize) -> (DecodePool, super::UploadQueue, WakeRx) {
    let uploads: super::UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = futures_channel::mpsc::unbounded();
    (DecodePool::start(uploads.clone(), tx, workers), uploads, rx)
}

fn assert_pool_released(state: &std::sync::Weak<super::PoolState>) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while state.strong_count() != 0 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(state.strong_count(), 0, "decoder workers retained the dropped pool");
}

#[test]
fn pool_drop_releases_idle_workers() {
    for _ in 0..16 {
        let (pool, _uploads, _rx) = test_pool(6);
        let state = Arc::downgrade(&pool.state);
        drop(pool);
        assert_pool_released(&state);
    }
}

#[test]
fn pool_clone_remains_usable_until_last_drop() {
    let (pool, _uploads, _rx) = test_pool(1);
    let state = Arc::downgrade(&pool.state);
    let remaining = pool.clone();
    drop(pool);
    remaining.enqueue(far_job_for(1, "missing-preview.bc1"), true);
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut failed = Vec::new();
    while failed.is_empty() && Instant::now() < deadline {
        failed = remaining.drain_done().failed;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(failed.len(), 1);
    drop(remaining);
    assert_pool_released(&state);
}

#[test]
fn pool_concurrent_last_clients_release_workers() {
    let (pool, _uploads, _rx) = test_pool(2);
    let state = Arc::downgrade(&pool.state);
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let other = pool.clone();
    let other_barrier = barrier.clone();
    let thread = std::thread::spawn(move || {
        other_barrier.wait();
        drop(other);
    });
    barrier.wait();
    drop(pool);
    thread.join().unwrap();
    assert_pool_released(&state);
}

#[test]
fn pool_drop_releases_worker_with_exhausted_read_buffers() {
    let temp = std::env::temp_dir().join(format!("skwd_drop_{}.bc7", std::process::id()));
    std::fs::write(&temp, skb1_container()).unwrap();
    let (pool, _uploads, _rx) = test_pool(1);
    let state = Arc::downgrade(&pool.state);
    let buffers = pool.state.read_pool.clone();
    let held_buffer = buffers.take_bounded().unwrap();
    pool.set_wanted(HashSet::from([8]));
    pool.enqueue(Job { tier: 1, w: 640, h: 360, ..far_job_for(8, temp.to_str().unwrap()) }, true);
    let deadline = Instant::now() + Duration::from_secs(2);
    while pool.state.in_flight.load(std::sync::atomic::Ordering::Acquire) == 0
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(pool.state.in_flight.load(std::sync::atomic::Ordering::Acquire), 1);
    drop(pool);
    assert_pool_released(&state);
    buffers.put(held_buffer);
    assert!(buffers.take_bounded().is_none());
    std::fs::remove_file(temp).unwrap();
}

fn skb1_container() -> Vec<u8> {
    let mut levels = Vec::new();
    let (mut width, mut height) = (640u32, 360u32);
    loop {
        levels.push((width, height));
        let (next_width, next_height) = (width / 2, height / 2);
        if next_width < 16 || next_height < 16 {
            break;
        }
        (width, height) = (next_width, next_height);
    }
    let mut buf = Vec::with_capacity(near_container_bytes(640, 360));
    buf.extend_from_slice(b"SKB1");
    buf.extend_from_slice(&[0, levels.len() as u8]);
    buf.extend_from_slice(&640u16.to_le_bytes());
    buf.extend_from_slice(&360u16.to_le_bytes());
    buf.extend_from_slice(&0u16.to_le_bytes());
    for (width, height) in levels {
        let bytes = ((width + 3) & !3) * ((height + 3) & !3);
        buf.extend_from_slice(&(width as u16).to_le_bytes());
        buf.extend_from_slice(&(height as u16).to_le_bytes());
        buf.extend_from_slice(&bytes.to_le_bytes());
    }
    buf.resize(NEAR_CONTAINER_BYTES, 0);
    buf
}

#[test]
fn pool_lifo_then_fifo() {
    let (pool, _uploads, _rx) = test_pool(0);
    pool.enqueue(far_job_for(1, "unused"), false);
    pool.enqueue(far_job_for(2, "unused"), false);
    pool.enqueue(far_job_for(3, "unused"), true);
    pool.enqueue(far_job_for(4, "unused"), true);

    let order = (0..4).map(|_| super::next_job(&pool.state).unwrap().store_idx).collect::<Vec<_>>();
    assert_eq!(order, vec![4, 3, 1, 2]);
}

#[test]
fn pool_cancels_unwanted() {
    let temp = std::env::temp_dir().join(format!("skwd_cancel_{}.bc7", std::process::id()));
    std::fs::write(&temp, skb1_container()).unwrap();
    let (pool, uploads, _rx) = test_pool(1);
    pool.set_wanted(HashSet::new());
    pool.enqueue(Job { tier: 1, w: 640, h: 360, ..far_job_for(7, temp.to_str().unwrap()) }, true);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut cancelled = Vec::new();
    while cancelled.is_empty() && Instant::now() < deadline {
        cancelled = pool.drain_cancelled();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(cancelled, vec![7]);
    let drained = pool.drain_done();
    assert!(drained.done.is_empty() && drained.failed.is_empty());
    assert!(uploads.lock().unwrap().is_empty());
    assert!(pool.is_idle());
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn pool_decodes_wanted() {
    let temp = std::env::temp_dir().join(format!("skwd_want_{}.bc7", std::process::id()));
    std::fs::write(&temp, skb1_container()).unwrap();
    let (pool, uploads, _rx) = test_pool(1);
    pool.set_wanted(HashSet::from([8]));
    pool.enqueue(Job { tier: 1, w: 640, h: 360, ..far_job_for(8, temp.to_str().unwrap()) }, true);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut done = Vec::new();
    while done.is_empty() && Instant::now() < deadline {
        done = pool.drain_done().done;
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(done, vec![DecodeDone { store_idx: 8, tier: 1 }]);
    let ups = uploads.lock().unwrap();
    assert_eq!(ups.len(), 1);
    assert_eq!(ups[0].tier, 1);
    assert_eq!(&ups[0].data[0..4], b"SKB1");
    drop(ups);
    assert!(pool.drain_cancelled().is_empty());
    while !pool.is_idle() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(pool.is_idle());
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn bench_bc1_encode() {
    use texpresso::{Algorithm, Format, Params};
    let (w, h) = (640usize, 360usize);
    let rgba = vec![123u8; w * h * 4];
    let size = Format::Bc1.compressed_size(w, h);
    assert_eq!(size, w * h / 2);
    let mut out = vec![0u8; size];
    for (label, algo) in [("RangeFit", Algorithm::RangeFit), ("ClusterFit", Algorithm::ClusterFit)]
    {
        let params = Params { algorithm: algo, ..Default::default() };
        let start = std::time::Instant::now();
        for _ in 0..20 {
            Format::Bc1.compress(&rgba, w, h, params, &mut out);
        }
        let ms = start.elapsed().as_secs_f64() * 1000.0 / 20.0;
        println!("BC1 {label} 640x360: {ms:.2} ms/frame ({size} bytes, 8:1)");
    }
}
