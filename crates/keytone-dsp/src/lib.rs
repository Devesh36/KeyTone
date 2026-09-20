//! Allocation-free audio processing primitives for Keytone's callback.

use std::f32::consts::{FRAC_PI_4, PI};
use std::sync::atomic::{AtomicU32, Ordering};

use keytone_core::Effects;

#[derive(Debug, Clone, Copy)]
pub struct DspParams {
    pub master_volume: f32,
    pub pitch_semitones: f32,
    pub pitch_randomness: f32,
    pub volume_randomness: f32,
    pub bass_db: f32,
    pub treble_db: f32,
    pub reverb: f32,
    pub spatial: f32,
}

impl From<Effects> for DspParams {
    fn from(value: Effects) -> Self {
        let value = value.sanitized();
        Self {
            master_volume: value.master_volume,
            pitch_semitones: value.pitch,
            pitch_randomness: value.pitch_randomness,
            volume_randomness: value.volume_randomness,
            bass_db: value.bass,
            treble_db: value.treble,
            reverb: value.reverb,
            spatial: value.spatial,
        }
    }
}

/// Effect controls shared with the audio callback via relaxed atomic loads.
pub struct AtomicDspParams {
    master_volume: AtomicU32,
    pitch_semitones: AtomicU32,
    pitch_randomness: AtomicU32,
    volume_randomness: AtomicU32,
    bass_db: AtomicU32,
    treble_db: AtomicU32,
    reverb: AtomicU32,
    spatial: AtomicU32,
}

impl AtomicDspParams {
    #[must_use]
    pub fn new(params: DspParams) -> Self {
        Self {
            master_volume: AtomicU32::new(params.master_volume.to_bits()),
            pitch_semitones: AtomicU32::new(params.pitch_semitones.to_bits()),
            pitch_randomness: AtomicU32::new(params.pitch_randomness.to_bits()),
            volume_randomness: AtomicU32::new(params.volume_randomness.to_bits()),
            bass_db: AtomicU32::new(params.bass_db.to_bits()),
            treble_db: AtomicU32::new(params.treble_db.to_bits()),
            reverb: AtomicU32::new(params.reverb.to_bits()),
            spatial: AtomicU32::new(params.spatial.to_bits()),
        }
    }

    pub fn store(&self, params: DspParams) {
        self.master_volume
            .store(params.master_volume.to_bits(), Ordering::Relaxed);
        self.pitch_semitones
            .store(params.pitch_semitones.to_bits(), Ordering::Relaxed);
        self.pitch_randomness
            .store(params.pitch_randomness.to_bits(), Ordering::Relaxed);
        self.volume_randomness
            .store(params.volume_randomness.to_bits(), Ordering::Relaxed);
        self.bass_db
            .store(params.bass_db.to_bits(), Ordering::Relaxed);
        self.treble_db
            .store(params.treble_db.to_bits(), Ordering::Relaxed);
        self.reverb
            .store(params.reverb.to_bits(), Ordering::Relaxed);
        self.spatial
            .store(params.spatial.to_bits(), Ordering::Relaxed);
    }

    #[must_use]
    pub fn load(&self) -> DspParams {
        DspParams {
            master_volume: f32::from_bits(self.master_volume.load(Ordering::Relaxed)),
            pitch_semitones: f32::from_bits(self.pitch_semitones.load(Ordering::Relaxed)),
            pitch_randomness: f32::from_bits(self.pitch_randomness.load(Ordering::Relaxed)),
            volume_randomness: f32::from_bits(self.volume_randomness.load(Ordering::Relaxed)),
            bass_db: f32::from_bits(self.bass_db.load(Ordering::Relaxed)),
            treble_db: f32::from_bits(self.treble_db.load(Ordering::Relaxed)),
            reverb: f32::from_bits(self.reverb.load(Ordering::Relaxed)),
            spatial: f32::from_bits(self.spatial.load(Ordering::Relaxed)),
        }
    }
}

/// Equal-power stereo pan with a deliberately narrowed maximum field.
#[must_use]
pub fn stereo_pan(position: f32, spatial_strength: f32) -> (f32, f32) {
    let pan = position.clamp(-1.0, 1.0) * spatial_strength.clamp(0.0, 1.0) * 0.85;
    let angle = (pan + 1.0) * FRAC_PI_4;
    (angle.cos(), angle.sin())
}

