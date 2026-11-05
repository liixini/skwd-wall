use std::{sync::Arc, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewSize {
    Hero,
    Thumbnail,
    Card,
}

#[derive(Clone, Debug)]
pub struct PreviewFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
    pub delay: Duration,
}

#[derive(Clone, Debug)]
pub struct PreviewFrames {
    pub frames: Vec<PreviewFrame>,
    pub duration: Duration,
}

impl PreviewFrames {
    pub fn animated(&self) -> bool {
        self.frames.len() > 1
    }
}

pub fn preview_frame_index(delays: &[Duration], duration: Duration, elapsed: Duration) -> usize {
    if delays.len() < 2 || duration.is_zero() {
        return 0;
    }

    let mut remaining = elapsed.as_nanos() % duration.as_nanos();
    for (index, delay) in delays.iter().enumerate() {
        if remaining < delay.as_nanos() {
            return index;
        }
        remaining -= delay.as_nanos();
    }
    0
}
