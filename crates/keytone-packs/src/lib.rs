//! Safe parsing, validation and eager sample loading for Keytone Sound Pack v1.

use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use keytone_core::{KeyCategory, KeyCode, KeyState};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub license: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub preview: Option<String>,
    pub samples: ManifestSamples,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSamples {
    pub press: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub release: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub license: String,
    pub tags: Vec<String>,
    pub has_release_samples: bool,
}

#[derive(Debug)]
pub struct SampleData {
    pub mono: Box<[f32]>,
    pub sample_rate: u32,
}

#[derive(Debug, Default)]
struct EventMapping {
    keys: HashMap<KeyCode, Box<[usize]>>,
    categories: HashMap<KeyCategory, Box<[usize]>>,
}

#[derive(Debug)]
pub struct LoadedPack {
    pub manifest: PackManifest,
    pub samples: Box<[Arc<SampleData>]>,
    press: EventMapping,
    release: EventMapping,
}

#[derive(Clone, Copy)]
struct BundledPackFile {
    relative_path: &'static str,
    bytes: &'static [u8],
}

struct BundledPack {
    id: &'static str,
    version: &'static str,
    files: &'static [BundledPackFile],
}

include!(concat!(env!("OUT_DIR"), "/bundled_packs.rs"));

impl LoadedPack {
    #[must_use]
    pub fn info(&self) -> PackInfo {
        PackInfo {
            id: self.manifest.id.clone(),
            name: self.manifest.name.clone(),
            author: self.manifest.author.clone(),
            version: self.manifest.version.clone(),
            description: self.manifest.description.clone(),
            license: self.manifest.license.clone(),
            tags: self.manifest.tags.clone(),
            has_release_samples: !self.manifest.samples.release.is_empty(),
        }
    }

    /// Resolve explicit key -> category -> default without allocating.
    #[must_use]
    pub fn sample_for(
        &self,
        key: KeyCode,
        state: KeyState,
        variation: u64,
    ) -> Option<Arc<SampleData>> {
        let mapping = match state {
            KeyState::Pressed => &self.press,
            KeyState::Released => &self.release,
        };
        let indices = mapping
            .keys
            .get(&key)
            .or_else(|| mapping.categories.get(&key.category()))
            .or_else(|| mapping.categories.get(&KeyCategory::Default))?;
        let index = indices[variation as usize % indices.len()];
        self.samples.get(index).cloned()
    }
}

#[derive(Debug, Error)]
pub enum PackError {
    #[error("could not read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("manifest is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("sample path is unsafe: {0}")]
    UnsafePath(String),
    #[error("could not decode sample {path}: {source}")]
    Wav { path: PathBuf, source: hound::Error },
    #[error("unsupported WAV format in {0}")]
    UnsupportedWav(PathBuf),
}

#[derive(Clone, Copy)]
enum DemoCharacter {
    Creamy,
    Clicky,
    Retro,
    DeepThock,
    Tactile,
    Alloy,
    Spring,
}

#[derive(Clone, Copy)]
enum DemoSampleKind {
    Standard,
    Space,
    Enter,
    Backspace,
    Modifier,
    Arrow,
    Release,
    SpaceRelease,
}

const DEMO_GENERATOR_VERSION: &str = "2";

