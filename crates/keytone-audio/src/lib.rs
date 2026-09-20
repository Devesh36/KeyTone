//! Low-latency CPAL output, polyphonic voice mixing and trigger instrumentation.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig};
use crossbeam_channel::{Receiver, Sender};
use crossbeam_queue::ArrayQueue;
use keytone_core::{Effects, KeyEvent, KeyState};
use keytone_dsp::{pitch_ratio, stereo_pan, AtomicDspParams, DspParams, StereoReverb, ToneFilter};
use keytone_packs::{LoadedPack, SampleData};
use parking_lot::Mutex;
use serde::Serialize;
use thiserror::Error;

pub const MAX_VOICES: usize = 64;
const TRIGGER_QUEUE_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioStatus {
    Stopped,
    Running,
    DeviceError,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStats {
    pub status: AudioStatus,
    pub scheduled_events: u64,
    pub dropped_events: u64,
    pub average_scheduling_micros: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("no audio output device is available")]
    NoOutputDevice,
    #[error("could not enumerate audio devices: {0}")]
    Enumerate(String),
    #[error("output device '{0}' was not found")]
    DeviceNotFound(String),
    #[error("could not query the output format: {0}")]
    OutputConfig(String),
    #[error("unsupported output sample format: {0}")]
    UnsupportedFormat(SampleFormat),
    #[error("could not open the output stream: {0}")]
    BuildStream(String),
    #[error("could not start the output stream: {0}")]
    PlayStream(String),
    #[error("the audio control thread is unavailable")]
    ControlUnavailable,
}

struct Trigger {
    sample: Arc<SampleData>,
    gain: f32,
    rate: f32,
    pan_left: f32,
    pan_right: f32,
}

struct Voice {
    sample: Arc<SampleData>,
    cursor: f32,
    step: f32,
    gain: f32,
    pan_left: f32,
    pan_right: f32,
}

struct Mixer {
    queue: Arc<ArrayQueue<Trigger>>,
    params: Arc<AtomicDspParams>,
    voices: Vec<Option<Voice>>,
    steal_cursor: usize,
    output_rate: f32,
    tone: ToneFilter,
    reverb: StereoReverb,
}

impl Mixer {
    fn new(
        queue: Arc<ArrayQueue<Trigger>>,
        params: Arc<AtomicDspParams>,
        sample_rate: u32,
    ) -> Self {
        let mut voices = Vec::with_capacity(MAX_VOICES);
        voices.resize_with(MAX_VOICES, || None);
        Self {
            queue,
            params,
            voices,
            steal_cursor: 0,
            output_rate: sample_rate as f32,
            tone: ToneFilter::new(sample_rate as f32),
            reverb: StereoReverb::new(sample_rate),
        }
    }

    fn receive_triggers(&mut self) {
        while let Some(trigger) = self.queue.pop() {
            let index = self
                .voices
                .iter()
                .position(Option::is_none)
                .unwrap_or_else(|| {
                    let index = self.steal_cursor;
                    self.steal_cursor = (self.steal_cursor + 1) % MAX_VOICES;
                    index
                });
            self.voices[index] = Some(Voice {
                step: trigger.sample.sample_rate as f32 / self.output_rate * trigger.rate,
                sample: trigger.sample,
                cursor: 0.0,
                gain: trigger.gain,
                pan_left: trigger.pan_left,
                pan_right: trigger.pan_right,
            });
        }
    }

    fn next_frame(&mut self, params: DspParams) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;
        for slot in &mut self.voices {
            let Some(voice) = slot else { continue };
            let index = voice.cursor as usize;
            if index + 1 >= voice.sample.mono.len() {
                *slot = None;
                continue;
            }
            let fraction = voice.cursor - index as f32;
            let first = voice.sample.mono[index];
            let second = voice.sample.mono[index + 1];
            let sample = (first + (second - first) * fraction) * voice.gain;
            left += sample * voice.pan_left;
            right += sample * voice.pan_right;
            voice.cursor += voice.step;
        }
        let (left, right) = self
            .tone
            .process(left, right, params.bass_db, params.treble_db);
        let (left, right) = self.reverb.process(left, right, params.reverb);
        let master = params.master_volume;
        (
            (left * master).clamp(-1.0, 1.0),
            (right * master).clamp(-1.0, 1.0),
        )
    }

    fn render<T>(&mut self, output: &mut [T], channels: usize)
    where
        T: Sample + SizedSample + FromSample<f32>,
    {
        self.receive_triggers();
        let params = self.params.load();
        for frame in output.chunks_mut(channels) {
            let (left, right) = self.next_frame(params);
            if let Some(sample) = frame.first_mut() {
                *sample = T::from_sample(left);
            }
            if let Some(sample) = frame.get_mut(1) {
                *sample = T::from_sample(right);
            }
            for sample in frame.iter_mut().skip(2) {
                *sample = T::from_sample((left + right) * 0.5);
            }
        }
    }
}

