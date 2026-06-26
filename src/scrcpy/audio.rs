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
const INITIAL_BUFFER_MARGIN_MS: usize = 10;
const AVERAGE_RANGE: usize = 128;
const DISCONTINUITY_THRESHOLD_US: i64 = 100_000;
const COMPENSATION_INTERVAL_FRAMES: usize = AUDIO_SAMPLE_RATE as usize;
const COMPENSATION_DISTANCE_FRAMES: usize = AUDIO_SAMPLE_RATE as usize * 4;
const COMPENSATION_MAX_RATE_DIVISOR: isize = 50;
const COMPENSATION_START_THRESHOLD_MS: usize = 4;
const COMPENSATION_STOP_THRESHOLD_MS: usize = 1;
const AUDIO_OUTPUT_BUFFER_FRAMES: u32 = AUDIO_SAMPLE_RATE / 100;

#[derive(Clone, Copy)]
struct AudioBufferConfig {
    target_frames: usize,
    max_played_frames: usize,
    max_initial_frames: usize,
    capacity_frames: usize,
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
            max_played_frames: target_frames * 11 / 10 + frames_from_ms(BUFFER_MARGIN_MS),
            max_initial_frames: target_frames + frames_from_ms(INITIAL_BUFFER_MARGIN_MS),
            capacity_frames: target_frames + AUDIO_SAMPLE_RATE as usize,
            local_frames: frames_from_ms(LOCAL_BUFFER_MS),
        }
    }

    fn target_samples(self) -> usize {
        self.target_frames * AUDIO_CHANNELS as usize
    }

    fn max_played_samples(self) -> usize {
        self.max_played_frames * AUDIO_CHANNELS as usize
    }

    fn max_initial_samples(self) -> usize {
        self.max_initial_frames * AUDIO_CHANNELS as usize
    }

    fn capacity_samples(self) -> usize {
        self.capacity_frames * AUDIO_CHANNELS as usize
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
    regulator: Mutex<AudioRegulator>,
    config: AudioBufferConfig,
    closed: AtomicBool,
    received: AtomicBool,
    played: AtomicBool,
    underflow_frames: AtomicUsize,
}

impl AudioSampleQueue {
    fn new(config: AudioBufferConfig) -> Self {
        Self {
            inner: Arc::new(AudioSampleQueueInner {
                samples: Mutex::new(AudioRingBuffer::new(config.capacity_samples())),
                sample_count: AtomicUsize::new(0),
                regulator: Mutex::new(AudioRegulator::new()),
                config,
                closed: AtomicBool::new(false),
                received: AtomicBool::new(false),
                played: AtomicBool::new(false),
                underflow_frames: AtomicUsize::new(0),
            }),
        }
    }

    pub(crate) fn prepare_push(
        &self,
        input_frames: usize,
        pts: Option<i64>,
    ) -> Option<AudioCompensation> {
        let mut regulator = self.inner.regulator.lock().unwrap();
        let Some(pts) = pts else {
            return None;
        };

        let mut compensation = None;
        if regulator.next_expected_pts != 0
            && pts - regulator.next_expected_pts > DISCONTINUITY_THRESHOLD_US
        {
            self.insert_discontinuity_silence(input_frames);
            regulator.reset(self.inner.config.target_frames);
            self.inner.underflow_frames.store(0, Ordering::Release);
            compensation = Some(AudioCompensation::disabled());
        }

        regulator.next_expected_pts =
            pts + (input_frames as i64 * 1_000_000 / AUDIO_SAMPLE_RATE as i64);
        compensation
    }

    pub(crate) fn push_samples(&self, samples: impl IntoIterator<Item = f32>) -> AudioPushStats {
        let incoming = samples.into_iter().collect::<Vec<_>>();
        if incoming.is_empty() {
            return AudioPushStats::default();
        }

        let mut queue = self.inner.samples.lock().unwrap();
        let mut skipped_samples = 0;
        let incoming_start = incoming
            .len()
            .saturating_sub(self.inner.config.capacity_samples());
        skipped_samples += incoming_start;

        let incoming_len = incoming.len() - incoming_start;
        let total_samples = queue.len() + incoming_len;
        if total_samples > self.inner.config.capacity_samples() {
            let drop_count = total_samples - self.inner.config.capacity_samples();
            skipped_samples += queue.drop_oldest(drop_count);
        }

        queue.push_slice(&incoming[incoming_start..]);

        let max_samples = if self.inner.played.load(Ordering::Acquire) {
            self.inner.config.max_played_samples()
        } else {
            self.inner.config.max_initial_samples()
        };
        if queue.len() > max_samples {
            let drop_count = queue.len() - max_samples;
            skipped_samples += queue.drop_oldest(drop_count);
        }

        let buffered_samples = queue.len();
        self.inner
            .sample_count
            .store(buffered_samples, Ordering::Release);

        AudioPushStats {
            written_frames: samples_to_frames(incoming_len),
            skipped_frames: samples_to_frames(skipped_samples),
            buffered_frames: samples_to_frames(buffered_samples),
        }
    }