/// Materializes the CC0 demo packs into a writable application-data directory.
/// All samples are synthesized locally from the formulas below; no recordings are bundled.
pub fn ensure_demo_packs(root: &Path) -> Result<(), PackError> {
    let demos = [
        (
            "creamy",
            "Creamy",
            "Warm, damped linear switches with a soft bottom-out.",
            DemoCharacter::Creamy,
        ),
        (
            "clicky",
            "Clicky",
            "A bright tactile click with a short, crisp decay.",
            DemoCharacter::Clicky,
        ),
        (
            "retro",
            "Retro Terminal",
            "Dry, mid-forward typewriter-inspired impacts.",
            DemoCharacter::Retro,
        ),
        (
            "deep-thock",
            "Deep Thock",
            "Dense low-end impacts with a softly resonant keyboard case.",
            DemoCharacter::DeepThock,
        ),
        (
            "tactile",
            "Tactile Workshop",
            "A defined tactile bump followed by a firm bottom-out.",
            DemoCharacter::Tactile,
        ),
        (
            "alloy",
            "Alloy Linear",
            "Clean linear strikes with crisp aluminium-like resonance.",
            DemoCharacter::Alloy,
        ),
        (
            "spring",
            "Spring Clack",
            "Snappy mechanical impacts with lively spring resonance.",
            DemoCharacter::Spring,
        ),
    ];
    for (id, name, description, character) in demos {
        let directory = root.join(id);
        let version_marker = directory.join(".keytone-demo-version");
        let current_version = fs::read_to_string(&version_marker).unwrap_or_default();
        if directory.join("manifest.json").is_file()
            && current_version.trim() == DEMO_GENERATOR_VERSION
        {
            continue;
        }
        generate_demo_pack(&directory, id, name, description, character)?;
        fs::write(&version_marker, DEMO_GENERATOR_VERSION).map_err(|source| PackError::Write {
            path: version_marker,
            source,
        })?;
    }
    ensure_bundled_packs(root)
}