pub struct AudioEngine {
    pack: ArcSwap<LoadedPack>,
    params: Arc<AtomicDspParams>,
    queue: Arc<ArrayQueue<Trigger>>,
    control: Sender<AudioCommand>,
    enabled: AtomicBool,
    release_sounds: AtomicBool,
    random_counter: AtomicU64,
    scheduled_events: AtomicU64,
    dropped_events: AtomicU64,
    total_scheduling_nanos: AtomicU64,
    status: Arc<AtomicU8>,
    last_error: Arc<Mutex<Option<String>>>,
}

enum AudioCommand {
    Start {
        selected_device: Option<String>,
        reply: Sender<Result<(), AudioError>>,
    },
    Stop,
    Shutdown,
}

impl AudioEngine {
    #[must_use]
    pub fn new(pack: Arc<LoadedPack>, effects: Effects, release_sounds: bool) -> Self {
        let params = Arc::new(AtomicDspParams::new(DspParams::from(effects)));
        let queue = Arc::new(ArrayQueue::new(TRIGGER_QUEUE_CAPACITY));
        let status = Arc::new(AtomicU8::new(AudioStatus::Stopped as u8));
        let last_error = Arc::new(Mutex::new(None));
        let (control, commands) = crossbeam_channel::unbounded();
        let thread_queue = Arc::clone(&queue);
        let thread_params = Arc::clone(&params);
        let thread_status = Arc::clone(&status);
        let thread_error = Arc::clone(&last_error);
        if let Err(error) = std::thread::Builder::new()
            .name("keytone-audio-control".into())
            .spawn(move || {
                audio_control_loop(
                    commands,
                    thread_queue,
                    thread_params,
                    thread_status,
                    thread_error,
                );
            })
        {
            *last_error.lock() = Some(format!("could not start audio control thread: {error}"));
            status.store(AudioStatus::DeviceError as u8, Ordering::Release);
        }
        Self {
            pack: ArcSwap::new(pack),
            params,
            queue,
            control,
            enabled: AtomicBool::new(true),
            release_sounds: AtomicBool::new(release_sounds),
            random_counter: AtomicU64::new(0x9e37_79b9_7f4a_7c15),
            scheduled_events: AtomicU64::new(0),
            dropped_events: AtomicU64::new(0),
            total_scheduling_nanos: AtomicU64::new(0),
            status,
            last_error,
        }
    }

