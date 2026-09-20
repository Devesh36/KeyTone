use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Instant;

use keytone_audio::{output_devices, AudioEngine, AudioStats};
use keytone_core::{AppSettings, Effects, KeyCode, KeyEvent, KeyState, Preset, SettingsEnvelope};
use keytone_input::{permission_instructions, request_permission, InputMonitor, PermissionState};
use keytone_packs::{ensure_demo_packs, load_pack, LoadedPack, PackInfo, PackManifest};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("application data directory is unavailable: {0}")]
    DataDirectory(String),
    #[error("could not create {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("sound pack error: {0}")]
    Pack(#[from] keytone_packs::PackError),
    #[error("audio error: {0}")]
    Audio(#[from] keytone_audio::AudioError),
    #[error("sound pack '{0}' is not installed")]
    MissingPack(String),
    #[error("preset '{0}' was not found")]
    MissingPreset(String),
    #[error("preset name cannot be empty")]
    EmptyPresetName,
    #[error("sound pack '{0}' is already installed")]
    PackAlreadyInstalled(String),
    #[error("import path is not a sound-pack directory or manifest.json")]
    InvalidImportPath,
    #[error("could not serialize settings: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub settings: AppSettings,
    pub presets: Vec<Preset>,
    pub packs: Vec<PackInfo>,
    pub audio: AudioStats,
    pub output_devices: Vec<String>,
    pub permission: &'static str,
    pub permission_instructions: &'static str,
    pub warnings: Vec<String>,
}

pub struct AppRuntime {
    settings: Mutex<SettingsEnvelope>,
    settings_path: PathBuf,
    packs_root: PathBuf,
    packs: Mutex<HashMap<String, (PathBuf, PackInfo)>>,
    audio: Arc<AudioEngine>,
    permission: Arc<AtomicU8>,
    warnings: Arc<Mutex<Vec<String>>>,
    input: InputMonitor,
}

impl AppRuntime {
    pub fn initialize(app: &AppHandle) -> Result<Self, RuntimeError> {
        let data_root = app
            .path()
            .app_data_dir()
            .map_err(|error| RuntimeError::DataDirectory(error.to_string()))?;
        create_dir(&data_root)?;
        let packs_root = data_root.join("packs");
        create_dir(&packs_root)?;
        ensure_demo_packs(&packs_root)?;
        let settings_path = data_root.join("settings.json");
        let mut warnings = Vec::new();
        let mut settings = load_settings(&settings_path, &mut warnings);
        let packs = scan_packs(&packs_root, &mut warnings);
        if !packs.contains_key(&settings.settings.active_pack) {
            warnings.push(format!(
                "Configured pack '{}' is unavailable; using Creamy.",
                settings.settings.active_pack
            ));
            settings.settings.active_pack = "creamy".into();
        }
        let active_pack = load_installed_pack(&packs, &settings.settings.active_pack)?;
        let audio = Arc::new(AudioEngine::new(
            active_pack,
            settings.settings.effects,
            settings.settings.release_sounds,
        ));
        audio.set_enabled(settings.settings.engine_enabled);
        if let Err(error) = audio.start(settings.settings.output_device.as_deref()) {
            warnings.push(format!("Audio output is unavailable: {error}"));
        }
        audio.set_enabled(settings.settings.engine_enabled);

        let initial_permission = keytone_input::permission_state();
        if initial_permission == PermissionState::Missing {
            request_permission();
            warnings.push(permission_instructions().into());
        }
        let permission = Arc::new(AtomicU8::new(permission_code(initial_permission)));
        let warnings = Arc::new(Mutex::new(warnings));
        let input_audio = Arc::clone(&audio);
        let error_permission = Arc::clone(&permission);
        let error_warnings = Arc::clone(&warnings);
        let input = InputMonitor::start(
            settings.settings.play_repeats,
            move |event| input_audio.trigger(event),
            move |error| {
                error_permission
                    .store(permission_code(PermissionState::Missing), Ordering::Release);
                error_warnings.lock().push(format!(
                    "Keyboard monitoring is unavailable: {error}. {}",
                    permission_instructions()
                ));
            },
        );

        let runtime = Self {
            settings: Mutex::new(settings),
            settings_path,
            packs_root,
            packs: Mutex::new(packs),
            audio,
            permission,
            warnings,
            input,
        };
        runtime.persist()?;
        Ok(runtime)
    }

    #[must_use]
    pub fn audio(&self) -> &AudioEngine {
        &self.audio
    }

    #[must_use]
    pub fn snapshot(&self) -> AppSnapshot {
        let envelope = self.settings.lock().clone();
        let mut packs = self
            .packs
            .lock()
            .values()
            .map(|(_, info)| info.clone())
            .collect::<Vec<_>>();
        packs.sort_by(|left, right| left.name.cmp(&right.name));
        AppSnapshot {
            settings: envelope.settings,
            presets: envelope.presets,
            packs,
            audio: self.audio.stats(),
            output_devices: output_devices().unwrap_or_default(),
            permission: permission_label(self.permission.load(Ordering::Acquire)),
            permission_instructions: permission_instructions(),
            warnings: self.warnings.lock().clone(),
        }
    }

    pub fn set_engine(&self, enabled: bool) -> Result<(), RuntimeError> {
        self.audio.set_enabled(enabled);
        self.settings.lock().settings.engine_enabled = enabled;
        self.persist()
    }

    pub fn update_effects(&self, effects: Effects) -> Result<(), RuntimeError> {
        let effects = effects.sanitized();
        self.audio.set_effects(effects);
        self.settings.lock().settings.effects = effects;
        self.persist()
    }

    pub fn update_preferences(
        &self,
        release_sounds: bool,
        play_repeats: bool,
        launch_at_startup: bool,
        keep_running_on_close: bool,
    ) -> Result<(), RuntimeError> {
        self.audio.set_release_sounds(release_sounds);
        self.input.set_play_repeats(play_repeats);
        let mut envelope = self.settings.lock();
        envelope.settings.release_sounds = release_sounds;
        envelope.settings.play_repeats = play_repeats;
        envelope.settings.launch_at_startup = launch_at_startup;
        envelope.settings.keep_running_on_close = keep_running_on_close;
        drop(envelope);
        self.persist()
    }

    pub fn activate_pack(&self, id: &str) -> Result<(), RuntimeError> {
        let pack = load_installed_pack(&self.packs.lock(), id)?;
        self.audio.set_pack(pack);
        let mut envelope = self.settings.lock();
        envelope.settings.active_pack = id.into();
        envelope.settings.active_preset = None;
        drop(envelope);
        self.persist()
    }

    pub fn preview_pack(&self, id: &str) -> Result<(), RuntimeError> {
        let pack = load_installed_pack(&self.packs.lock(), id)?;
        let current_id = self.settings.lock().settings.active_pack.clone();
        let current = load_installed_pack(&self.packs.lock(), &current_id)?;
        self.audio.set_pack(pack);
        self.audio.trigger(KeyEvent {
            key: KeyCode::Space,
            state: KeyState::Pressed,
            timestamp: Instant::now(),
        });
        // The queued trigger owns its selected sample, so restoring immediately is safe.
        self.audio.set_pack(current);
        Ok(())
    }

    pub fn import_pack(&self, input: &str) -> Result<(), RuntimeError> {
        let input = Path::new(input);
        let source = if input.is_dir() {
            input
        } else if input.file_name().and_then(|value| value.to_str()) == Some("manifest.json") {
            input.parent().ok_or(RuntimeError::InvalidImportPath)?
        } else {
            return Err(RuntimeError::InvalidImportPath);
        };
        let pack = load_pack(source)?;
        let id = pack.manifest.id.clone();
        if self.packs.lock().contains_key(&id) {
            return Err(RuntimeError::PackAlreadyInstalled(id));
        }
        let destination = self.packs_root.join(&id);
        copy_pack_files(source, &destination, &pack.manifest)?;
        let installed = load_pack(&destination)?;
        self.packs
            .lock()
            .insert(id, (destination, installed.info()));
        Ok(())
    }

    pub fn save_preset(&self, name: String) -> Result<(), RuntimeError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(RuntimeError::EmptyPresetName);
        }
        let mut envelope = self.settings.lock();
        let preset = Preset::new(
            name,
            envelope.settings.active_pack.clone(),
            envelope.settings.effects,
        );
        if let Some(existing) = envelope.presets.iter_mut().find(|item| item.name == name) {
            *existing = preset;
        } else {
            envelope.presets.push(preset);
        }
        envelope.settings.active_preset = Some(name.into());
        drop(envelope);
        self.persist()
    }

    pub fn load_preset(&self, name: &str) -> Result<(), RuntimeError> {
        let preset = self
            .settings
            .lock()
            .presets
            .iter()
            .find(|preset| preset.name == name)
            .cloned()
            .ok_or_else(|| RuntimeError::MissingPreset(name.into()))?;
        let pack = load_installed_pack(&self.packs.lock(), &preset.pack)?;
        self.audio.set_pack(pack);
        self.audio.set_effects(preset.effects);
        let mut envelope = self.settings.lock();
        envelope.settings.active_pack = preset.pack;
        envelope.settings.effects = preset.effects;
        envelope.settings.active_preset = Some(preset.name);
        drop(envelope);
        self.persist()
    }

    pub fn duplicate_preset(&self, name: &str) -> Result<(), RuntimeError> {
        let preset = self
            .settings
            .lock()
            .presets
            .iter()
            .find(|preset| preset.name == name)
            .cloned()
            .ok_or_else(|| RuntimeError::MissingPreset(name.into()))?;
        let mut envelope = self.settings.lock();
        let base = format!("{} Copy", preset.name);
        let mut candidate = base.clone();
        let mut number = 2;
        while envelope.presets.iter().any(|item| item.name == candidate) {
            candidate = format!("{base} {number}");
            number += 1;
        }
        envelope
            .presets
            .push(Preset::new(candidate, preset.pack, preset.effects));
        drop(envelope);
        self.persist()
    }

    pub fn reset_effects(&self) -> Result<(), RuntimeError> {
        self.update_effects(Effects::default())
    }

    pub fn select_output_device(&self, name: Option<String>) -> Result<(), RuntimeError> {
        self.audio.stop();
        self.audio.start(name.as_deref())?;
        let mut envelope = self.settings.lock();
        envelope.settings.output_device = name;
        self.audio.set_enabled(envelope.settings.engine_enabled);
        drop(envelope);
        self.persist()
    }

    fn persist(&self) -> Result<(), RuntimeError> {
        let payload = serde_json::to_vec_pretty(&*self.settings.lock())?;
        let temporary = self.settings_path.with_extension("json.tmp");
        fs::write(&temporary, payload).map_err(|source| RuntimeError::Write {
            path: temporary.clone(),
            source,
        })?;
        fs::rename(&temporary, &self.settings_path).map_err(|source| RuntimeError::Write {
            path: self.settings_path.clone(),
            source,
        })
    }
}