fn ensure_bundled_packs(root: &Path) -> Result<(), PackError> {
    for pack in BUNDLED_PACKS {
        let directory = root.join(pack.id);
        let version_marker = directory.join(".keytone-bundled-version");
        let current_version = fs::read_to_string(&version_marker).unwrap_or_default();
        if directory.join("manifest.json").is_file() && current_version.trim() == pack.version {
            continue;
        }

        for file in pack.files {
            let path = directory.join(file.relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| PackError::Write {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            fs::write(&path, file.bytes).map_err(|source| PackError::Write { path, source })?;
        }
        fs::write(&version_marker, pack.version).map_err(|source| PackError::Write {
            path: version_marker,
            source,
        })?;
    }
    Ok(())
}

fn generate_demo_pack(
    directory: &Path,
    id: &str,
    name: &str,
    description: &str,
    character: DemoCharacter,
) -> Result<(), PackError> {
    let mappings = [
        ("press/default/01.wav", 0.975, DemoSampleKind::Standard, 1),
        ("press/default/02.wav", 0.995, DemoSampleKind::Standard, 2),
        ("press/default/03.wav", 1.020, DemoSampleKind::Standard, 3),
        ("press/default/04.wav", 1.045, DemoSampleKind::Standard, 4),
        ("press/space/01.wav", 0.985, DemoSampleKind::Space, 5),
        ("press/space/02.wav", 1.020, DemoSampleKind::Space, 6),
        ("press/enter/01.wav", 1.000, DemoSampleKind::Enter, 7),
        (
            "press/backspace/01.wav",
            1.000,
            DemoSampleKind::Backspace,
            8,
        ),
        ("press/modifier/01.wav", 1.000, DemoSampleKind::Modifier, 9),
        ("press/arrow/01.wav", 1.000, DemoSampleKind::Arrow, 10),
        ("release/default/01.wav", 0.985, DemoSampleKind::Release, 11),
        ("release/default/02.wav", 1.025, DemoSampleKind::Release, 12),
        (
            "release/space/01.wav",
            1.000,
            DemoSampleKind::SpaceRelease,
            13,
        ),
    ];
    for (relative, pitch, kind, variation) in mappings {
        let path = directory.join("samples").join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| PackError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        write_synth_wav(&path, character, pitch, kind, variation)?;
    }
    let manifest = serde_json::json!({
        "schemaVersion": 1,
        "id": id,
        "name": name,
        "author": "Keytone",
        "version": "1.1.0",
        "description": description,
        "license": "CC0-1.0",
        "tags": match character {
            DemoCharacter::Creamy => vec!["creamy", "linear"],
            DemoCharacter::Clicky => vec!["bright", "tactile"],
            DemoCharacter::Retro => vec!["retro", "dry"],
            DemoCharacter::DeepThock => vec!["deep", "thock"],
            DemoCharacter::Tactile => vec!["tactile", "firm"],
            DemoCharacter::Alloy => vec!["linear", "alloy"],
            DemoCharacter::Spring => vec!["spring", "resonant"],
        },
        "samples": {
            "press": {
                "default": [
                    "samples/press/default/01.wav",
                    "samples/press/default/02.wav",
                    "samples/press/default/03.wav",
                    "samples/press/default/04.wav"
                ],
                "space": ["samples/press/space/01.wav", "samples/press/space/02.wav"],
                "enter": ["samples/press/enter/01.wav"],
                "backspace": ["samples/press/backspace/01.wav"],
                "modifier": ["samples/press/modifier/01.wav"],
                "arrow": ["samples/press/arrow/01.wav"]
            },
            "release": {
                "default": ["samples/release/default/01.wav", "samples/release/default/02.wav"],
                "space": ["samples/release/space/01.wav"]
            }
        }
    });
    let manifest_path = directory.join("manifest.json");
    let json = serde_json::to_vec_pretty(&manifest)?;
    fs::write(&manifest_path, json).map_err(|source| PackError::Write {
        path: manifest_path,
        source,
    })
}

fn write_synth_wav(
    path: &Path,
    character: DemoCharacter,
    variation_pitch: f32,
    kind: DemoSampleKind,
    variation: u32,
) -> Result<(), PackError> {
    use std::f32::consts::TAU;

    const SAMPLE_RATE: u32 = 48_000;
    let base_duration = match character {
        DemoCharacter::Creamy => 0.110,
        DemoCharacter::Clicky => 0.085,
        DemoCharacter::Retro => 0.105,
        DemoCharacter::DeepThock => 0.145,
        DemoCharacter::Tactile => 0.105,
        DemoCharacter::Alloy => 0.135,
        DemoCharacter::Spring => 0.175,
    };
    let duration = base_duration * kind.duration_scale();
    let frames = (SAMPLE_RATE as f32 * duration) as usize;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let pitch_scale = variation_pitch * kind.pitch_scale();
    let mut noise = character.seed() ^ kind.seed() ^ variation.wrapping_mul(0x9e37_79b9);
    let mut previous_noise = 0.0_f32;
    let mut low_noise = 0.0_f32;
    let mut samples = Vec::with_capacity(frames);
    for frame in 0..frames {
        let time = frame as f32 / SAMPLE_RATE as f32;
        let progress = frame as f32 / frames as f32;
        noise = noise.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let random = ((noise >> 8) as f32 / (1_u32 << 24) as f32).mul_add(2.0, -1.0);
        let high_noise = random - previous_noise;
        previous_noise = random;
        low_noise += (random - low_noise) * 0.16;

        let value = match character {
            DemoCharacter::Creamy => {
                let body = resonator(time, 112.0 * pitch_scale, 30.0, 0.0)
                    + 0.34 * resonator(time, 228.0 * pitch_scale, 46.0, 0.35);
                let contact = low_noise * (-105.0 * time).exp();
                body * 0.66 + contact * 0.32
            }
            DemoCharacter::Clicky => {
                let click = high_noise * (-330.0 * time).exp();
                let jacket = burst(time, 0.0042, 420.0)
                    * (high_noise * 0.65 + (TAU * 2_350.0 * pitch_scale * time).sin() * 0.35);
                let body = resonator(time, 340.0 * pitch_scale, 58.0, 0.2);
                click * 0.52 + jacket * 0.48 + body * 0.24
            }
            DemoCharacter::Retro => {
                let quantized = (random * 6.0).round() / 6.0;
                let clack = quantized * (-92.0 * time).exp();
                let body = resonator(time, 425.0 * pitch_scale, 43.0, 0.0)
                    + 0.31 * resonator(time, 880.0 * pitch_scale, 66.0, 0.8);
                clack * 0.38 + body * 0.56
            }
            DemoCharacter::DeepThock => {
                let body = resonator(time, 82.0 * pitch_scale, 22.0, 0.0)
                    + 0.55 * resonator(time, 164.0 * pitch_scale, 31.0, 0.5)
                    + 0.20 * resonator(time, 252.0 * pitch_scale, 44.0, 1.1);
                let case = low_noise * (-68.0 * time).exp();
                body * 0.72 + case * 0.34
            }
            DemoCharacter::Tactile => {
                let bump = resonator(time, 610.0 * pitch_scale, 105.0, 0.3);
                let bottom_out = burst(time, 0.007, 180.0)
                    * (high_noise * 0.48
                        + resonator(time - 0.007, 185.0 * pitch_scale, 42.0, 0.0) * 0.7);
                let case = resonator(time, 305.0 * pitch_scale, 52.0, 0.7);
                bump * 0.28 + bottom_out * 0.68 + case * 0.24
            }
            DemoCharacter::Alloy => {
                let contact = high_noise * (-215.0 * time).exp();
                let case = resonator(time, 710.0 * pitch_scale, 34.0, 0.0)
                    + 0.34 * resonator(time, 1_510.0 * pitch_scale, 44.0, 0.7)
                    + 0.13 * resonator(time, 3_120.0 * pitch_scale, 63.0, 1.4);
                contact * 0.42 + case * 0.62
            }
            DemoCharacter::Spring => {
                let snap = high_noise * (-270.0 * time).exp();
                let spring = resonator(time, 1_330.0 * pitch_scale, 20.0, 0.0)
                    + 0.72 * resonator(time, 1_415.0 * pitch_scale, 23.0, 0.8)
                    + 0.18 * resonator(time, 2_760.0 * pitch_scale, 35.0, 1.5);
                let body = resonator(time, 265.0 * pitch_scale, 43.0, 0.4);
                snap * 0.44 + spring * 0.34 + body * 0.38
            }
        };

        let release_shape = if kind.is_release() {
            0.76 + 0.24 * (-95.0 * time).exp()
        } else {
            1.0
        };
        let tail_fade = ((1.0 - progress) / 0.16).clamp(0.0, 1.0);
        samples.push((value * kind.body_scale() * release_shape).tanh() * tail_fade);
    }

    let peak = samples.iter().copied().map(f32::abs).fold(0.0, f32::max);
    let gain = kind.target_peak() / peak.max(0.001);
    let mut writer = hound::WavWriter::create(path, spec).map_err(|source| PackError::Wav {
        path: path.to_path_buf(),
        source,
    })?;
    for value in samples {
        let sample = (value * gain * f32::from(i16::MAX))
            .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16;
        writer
            .write_sample(sample)
            .map_err(|source| PackError::Wav {
                path: path.to_path_buf(),
                source,
            })?;
    }
    writer.finalize().map_err(|source| PackError::Wav {
        path: path.to_path_buf(),
        source,
    })
}

fn resonator(time: f32, frequency: f32, decay: f32, phase: f32) -> f32 {
    use std::f32::consts::TAU;

    if time < 0.0 {
        0.0
    } else {
        (TAU * frequency * time + phase).sin() * (-decay * time).exp()
    }
}

fn burst(time: f32, start: f32, decay: f32) -> f32 {
    if time < start {
        0.0
    } else {
        (-decay * (time - start)).exp()
    }
}

impl DemoCharacter {
    const fn seed(self) -> u32 {
        match self {
            Self::Creamy => 0x1357_2468,
            Self::Clicky => 0x2468_1357,
            Self::Retro => 0x8088_6502,
            Self::DeepThock => 0x0dee_7001,
            Self::Tactile => 0x7ac7_11e0,
            Self::Alloy => 0xa110_9001,
            Self::Spring => 0x5a71_1a6c,
        }
    }
}

impl DemoSampleKind {
    const fn is_release(self) -> bool {
        matches!(self, Self::Release | Self::SpaceRelease)
    }

    const fn duration_scale(self) -> f32 {
        match self {
            Self::Space => 1.24,
            Self::Enter => 1.08,
            Self::Backspace => 0.92,
            Self::Modifier => 0.82,
            Self::Arrow => 0.72,
            Self::Release => 0.48,
            Self::SpaceRelease => 0.64,
            Self::Standard => 1.0,
        }
    }

    const fn pitch_scale(self) -> f32 {
        match self {
            Self::Space => 0.68,
            Self::Enter => 0.82,
            Self::Backspace => 1.12,
            Self::Modifier => 0.90,
            Self::Arrow => 1.18,
            Self::Release => 1.30,
            Self::SpaceRelease => 0.94,
            Self::Standard => 1.0,
        }
    }

    const fn body_scale(self) -> f32 {
        match self {
            Self::Space => 1.20,
            Self::Enter => 1.08,
            Self::Backspace => 0.92,
            Self::Modifier => 0.76,
            Self::Arrow => 0.68,
            Self::Release => 0.62,
            Self::SpaceRelease => 0.72,
            Self::Standard => 1.0,
        }
    }

    const fn target_peak(self) -> f32 {
        match self {
            Self::Space => 0.88,
            Self::Enter => 0.84,
            Self::Backspace => 0.78,
            Self::Modifier => 0.70,
            Self::Arrow => 0.64,
            Self::Release => 0.56,
            Self::SpaceRelease => 0.62,
            Self::Standard => 0.82,
        }
    }

    const fn seed(self) -> u32 {
        match self {
            Self::Standard => 0x1010_0101,
            Self::Space => 0x2020_0202,
            Self::Enter => 0x3030_0303,
            Self::Backspace => 0x4040_0404,
            Self::Modifier => 0x5050_0505,
            Self::Arrow => 0x6060_0606,
            Self::Release => 0x7070_0707,
            Self::SpaceRelease => 0x8080_0808,
        }
    }
}

pub fn parse_manifest(json: &str) -> Result<PackManifest, PackError> {
    let manifest: PackManifest = serde_json::from_str(json)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest(manifest: &PackManifest) -> Result<(), PackError> {
    if manifest.schema_version != 1 {
        return Err(PackError::InvalidManifest(format!(
            "unsupported schemaVersion {}; expected 1",
            manifest.schema_version
        )));
    }
    if manifest.id.is_empty()
        || !manifest
            .id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(PackError::InvalidManifest(
            "id must contain only letters, numbers, '-' or '_'".into(),
        ));
    }
    for (label, value) in [
        ("name", &manifest.name),
        ("author", &manifest.author),
        ("version", &manifest.version),
        ("license", &manifest.license),
    ] {
        if value.trim().is_empty() {
            return Err(PackError::InvalidManifest(format!(
                "{label} cannot be empty"
            )));
        }
    }
    if manifest.samples.press.is_empty() {
        return Err(PackError::InvalidManifest(
            "samples.press must contain at least one mapping".into(),
        ));
    }
    for (mapping_name, paths) in manifest
        .samples
        .press
        .iter()
        .chain(manifest.samples.release.iter())
    {
        validate_mapping_name(mapping_name)?;
        if paths.is_empty() {
            return Err(PackError::InvalidManifest(format!(
                "mapping '{mapping_name}' has no samples"
            )));
        }
        for path in paths {
            validate_relative_path(path)?;
        }
    }
    if let Some(preview) = &manifest.preview {
        validate_relative_path(preview)?;
    }
    Ok(())
}

fn validate_mapping_name(name: &str) -> Result<(), PackError> {
    if parse_category(name).is_some() {
        return Ok(());
    }
    if let Some(key_name) = name.strip_prefix("key:") {
        let encoded = format!("\"{key_name}\"");
        let key: Result<KeyCode, _> = serde_json::from_str(&encoded);
        if key.is_ok() {
            return Ok(());
        }
    }
    Err(PackError::InvalidManifest(format!(
        "unknown mapping '{name}'; use a category or key:<key-code>"
    )))
}

fn validate_relative_path(value: &str) -> Result<(), PackError> {
    let path = Path::new(value);
    let safe = !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir));
    if !safe || path.extension().and_then(|value| value.to_str()) != Some("wav") {
        return Err(PackError::UnsafePath(value.into()));
    }
    Ok(())
}

