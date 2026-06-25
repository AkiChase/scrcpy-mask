use std::{
    collections::VecDeque,
    num::NonZero,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Source};

use super::media::AudioCodec;

pub const AUDIO_SAMPLE_RATE: u32 = 48_000;
pub const AUDIO_CHANNELS: u16 = 2;

const DEFAULT_TARGET_BUFFER_MS: usize = 50;
const FLAC_TARGET_BUFFER_MS: usize = 120;
const BUFFER_MARGIN_MS: usize = 60;
const LOCAL_BUFFER_MS: usize = 10;
const AUDIO_OUTPUT_BUFFER_FRAMES: u32 = AUDIO_SAMPLE_RATE / 100;

#[derive(Clone, Copy)]
struct AudioBufferConfig {
    target_frames: usize,
    max_frames: usize,
    local_frames: usize,
}

impl AudioBufferConfig {
    fn for_codec(codec: AudioCodec) -> Self {
        let target_ms = match codec {
            AudioCodec::Flac => FLAC_TARGET_BUFFER_MS,
            AudioCodec::Opus | AudioCodec::Aac | AudioCodec::Raw => DEFAULT_TARGET_BUFFER_MS,
        };
        let target_frames = frames_from_ms(target_ms);
        Self {
            target_frames,
            max_frames: target_frames * 11 / 10 + frames_from_ms(BUFFER_MARGIN_MS),
            local_frames: frames_from_ms(LOCAL_BUFFER_MS),
        }
    }

    fn target_samples(self) -> usize {
        self.target_frames * AUDIO_CHANNELS as usize
    }

    fn max_samples(self) -> usize {
        self.max_frames * AUDIO_CHANNELS as usize
    }

    fn local_samples(self) -> usize {
        self.local_frames * AUDIO_CHANNELS as usize
    }
}

fn frames_from_ms(ms: usize) -> usize {
    (AUDIO_SAMPLE_RATE as usize * ms) / 1000
}

#[derive(Clone)]
pub struct AudioSampleQueue {
    inner: Arc<AudioSampleQueueInner>,
}

struct AudioSampleQueueInner {
    samples: Mutex<AudioRingBuffer>,
    sample_count: AtomicUsize,
    config: AudioBufferConfig,
    closed: AtomicBool,
}

impl AudioSampleQueue {
    fn new(config: AudioBufferConfig) -> Self {
        Self {
            inner: Arc::new(AudioSampleQueueInner {
                samples: Mutex::new(AudioRingBuffer::new(config.max_samples())),
                sample_count: AtomicUsize::new(0),
                config,
                closed: AtomicBool::new(false),
            }),
        }
    }

    pub fn push_samples(&self, samples: impl IntoIterator<Item = f32>) {
        let incoming = samples.into_iter().collect::<Vec<_>>();
        if incoming.is_empty() {
            return;
        }

        let mut queue = self.inner.samples.lock().unwrap();
        let total_samples = queue.len() + incoming.len();
        let mut incoming_start = 0;
        if total_samples > self.inner.config.max_samples() {
            let drop_count = total_samples.saturating_sub(self.inner.config.target_samples());
            let dropped = queue.drop_oldest(drop_count);
            incoming_start = drop_count.saturating_sub(dropped).min(incoming.len());
        }

        queue.push_slice(&incoming[incoming_start..]);
        self.inner
            .sample_count
            .store(queue.len(), Ordering::Release);
    }

    fn pop_samples(&self, samples: &mut VecDeque<f32>, max_samples: usize) {
        let mut queue = self.inner.samples.lock().unwrap();
        queue.pop_into(samples, max_samples);
        self.inner
            .sample_count
            .store(queue.len(), Ordering::Release);
    }

    fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    fn sample_count(&self) -> usize {
        self.inner.sample_count.load(Ordering::Acquire)
    }

    fn close(&self) {
        self.inner.closed.store(true, Ordering::Release);
    }
}

struct AudioRingBuffer {
    samples: Vec<f32>,
    start: usize,
    len: usize,
}

impl AudioRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            samples: vec![0.0; capacity],
            start: 0,
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn drop_oldest(&mut self, count: usize) -> usize {
        let count = count.min(self.len);
        if count == 0 {
            return 0;
        }

        self.start = (self.start + count) % self.samples.len();
        self.len -= count;
        count
    }

    fn push_slice(&mut self, incoming: &[f32]) {
        debug_assert!(self.len + incoming.len() <= self.samples.len());
        for &sample in incoming {
            let index = (self.start + self.len) % self.samples.len();
            self.samples[index] = sample;
            self.len += 1;
        }
    }

    fn pop_into(&mut self, out: &mut VecDeque<f32>, max_samples: usize) {
        let count = self.len.min(max_samples);
        for _ in 0..count {
            out.push_back(self.samples[self.start]);
            self.start = (self.start + 1) % self.samples.len();
        }
        self.len -= count;
    }
}

pub struct ScrcpyAudioPlayer {
    queue: AudioSampleQueue,
    _sink: MixerDeviceSink,
}

impl ScrcpyAudioPlayer {
    pub fn new(codec: AudioCodec) -> Result<Self, String> {
        let config = AudioBufferConfig::for_codec(codec);
        let queue = AudioSampleQueue::new(config);
        let sink = open_audio_sink().map_err(|e| format!("Failed to open audio output: {e}"))?;
        sink.mixer().add(ScrcpyAudioSource {
            queue: queue.clone(),
            config,
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

fn open_audio_sink() -> Result<MixerDeviceSink, rodio::stream::DeviceSinkError> {
    DeviceSinkBuilder::from_default_device()?
        .with_buffer_size(rodio::cpal::BufferSize::Fixed(AUDIO_OUTPUT_BUFFER_FRAMES))
        .open_sink_or_fallback()
}

struct ScrcpyAudioSource {
    queue: AudioSampleQueue,
    config: AudioBufferConfig,
    buffered_samples: VecDeque<f32>,
    buffering: bool,
}

impl Iterator for ScrcpyAudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.buffered_samples.pop_front() {
            return Some(sample);
        }

        let closed = self.queue.is_closed();
        if self.buffering && !closed && self.queue.sample_count() < self.config.target_samples() {
            return Some(0.0);
        }
        self.buffering = false;

        self.queue
            .pop_samples(&mut self.buffered_samples, self.config.local_samples());

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
