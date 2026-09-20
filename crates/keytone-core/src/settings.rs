use serde::{Deserialize, Serialize};

use crate::SETTINGS_SCHEMA_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Effects {
    pub master_volume: f32,
    /// Fixed pitch offset in semitones.
    pub pitch: f32,
    /// Random pitch range as a fractional ratio (0.02 = +/-2%).
    pub pitch_randomness: f32,
    /// Random gain range as a fractional ratio.
    pub volume_randomness: f32,
    /// Low shelf gain in dB.
    pub bass: f32,
    /// High shelf gain in dB.
    pub treble: f32,
    pub reverb: f32,
    pub spatial: f32,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            master_volume: 0.82,
            pitch: 0.0,
            pitch_randomness: 0.02,
            volume_randomness: 0.03,
            bass: 2.0,
            treble: 0.0,
            reverb: 0.08,
            spatial: 0.72,
        }
    }
}

impl Effects {
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        self.master_volume = finite_clamp(self.master_volume, 0.0, 1.0, 0.82);
        self.pitch = finite_clamp(self.pitch, -12.0, 12.0, 0.0);
        self.pitch_randomness = finite_clamp(self.pitch_randomness, 0.0, 0.12, 0.02);
        self.volume_randomness = finite_clamp(self.volume_randomness, 0.0, 0.2, 0.03);
        self.bass = finite_clamp(self.bass, -12.0, 12.0, 0.0);
        self.treble = finite_clamp(self.treble, -12.0, 12.0, 0.0);
        self.reverb = finite_clamp(self.reverb, 0.0, 0.5, 0.08);
        self.spatial = finite_clamp(self.spatial, 0.0, 1.0, 0.72);
        self
    }
}

fn finite_clamp(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub schema_version: u32,
    pub name: String,
    pub pack: String,
    pub effects: Effects,
}

impl Preset {
    #[must_use]
    pub fn new(name: impl Into<String>, pack: impl Into<String>, effects: Effects) -> Self {
        Self {
            schema_version: 1,
            name: name.into(),
            pack: pack.into(),
            effects: effects.sanitized(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[allow(clippy::struct_excessive_bools)] // Independent persisted user toggles, not state flags.
pub struct AppSettings {
    pub engine_enabled: bool,
    pub active_pack: String,
    pub active_preset: Option<String>,
    pub effects: Effects,
    pub release_sounds: bool,
    pub play_repeats: bool,
    pub launch_at_startup: bool,
    pub keep_running_on_close: bool,
    pub output_device: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            engine_enabled: true,
            active_pack: "creamy".into(),
            active_preset: Some("Late Night Cream".into()),
            effects: Effects::default(),
            release_sounds: true,
            play_repeats: false,
            launch_at_startup: false,
            keep_running_on_close: true,
            output_device: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsEnvelope {
    pub schema_version: u32,
    pub settings: AppSettings,
    pub presets: Vec<Preset>,
}

impl Default for SettingsEnvelope {
    fn default() -> Self {
        let settings = AppSettings::default();
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            presets: vec![Preset::new(
                "Late Night Cream",
                settings.active_pack.clone(),
                settings.effects,
            )],
            settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_are_bounded_and_nan_safe() {
        let result = Effects {
            master_volume: 9.0,
            pitch: f32::NAN,
            reverb: -1.0,
            spatial: 8.0,
            ..Effects::default()
        }
        .sanitized();
        assert!((result.master_volume - 1.0).abs() < f32::EPSILON);
        assert!(result.pitch.abs() < f32::EPSILON);
        assert!(result.reverb.abs() < f32::EPSILON);
        assert!((result.spatial - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn settings_and_presets_round_trip() {
        let original = SettingsEnvelope::default();
        let json = serde_json::to_string(&original).expect("settings serialize");
        let decoded: SettingsEnvelope = serde_json::from_str(&json).expect("settings deserialize");
        assert_eq!(decoded, original);
        assert_eq!(decoded.presets[0].schema_version, 1);
    }

    #[test]
    fn missing_new_fields_receive_defaults() {
        let decoded: AppSettings = serde_json::from_str("{}").expect("default settings");
        assert_eq!(decoded, AppSettings::default());
    }
}