pub fn load_pack(directory: &Path) -> Result<LoadedPack, PackError> {
    let manifest_path = directory.join("manifest.json");
    let json = fs::read_to_string(&manifest_path).map_err(|source| PackError::Read {
        path: manifest_path,
        source,
    })?;
    let manifest = parse_manifest(&json)?;
    load_validated_pack(directory, manifest)
}

fn load_validated_pack(directory: &Path, manifest: PackManifest) -> Result<LoadedPack, PackError> {
    let mut samples = Vec::<Arc<SampleData>>::new();
    let press = load_mapping(directory, &manifest.samples.press, &mut samples)?;
    let release = load_mapping(directory, &manifest.samples.release, &mut samples)?;
    Ok(LoadedPack {
        manifest,
        samples: samples.into_boxed_slice(),
        press,
        release,
    })
}

fn load_mapping(
    directory: &Path,
    entries: &HashMap<String, Vec<String>>,
    samples: &mut Vec<Arc<SampleData>>,
) -> Result<EventMapping, PackError> {
    let mut mapping = EventMapping::default();
    for (name, paths) in entries {
        let mut indices = Vec::with_capacity(paths.len());
        for relative_path in paths {
            let sample = Arc::new(decode_wav(&directory.join(relative_path))?);
            indices.push(samples.len());
            samples.push(sample);
        }
        if let Some(category) = parse_category(name) {
            mapping
                .categories
                .insert(category, indices.into_boxed_slice());
        } else if let Some(key_name) = name.strip_prefix("key:") {
            let encoded = format!("\"{key_name}\"");
            let key: KeyCode = serde_json::from_str(&encoded)
                .map_err(|_| PackError::InvalidManifest(format!("invalid key mapping '{name}'")))?;
            mapping.keys.insert(key, indices.into_boxed_slice());
        }
    }
    Ok(mapping)
}