    pub fn start(&self, selected_device: Option<&str>) -> Result<(), AudioError> {
        let (reply, result) = crossbeam_channel::bounded(1);
        self.control
            .send(AudioCommand::Start {
                selected_device: selected_device.map(str::to_owned),
                reply,
            })
            .map_err(|_| AudioError::ControlUnavailable)?;
        result
            .recv()
            .map_err(|_| AudioError::ControlUnavailable)??;
        self.enabled.store(true, Ordering::Release);
        Ok(())
    }

    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Release);
        let _ = self.control.send(AudioCommand::Stop);
        while self.queue.pop().is_some() {}
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    pub fn set_release_sounds(&self, enabled: bool) {
        self.release_sounds.store(enabled, Ordering::Release);
    }

    pub fn set_effects(&self, effects: Effects) {
        self.params.store(DspParams::from(effects));
    }

    pub fn set_pack(&self, pack: Arc<LoadedPack>) {
        self.pack.store(pack);
    }

    /// Resolves and queues one event. The returned timing covers software scheduling only.
    pub fn trigger(&self, event: KeyEvent) {
        if !self.enabled.load(Ordering::Relaxed)
            || (event.state == KeyState::Released && !self.release_sounds.load(Ordering::Relaxed))
        {
            return;
        }
        let random = splitmix64(self.random_counter.fetch_add(1, Ordering::Relaxed));
        let Some(sample) = self.pack.load().sample_for(event.key, event.state, random) else {
            return;
        };
        let params = self.params.load();
        let pitch_random = signed_unit(random.rotate_left(21)) * params.pitch_randomness;
        let volume_random = signed_unit(random.rotate_left(43)) * params.volume_randomness;
        let (pan_left, pan_right) = stereo_pan(event.key.horizontal_position(), params.spatial);
        let trigger = Trigger {
            sample,
            gain: 1.0 + volume_random,
            rate: pitch_ratio(params.pitch_semitones, pitch_random),
            pan_left,
            pan_right,
        };
        if self.queue.push(trigger).is_ok() {
            let latency = Instant::now().saturating_duration_since(event.timestamp);
            self.scheduled_events.fetch_add(1, Ordering::Relaxed);
            self.total_scheduling_nanos.fetch_add(
                latency.as_nanos().min(u128::from(u64::MAX)) as u64,
                Ordering::Relaxed,
            );
        } else {
            self.dropped_events.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[must_use]
    pub fn stats(&self) -> AudioStats {
        let scheduled = self.scheduled_events.load(Ordering::Relaxed);
        let average_nanos = self.total_scheduling_nanos.load(Ordering::Relaxed) / scheduled.max(1);
        AudioStats {
            status: decode_status(self.status.load(Ordering::Acquire)),
            scheduled_events: scheduled,
            dropped_events: self.dropped_events.load(Ordering::Relaxed),
            average_scheduling_micros: average_nanos / 1_000,
            last_error: self.last_error.lock().clone(),
        }
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        let _ = self.control.send(AudioCommand::Shutdown);
    }
}

fn audio_control_loop(
    commands: Receiver<AudioCommand>,
    queue: Arc<ArrayQueue<Trigger>>,
    params: Arc<AtomicDspParams>,
    status: Arc<AtomicU8>,
    last_error: Arc<Mutex<Option<String>>>,
) {
    let mut stream: Option<Stream> = None;
    while let Ok(command) = commands.recv() {
        match command {
            AudioCommand::Start {
                selected_device,
                reply,
            } => {
                stream.take();
                let result = create_stream(
                    selected_device.as_deref(),
                    Arc::clone(&queue),
                    Arc::clone(&params),
                    Arc::clone(&status),
                    Arc::clone(&last_error),
                );
                match result {
                    Ok(new_stream) => {
                        stream = Some(new_stream);
                        status.store(AudioStatus::Running as u8, Ordering::Release);
                        *last_error.lock() = None;
                        let _ = reply.send(Ok(()));
                    }
                    Err(error) => {
                        status.store(AudioStatus::DeviceError as u8, Ordering::Release);
                        *last_error.lock() = Some(error.to_string());
                        let _ = reply.send(Err(error));
                    }
                }
            }
            AudioCommand::Stop => {
                stream.take();
                status.store(AudioStatus::Stopped as u8, Ordering::Release);
            }
            AudioCommand::Shutdown => break,
        }
    }
}

fn create_stream(
    selected_device: Option<&str>,
    queue: Arc<ArrayQueue<Trigger>>,
    params: Arc<AtomicDspParams>,
    status: Arc<AtomicU8>,
    last_error: Arc<Mutex<Option<String>>>,
) -> Result<Stream, AudioError> {
    let host = cpal::default_host();
    let device = select_device(&host, selected_device)?;
    let supported = device
        .default_output_config()
        .map_err(|error| AudioError::OutputConfig(error.to_string()))?;
    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.into();
    let channels = usize::from(config.channels);
    let sample_rate = config.sample_rate.0;
    let error_callback = move |error: cpal::StreamError| {
        status.store(AudioStatus::DeviceError as u8, Ordering::Release);
        *last_error.lock() = Some(error.to_string());
        tracing::error!(%error, "audio output stream failed");
    };
    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(
            &device,
            &config,
            Mixer::new(queue, params, sample_rate),
            channels,
            error_callback,
        ),
        SampleFormat::I16 => build_stream::<i16>(
            &device,
            &config,
            Mixer::new(queue, params, sample_rate),
            channels,
            error_callback,
        ),
        SampleFormat::U16 => build_stream::<u16>(
            &device,
            &config,
            Mixer::new(queue, params, sample_rate),
            channels,
            error_callback,
        ),
        format => return Err(AudioError::UnsupportedFormat(format)),
    }?;
    stream
        .play()
        .map_err(|error| AudioError::PlayStream(error.to_string()))?;
    Ok(stream)
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    mut mixer: Mixer,
    channels: usize,
    error_callback: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, AudioError>
where
    T: Sample + SizedSample + FromSample<f32>,
{
    device
        .build_output_stream(
            config,
            move |output: &mut [T], _| mixer.render(output, channels),
            error_callback,
            None,
        )
        .map_err(|error| AudioError::BuildStream(error.to_string()))
}

fn select_device(host: &cpal::Host, selected: Option<&str>) -> Result<cpal::Device, AudioError> {
    if let Some(selected) = selected {
        let devices = host
            .output_devices()
            .map_err(|error| AudioError::Enumerate(error.to_string()))?;
        for device in devices {
            if device.name().ok().as_deref() == Some(selected) {
                return Ok(device);
            }
        }
        return Err(AudioError::DeviceNotFound(selected.into()));
    }
    host.default_output_device()
        .ok_or(AudioError::NoOutputDevice)
}

pub fn output_devices() -> Result<Vec<String>, AudioError> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|error| AudioError::Enumerate(error.to_string()))?;
    let mut names = devices
        .filter_map(|device| device.name().ok())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    Ok(names)
}

const fn decode_status(value: u8) -> AudioStatus {
    match value {
        1 => AudioStatus::Running,
        2 => AudioStatus::DeviceError,
        _ => AudioStatus::Stopped,
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn signed_unit(value: u64) -> f32 {
    let normalized = (value >> 40) as f32 / (1_u32 << 24) as f32;
    normalized.mul_add(2.0, -1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_value_is_in_signed_unit_range() {
        for index in 0..10_000 {
            assert!((-1.0..=1.0).contains(&signed_unit(splitmix64(index))));
        }
    }

    #[test]
    fn mixer_has_expected_polyphony() {
        let mut voices = Vec::with_capacity(MAX_VOICES);
        voices.resize_with(MAX_VOICES, || None::<Voice>);
        assert_eq!(voices.len(), 64);
    }
}
