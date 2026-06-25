use std::{
    collections::VecDeque,
    num::NonZero,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Source};

pub const AUDIO_SAMPLE_RATE: u32 = 48_000;
pub const AUDIO_CHANNELS: u16 = 2;

const MAX_BUFFERED_FRAMES: usize = (AUDIO_SAMPLE_RATE as usize * 120) / 1000;
const TARGET_BUFFERED_FRAMES: usize = (AUDIO_SAMPLE_RATE as usize * 60) / 1000;
const LOCAL_BUFFERED_FRAMES: usize = (AUDIO_SAMPLE_RATE as usize * 10) / 1000;

#[derive(Clone)]
pub struct AudioSampleQueue {
    inner: Arc<AudioSampleQueueInner>,
}

struct AudioSampleQueueInner {
    samples: Mutex<VecDeque<f32>>,
    closed: AtomicBool,
}

impl AudioSampleQueue {
    fn new() -> Self {
        Self {
            inner: Arc::new(AudioSampleQueueInner {
                samples: Mutex::new(VecDeque::new()),
                closed: AtomicBool::new(false),
            }),
        }
    }

    pub fn push_samples(&self, samples: impl IntoIterator<Item = f32>) {
        let mut queue = self.inner.samples.lock().unwrap();
        queue.extend(samples);

        let max_samples = MAX_BUFFERED_FRAMES * AUDIO_CHANNELS as usize;
        if queue.len() <= max_samples {
            return;
        }

        let target_samples = TARGET_BUFFERED_FRAMES * AUDIO_CHANNELS as usize;
        let drop_count = queue.len().saturating_sub(target_samples);
        queue.drain(..drop_count);
    }

    fn pop_samples(&self, samples: &mut VecDeque<f32>, max_samples: usize) {
        let mut queue = self.inner.samples.lock().unwrap();
        let pop_count = queue.len().min(max_samples);
        samples.extend(queue.drain(..pop_count));
    }

    fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    fn sample_count(&self) -> usize {
        self.inner.samples.lock().unwrap().len()
    }

    fn close(&self) {
        self.inner.closed.store(true, Ordering::Release);
    }
}

pub struct ScrcpyAudioPlayer {
    queue: AudioSampleQueue,
    _sink: MixerDeviceSink,
}

impl ScrcpyAudioPlayer {
    pub fn new() -> Result<Self, String> {
        let queue = AudioSampleQueue::new();
        let sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| format!("Failed to open audio output: {e}"))?;
        sink.mixer().add(ScrcpyAudioSource {
            queue: queue.clone(),
            buffered_samples: VecDeque::new(),
            buffering: true,
        });

        Ok(Self { queue, _sink: sink })
    }

    pub fn queue(&self) -> AudioSampleQueue {
        self.queue.clone()
    }
}

impl Drop for ScrcpyAudioPlayer {
    fn drop(&mut self) {
        self.queue.close();
    }
}

struct ScrcpyAudioSource {
    queue: AudioSampleQueue,
    buffered_samples: VecDeque<f32>,
    buffering: bool,
}

impl Iterator for ScrcpyAudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.buffered_samples.pop_front() {
            return Some(sample);
        }

        let target_samples = TARGET_BUFFERED_FRAMES * AUDIO_CHANNELS as usize;
        let closed = self.queue.is_closed();
        if self.buffering && !closed && self.queue.sample_count() < target_samples {
            return Some(0.0);
        }
        self.buffering = false;

        let local_samples = LOCAL_BUFFERED_FRAMES * AUDIO_CHANNELS as usize;
        self.queue
            .pop_samples(&mut self.buffered_samples, local_samples);

        match self.buffered_samples.pop_front() {
            Some(sample) => Some(sample),
            None if closed => None,
            None => {
                self.buffering = true;
                Some(0.0)
            }
        }
    }
}

impl Source for ScrcpyAudioSource {
    fn current_span_len(&self) -> Option<usize> {
        if self.queue.is_closed()
            && self.buffered_samples.is_empty()
            && self.queue.sample_count() == 0
        {
            Some(0)
        } else {
            None
        }
    }

    fn channels(&self) -> NonZero<u16> {
        NonZero::new(AUDIO_CHANNELS).unwrap()
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(AUDIO_SAMPLE_RATE).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