fn parse_category(name: &str) -> Option<KeyCategory> {
    match name {
        "default" => Some(KeyCategory::Default),
        "alpha" => Some(KeyCategory::Alpha),
        "number" => Some(KeyCategory::Number),
        "space" => Some(KeyCategory::Space),
        "enter" => Some(KeyCategory::Enter),
        "backspace" => Some(KeyCategory::Backspace),
        "modifier" => Some(KeyCategory::Modifier),
        "arrow" => Some(KeyCategory::Arrow),
        "function" => Some(KeyCategory::Function),
        _ => None,
    }
}

fn decode_wav(path: &Path) -> Result<SampleData, PackError> {
    let mut reader = hound::WavReader::open(path).map_err(|source| PackError::Wav {
        path: path.to_path_buf(),
        source,
    })?;
    let spec = reader.spec();
    if spec.channels == 0 || spec.channels > 8 || spec.sample_rate < 8_000 {
        return Err(PackError::UnsupportedWav(path.to_path_buf()));
    }
    let channels = usize::from(spec.channels);
    let interleaved = match spec.sample_format {
        hound::SampleFormat::Float if spec.bits_per_sample == 32 => reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| PackError::Wav {
                path: path.to_path_buf(),
                source,
            })?,
        hound::SampleFormat::Int if spec.bits_per_sample <= 16 => {
            let scale = f32::from(i16::MAX);
            reader
                .samples::<i16>()
                .map(|sample| sample.map(|value| f32::from(value) / scale))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|source| PackError::Wav {
                    path: path.to_path_buf(),
                    source,
                })?
        }
        hound::SampleFormat::Int if spec.bits_per_sample <= 32 => {
            let scale = ((1_i64 << (spec.bits_per_sample - 1)) - 1) as f32;
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|source| PackError::Wav {
                    path: path.to_path_buf(),
                    source,
                })?
        }
        _ => return Err(PackError::UnsupportedWav(path.to_path_buf())),
    };
    let mono = interleaved
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    if mono.is_empty() {
        return Err(PackError::UnsupportedWav(path.to_path_buf()));
    }
    Ok(SampleData {
        mono,
        sample_rate: spec.sample_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"{
      "schemaVersion": 1,
      "id": "test-pack",
      "name": "Test Pack",
      "author": "Keytone",
      "version": "1.0.0",
      "description": "test",
      "license": "CC0-1.0",
      "samples": {
        "press": {"default": ["samples/press/default/01.wav"], "key:space": ["samples/press/space/01.wav"]},
        "release": {"default": ["samples/release/default/01.wav"]}
      }
    }"#;

    #[test]
    fn parses_valid_manifest() {
        let manifest = parse_manifest(VALID).expect("valid manifest");
        assert_eq!(manifest.id, "test-pack");
    }

    #[test]
    fn rejects_bad_schema_and_traversal() {
        let bad_schema = VALID.replace("\"schemaVersion\": 1", "\"schemaVersion\": 99");
        assert!(parse_manifest(&bad_schema).is_err());
        let traversal = VALID.replace("samples/press/default/01.wav", "../outside.wav");
        assert!(matches!(
            parse_manifest(&traversal),
            Err(PackError::UnsafePath(_))
        ));
    }

    #[test]
    fn rejects_unknown_mapping() {
        let bad = VALID.replace("\"default\":", "\"totally-unknown\":");
        assert!(parse_manifest(&bad).is_err());
    }

    #[test]
    fn selection_prefers_key_then_category_then_default() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sample_paths = [
            "samples/press/default/01.wav",
            "samples/press/space/01.wav",
            "samples/release/default/01.wav",
        ];
        for (index, relative) in sample_paths.iter().enumerate() {
            let path = temp.path().join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("sample directory");
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: 48_000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::create(path, spec).expect("wav create");
            writer
                .write_sample((index as i16 + 1) * 100)
                .expect("sample");
            writer.finalize().expect("wav finalize");
        }
        fs::write(temp.path().join("manifest.json"), VALID).expect("manifest");
        let pack = load_pack(temp.path()).expect("load pack");
        let alpha = pack
            .sample_for(KeyCode::A, KeyState::Pressed, 0)
            .expect("default sample");
        let space = pack
            .sample_for(KeyCode::Space, KeyState::Pressed, 0)
            .expect("space sample");
        assert_ne!(alpha.mono[0], space.mono[0]);
        assert!(pack.sample_for(KeyCode::A, KeyState::Released, 0).is_some());
    }

    #[test]
    fn installs_generated_and_mit_licensed_builtin_packs() {
        let temp = tempfile::tempdir().expect("tempdir");
        ensure_demo_packs(temp.path()).expect("generate demo packs");

        let entries = fs::read_dir(temp.path())
            .expect("read packs")
            .collect::<Result<Vec<_>, _>>()
            .expect("pack entries");
        assert_eq!(entries.len(), 19);

        let thock = load_pack(&temp.path().join("deep-thock")).expect("load deep thock");
        assert_eq!(thock.manifest.version, "1.1.0");
        assert_eq!(thock.manifest.samples.press["default"].len(), 4);
        assert_eq!(thock.manifest.samples.press["space"].len(), 2);
        assert_eq!(thock.manifest.samples.release["default"].len(), 2);
        assert!(thock.samples.len() >= 13);
        assert_eq!(
            fs::read_to_string(temp.path().join("deep-thock/.keytone-demo-version"))
                .expect("version marker"),
            DEMO_GENERATOR_VERSION
        );

        assert_eq!(BUNDLED_PACKS.len(), 12);
        for bundled in BUNDLED_PACKS {
            let pack = load_pack(&temp.path().join(bundled.id)).expect("load bundled pack");
            assert_eq!(pack.manifest.license, "MIT");
            assert_eq!(pack.manifest.samples.press["default"].len(), 5);
            assert!(temp.path().join(bundled.id).join("LICENSE.txt").is_file());
        }

        let alpaca = load_pack(&temp.path().join("kbsim-alpaca")).expect("load Alpaca pack");
        assert_eq!(alpaca.manifest.license, "MIT");
        assert!(alpaca.manifest.samples.press.contains_key("space"));
        assert!(alpaca.manifest.samples.release.contains_key("space"));
        assert_eq!(
            fs::read_to_string(temp.path().join("kbsim-alpaca/.keytone-bundled-version"))
                .expect("bundled version marker"),
            "1"
        );
    }
}