    pub(crate) fn finish_push(
        &self,
        input_frames: usize,
        stats: AudioPushStats,
    ) -> Option<AudioCompensation> {
        self.inner.received.store(true, Ordering::Release);
        if !self.inner.played.load(Ordering::Acquire) {
            return None;
        }

        let underflow_frames = self.inner.underflow_frames.swap(0, Ordering::AcqRel);
        let mut regulator = self.inner.regulator.lock().unwrap();
        regulator.finish_push(self.inner.config, input_frames, stats, underflow_frames)
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

    fn mark_played(&self) {
        self.inner.played.store(true, Ordering::Release);
    }

    fn record_underflow_samples(&self, silence_samples: usize) {
        if self.inner.received.load(Ordering::Acquire) {
            self.inner
                .underflow_frames
                .fetch_add(samples_to_frames(silence_samples), Ordering::AcqRel);
        }
    }

    fn close(&self) {
        self.inner.closed.store(true, Ordering::Release);
    }

    fn insert_discontinuity_silence(&self, input_frames: usize) {
        let mut queue = self.inner.samples.lock().unwrap();
        let buffered_frames = samples_to_frames(queue.len());
        if input_frames + buffered_frames >= self.inner.config.target_frames {
            return;
        }

        let silence_frames = self.inner.config.target_frames - buffered_frames - input_frames;
        queue.push_silence(frames_to_samples(silence_frames));
        self.inner
            .sample_count
            .store(queue.len(), Ordering::Release);
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AudioCompensation {
    pub sample_delta: i32,
    pub distance: i32,
}

impl AudioCompensation {
    fn disabled() -> Self {
        Self {
            sample_delta: 0,
            distance: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AudioPushStats {
    written_frames: usize,
    skipped_frames: usize,
    buffered_frames: usize,
}

struct AudioRegulator {
    avg_buffering: MovingAverage,
    frames_since_resync: usize,
    underflow_report_frames: usize,
    compensation_active: bool,
    next_expected_pts: i64,
}

impl AudioRegulator {
    fn new() -> Self {
        Self {
            avg_buffering: MovingAverage::new(AVERAGE_RANGE),
            frames_since_resync: 0,
            underflow_report_frames: 0,
            compensation_active: false,
            next_expected_pts: 0,
        }
    }

    fn reset(&mut self, target_frames: usize) {
        self.avg_buffering.reset(target_frames as f32);
        self.frames_since_resync = 0;
        self.underflow_report_frames = 0;
        self.compensation_active = false;
    }

    fn finish_push(
        &mut self,
        config: AudioBufferConfig,
        input_frames: usize,
        stats: AudioPushStats,
        underflow_frames: usize,
    ) -> Option<AudioCompensation> {
        self.underflow_report_frames += underflow_frames;

        let instant_compensation = stats.written_frames as isize - input_frames as isize;
        let instant_adjustment =
            instant_compensation + underflow_frames as isize - stats.skipped_frames as isize;
        self.avg_buffering.add_instant(instant_adjustment);
        self.avg_buffering.push(stats.buffered_frames as f32);

        self.frames_since_resync += stats.written_frames;
        if self.frames_since_resync < COMPENSATION_INTERVAL_FRAMES {
            return None;
        }

        self.frames_since_resync = 0;
        let avg = self.avg_buffering.get();
        let mut diff = config.target_frames as isize - avg.round() as isize;
        let threshold_ms = if self.compensation_active {
            COMPENSATION_STOP_THRESHOLD_MS
        } else {
            COMPENSATION_START_THRESHOLD_MS
        };
        let threshold = frames_from_ms(threshold_ms) as isize;

        if diff.abs() < threshold {
            diff = 0;
        } else if diff < 0 && stats.buffered_frames < config.target_frames {
            diff = 0;
        }

        let abs_max_diff = COMPENSATION_DISTANCE_FRAMES as isize / COMPENSATION_MAX_RATE_DIVISOR;
        diff = diff.clamp(-abs_max_diff, abs_max_diff);
        self.underflow_report_frames = 0;
        self.compensation_active = diff != 0;

        Some(AudioCompensation {
            sample_delta: diff as i32,
            distance: COMPENSATION_DISTANCE_FRAMES as i32,
        })
    }
}

struct MovingAverage {
    avg: f32,
    range: usize,
    count: usize,
}

impl MovingAverage {
    fn new(range: usize) -> Self {
        Self {
            avg: 0.0,
            range,
            count: 0,
        }
    }

    fn reset(&mut self, value: f32) {
        self.avg = value;
    }

    fn add_instant(&mut self, frames: isize) {
        self.avg = (self.avg + frames as f32).max(0.0);
    }

    fn push(&mut self, value: f32) {
        self.count = (self.count + 1).min(self.range);
        self.avg = ((self.count - 1) as f32 * self.avg + value) / self.count as f32;
    }

    fn get(&self) -> f32 {
        self.avg
    }
}

fn samples_to_frames(samples: usize) -> usize {
    samples / AUDIO_CHANNELS as usize
}

fn frames_to_samples(frames: usize) -> usize {
    frames * AUDIO_CHANNELS as usize
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

    fn push_silence(&mut self, count: usize) {
        debug_assert!(self.len + count <= self.samples.len());
        for _ in 0..count {
            let index = (self.start + self.len) % self.samples.len();
            self.samples[index] = 0.0;
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
            started: false,
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
    started: bool,
}

impl Iterator for ScrcpyAudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.buffered_samples.pop_front() {
            return Some(sample);
        }

        let closed = self.queue.is_closed();
        if !self.started && !closed && self.queue.sample_count() < self.config.target_samples() {
            self.buffered_samples
                .extend(std::iter::repeat_n(0.0, self.config.local_samples()));
            return self.buffered_samples.pop_front();
        }
        self.started = true;

        self.queue
            .pop_samples(&mut self.buffered_samples, self.config.local_samples());

        if self.buffered_samples.is_empty() && closed {
            return None;
        }

        let read_samples = self.buffered_samples.len();
        if read_samples < self.config.local_samples() {
            let silence_samples = self.config.local_samples() - read_samples;
            self.queue.record_underflow_samples(silence_samples);
            self.buffered_samples
                .extend(std::iter::repeat_n(0.0, silence_samples));
        }
        self.queue.mark_played();
        self.buffered_samples.pop_front()
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