fn create_dir(path: &Path) -> Result<(), RuntimeError> {
    fs::create_dir_all(path).map_err(|source| RuntimeError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

fn load_settings(path: &Path, warnings: &mut Vec<String>) -> SettingsEnvelope {
    match fs::read_to_string(path) {
        Ok(json) => match serde_json::from_str::<SettingsEnvelope>(&json) {
            Ok(mut settings) if settings.schema_version == 1 => {
                settings.settings.effects = settings.settings.effects.sanitized();
                settings
            }
            Ok(_) => {
                warnings.push("Settings use an unsupported schema; defaults were loaded.".into());
                SettingsEnvelope::default()
            }
            Err(error) => {
                warnings.push(format!(
                    "Settings are invalid; defaults were loaded: {error}"
                ));
                SettingsEnvelope::default()
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => SettingsEnvelope::default(),
        Err(error) => {
            warnings.push(format!(
                "Settings could not be read; defaults were loaded: {error}"
            ));
            SettingsEnvelope::default()
        }
    }
}

fn scan_packs(root: &Path, warnings: &mut Vec<String>) -> HashMap<String, (PathBuf, PackInfo)> {
    let mut packs = HashMap::new();
    let Ok(entries) = fs::read_dir(root) else {
        return packs;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        match load_pack(&entry.path()) {
            Ok(pack) => {
                packs.insert(pack.manifest.id.clone(), (entry.path(), pack.info()));
            }
            Err(error) => warnings.push(format!(
                "Ignored malformed pack at {}: {error}",
                entry.path().display()
            )),
        }
    }
    packs
}

fn load_installed_pack(
    packs: &HashMap<String, (PathBuf, PackInfo)>,
    id: &str,
) -> Result<Arc<LoadedPack>, RuntimeError> {
    let (path, _) = packs
        .get(id)
        .ok_or_else(|| RuntimeError::MissingPack(id.into()))?;
    Ok(Arc::new(load_pack(path)?))
}

fn copy_pack_files(
    source: &Path,
    destination: &Path,
    manifest: &PackManifest,
) -> Result<(), RuntimeError> {
    create_dir(destination)?;
    let paths = manifest
        .samples
        .press
        .values()
        .chain(manifest.samples.release.values())
        .flatten()
        .chain(manifest.preview.iter());
    for relative in paths {
        let from = source.join(relative);
        let to = destination.join(relative);
        if let Some(parent) = to.parent() {
            create_dir(parent)?;
        }
        fs::copy(&from, &to).map_err(|source| RuntimeError::Write { path: to, source })?;
    }
    let manifest_source = source.join("manifest.json");
    let manifest_destination = destination.join("manifest.json");
    fs::copy(&manifest_source, &manifest_destination).map_err(|source| RuntimeError::Write {
        path: manifest_destination,
        source,
    })?;
    Ok(())
}

const fn permission_code(value: PermissionState) -> u8 {
    match value {
        PermissionState::Granted => 1,
        PermissionState::Missing => 2,
        PermissionState::Unknown => 0,
    }
}

const fn permission_label(value: u8) -> &'static str {
    match value {
        1 => "granted",
        2 => "missing",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_settings_fall_back_without_panicking() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("settings.json");
        fs::write(&path, "not-json").expect("write invalid settings");
        let mut warnings = Vec::new();
        let settings = load_settings(&path, &mut warnings);
        assert_eq!(settings, SettingsEnvelope::default());
        assert_eq!(warnings.len(), 1);
    }
}