#[must_use]
pub fn pitch_ratio(semitones: f32, random_fraction: f32) -> f32 {
    2.0_f32.powf(semitones.clamp(-12.0, 12.0) / 12.0) * (1.0 + random_fraction.clamp(-0.12, 0.12))
}

/// Lightweight tone controls: split the signal with one-pole filters and gain the bands.
pub struct ToneFilter {
    low_l: f32,
    low_r: f32,
    high_lp_l: f32,
    high_lp_r: f32,
    low_coefficient: f32,
    high_coefficient: f32,
}

impl ToneFilter {
    #[must_use]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            low_l: 0.0,
            low_r: 0.0,
            high_lp_l: 0.0,
            high_lp_r: 0.0,
            low_coefficient: one_pole_coefficient(220.0, sample_rate),
            high_coefficient: one_pole_coefficient(3_800.0, sample_rate),
        }
    }

    #[must_use]
    pub fn process(&mut self, left: f32, right: f32, bass_db: f32, treble_db: f32) -> (f32, f32) {
        self.low_l += self.low_coefficient * (left - self.low_l);
        self.low_r += self.low_coefficient * (right - self.low_r);
        self.high_lp_l += self.high_coefficient * (left - self.high_lp_l);
        self.high_lp_r += self.high_coefficient * (right - self.high_lp_r);

        let bass_gain = db_to_gain(bass_db.clamp(-12.0, 12.0)) - 1.0;
        let treble_gain = db_to_gain(treble_db.clamp(-12.0, 12.0)) - 1.0;
        (
            left + self.low_l * bass_gain + (left - self.high_lp_l) * treble_gain,
            right + self.low_r * bass_gain + (right - self.high_lp_r) * treble_gain,
        )
    }
}

fn one_pole_coefficient(cutoff: f32, sample_rate: f32) -> f32 {
    1.0 - (-2.0 * PI * cutoff / sample_rate.max(8_000.0)).exp()
}

fn db_to_gain(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// A compact feedback-delay reverb. Buffers are allocated before the stream starts.
pub struct StereoReverb {
    left: Vec<f32>,
    right: Vec<f32>,
    cursor: usize,
}

impl StereoReverb {
    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        let frames = ((sample_rate as f32 * 0.037).round() as usize).max(64);
        Self {
            left: vec![0.0; frames],
            right: vec![0.0; frames + 113],
            cursor: 0,
        }
    }

    #[must_use]
    pub fn process(&mut self, left: f32, right: f32, amount: f32) -> (f32, f32) {
        let amount = amount.clamp(0.0, 0.5);
        let left_index = self.cursor % self.left.len();
        let right_index = self.cursor % self.right.len();
        let wet_l = self.left[left_index];
        let wet_r = self.right[right_index];
        self.left[left_index] = left + wet_r * 0.48;
        self.right[right_index] = right + wet_l * 0.45;
        self.cursor = self.cursor.wrapping_add(1);
        (
            left * (1.0 - amount) + wet_l * amount,
            right * (1.0 - amount) + wet_r * amount,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_is_centered_at_zero_and_bounded() {
        let center = stereo_pan(0.0, 1.0);
        assert!((center.0 - center.1).abs() < 0.000_1);
        let left = stereo_pan(-1.0, 1.0);
        assert!(left.0 > left.1);
        assert!(left.1 > 0.0, "normal keys are never hard-panned");
        let disabled = stereo_pan(-1.0, 0.0);
        assert!((disabled.0 - disabled.1).abs() < 0.000_1);
    }

    #[test]
    fn pitch_is_bounded() {
        assert!((pitch_ratio(12.0, 0.0) - 2.0).abs() < 0.000_1);
        assert!(pitch_ratio(0.0, 10.0) <= 1.12);
    }

    #[test]
    fn atomic_params_round_trip() {
        let expected = DspParams::from(Effects::default());
        let params = AtomicDspParams::new(expected);
        assert_eq!(params.load().master_volume, expected.master_volume);
    }
}
