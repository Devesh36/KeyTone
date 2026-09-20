//! Platform-neutral domain types used by every Keytone component.

mod key;
mod settings;

pub use key::{KeyCategory, KeyCode, KeyEvent, KeyState};
pub use settings::{AppSettings, Effects, Preset, SettingsEnvelope};

/// Configuration format version supported by this build.
pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
