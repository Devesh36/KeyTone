//! Cross-platform global keyboard capture and privacy-preserving key normalization.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use keytone_core::{KeyCode, KeyEvent};
use rdev::Key;
use thiserror::Error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
use keytone_core::KeyState;
#[cfg(not(target_os = "macos"))]
use rdev::EventType;
#[cfg(not(target_os = "macos"))]
use std::collections::HashSet;
#[cfg(not(target_os = "macos"))]
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    Granted,
    Missing,
    Unknown,
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error("global keyboard listener failed: {0}")]
    Listener(String),
}

/// Backend interface keeps OS capture replaceable without leaking native key types.
pub trait InputBackend: Send + 'static {
    fn run(
        self,
        active: Arc<AtomicBool>,
        play_repeats: Arc<AtomicBool>,
        emit: impl Fn(KeyEvent) + Send + 'static,
    ) -> Result<(), InputError>;
}

#[derive(Default)]
pub struct NativeInputBackend;

impl InputBackend for NativeInputBackend {
    fn run(
        self,
        active: Arc<AtomicBool>,
        play_repeats: Arc<AtomicBool>,
        emit: impl Fn(KeyEvent) + Send + 'static,
    ) -> Result<(), InputError> {
        #[cfg(target_os = "macos")]
        {
            macos::listen(active, play_repeats, emit)
        }

        #[cfg(not(target_os = "macos"))]
        {
            let mut pressed = HashSet::new();
            rdev::listen(move |event| {
                if !active.load(Ordering::Relaxed) {
                    return;
                }
                let normalized = match event.event_type {
                    EventType::KeyPress(native_key) => {
                        let key = normalize_key(native_key);
                        let is_repeat = !pressed.insert(key);
                        if key == KeyCode::Unknown
                            || (is_repeat && !play_repeats.load(Ordering::Relaxed))
                        {
                            return;
                        }
                        Some(KeyEvent {
                            key,
                            state: KeyState::Pressed,
                            timestamp: Instant::now(),
                        })
                    }
                    EventType::KeyRelease(native_key) => {
                        let key = normalize_key(native_key);
                        if key == KeyCode::Unknown {
                            return;
                        }
                        pressed.remove(&key);
                        Some(KeyEvent {
                            key,
                            state: KeyState::Released,
                            timestamp: Instant::now(),
                        })
                    }
                    _ => None,
                };
                if let Some(event) = normalized {
                    emit(event);
                }
            })
            .map_err(|error| InputError::Listener(format!("{error:?}")))
        }
    }
}

pub struct InputMonitor {
    active: Arc<AtomicBool>,
    play_repeats: Arc<AtomicBool>,
}

impl InputMonitor {
    /// Starts one process-lifetime listener. Stopping disables delivery immediately.
    #[must_use]
    pub fn start(
        play_repeats: bool,
        emit: impl Fn(KeyEvent) + Send + 'static,
        on_error: impl Fn(InputError) + Send + 'static,
    ) -> Self {
        let active = Arc::new(AtomicBool::new(true));
        let play_repeats = Arc::new(AtomicBool::new(play_repeats));
        let thread_active = Arc::clone(&active);
        let thread_repeats = Arc::clone(&play_repeats);
        thread::Builder::new()
            .name("keytone-global-input".into())
            .spawn(move || {
                if let Err(error) = NativeInputBackend.run(thread_active, thread_repeats, emit) {
                    on_error(error);
                }
            })
            .ok();
        Self {
            active,
            play_repeats,
        }
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Release);
    }

    pub fn set_play_repeats(&self, play_repeats: bool) {
        self.play_repeats.store(play_repeats, Ordering::Release);
    }
}

impl Drop for InputMonitor {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

#[must_use]
pub fn permission_state() -> PermissionState {
    #[cfg(target_os = "macos")]
    {
        if objc2_core_graphics::CGPreflightListenEventAccess() {
            PermissionState::Granted
        } else {
            PermissionState::Missing
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Other native backends report permission failure when listener setup fails.
        PermissionState::Unknown
    }
}

/// Ask the host OS to show its keyboard-monitoring permission prompt, where supported.
pub fn request_permission() {
    #[cfg(target_os = "macos")]
    {
        let _ = objc2_core_graphics::CGRequestListenEventAccess();
    }
}

#[must_use]
pub const fn permission_instructions() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Open System Settings → Privacy & Security → Input Monitoring, enable Keytone, then restart Keytone."
    }
    #[cfg(target_os = "windows")]
    {
        "Keytone could not install its global keyboard hook. Restart it normally; elevated applications may require matching privileges."
    }
    #[cfg(target_os = "linux")]
    {
        "Grant the current user permission to read input devices (usually the input group), then sign out and back in. Wayland compositor policy may also apply."
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "Global keyboard capture is not supported on this operating system."
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub const fn normalize_key(key: Key) -> KeyCode {
    match key {
        Key::KeyA => KeyCode::A,
        Key::KeyB => KeyCode::B,
        Key::KeyC => KeyCode::C,
        Key::KeyD => KeyCode::D,
        Key::KeyE => KeyCode::E,
        Key::KeyF => KeyCode::F,
        Key::KeyG => KeyCode::G,
        Key::KeyH => KeyCode::H,
        Key::KeyI => KeyCode::I,
        Key::KeyJ => KeyCode::J,
        Key::KeyK => KeyCode::K,
        Key::KeyL => KeyCode::L,
        Key::KeyM => KeyCode::M,
        Key::KeyN => KeyCode::N,
        Key::KeyO => KeyCode::O,
        Key::KeyP => KeyCode::P,
        Key::KeyQ => KeyCode::Q,
        Key::KeyR => KeyCode::R,
        Key::KeyS => KeyCode::S,
        Key::KeyT => KeyCode::T,
        Key::KeyU => KeyCode::U,
        Key::KeyV => KeyCode::V,
        Key::KeyW => KeyCode::W,
        Key::KeyX => KeyCode::X,
        Key::KeyY => KeyCode::Y,
        Key::KeyZ => KeyCode::Z,
        Key::Num0 => KeyCode::Digit0,
        Key::Num1 => KeyCode::Digit1,
        Key::Num2 => KeyCode::Digit2,
        Key::Num3 => KeyCode::Digit3,
        Key::Num4 => KeyCode::Digit4,
        Key::Num5 => KeyCode::Digit5,
        Key::Num6 => KeyCode::Digit6,
        Key::Num7 => KeyCode::Digit7,
        Key::Num8 => KeyCode::Digit8,
        Key::Num9 => KeyCode::Digit9,
        Key::Escape => KeyCode::Escape,
        Key::Tab => KeyCode::Tab,
        Key::CapsLock => KeyCode::CapsLock,
        Key::ShiftLeft => KeyCode::ShiftLeft,
        Key::ShiftRight => KeyCode::ShiftRight,
        Key::ControlLeft => KeyCode::ControlLeft,
        Key::ControlRight => KeyCode::ControlRight,
        Key::Alt => KeyCode::AltLeft,
        Key::AltGr => KeyCode::AltRight,
        Key::MetaLeft => KeyCode::MetaLeft,
        Key::MetaRight => KeyCode::MetaRight,
        Key::Space => KeyCode::Space,
        Key::Return | Key::KpReturn => KeyCode::Enter,
        Key::Backspace => KeyCode::Backspace,
        Key::Delete => KeyCode::Delete,
        Key::Insert => KeyCode::Insert,
        Key::Home => KeyCode::Home,
        Key::End => KeyCode::End,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::UpArrow => KeyCode::ArrowUp,
        Key::DownArrow => KeyCode::ArrowDown,
        Key::LeftArrow => KeyCode::ArrowLeft,
        Key::RightArrow => KeyCode::ArrowRight,
        Key::F1 => KeyCode::F1,
        Key::F2 => KeyCode::F2,
        Key::F3 => KeyCode::F3,
        Key::F4 => KeyCode::F4,
        Key::F5 => KeyCode::F5,
        Key::F6 => KeyCode::F6,
        Key::F7 => KeyCode::F7,
        Key::F8 => KeyCode::F8,
        Key::F9 => KeyCode::F9,
        Key::F10 => KeyCode::F10,
        Key::F11 => KeyCode::F11,
        Key::F12 => KeyCode::F12,
        Key::Minus => KeyCode::Minus,
        Key::Equal => KeyCode::Equal,
        Key::LeftBracket => KeyCode::BracketLeft,
        Key::RightBracket => KeyCode::BracketRight,
        Key::BackSlash | Key::IntlBackslash => KeyCode::Backslash,
        Key::SemiColon => KeyCode::Semicolon,
        Key::Quote => KeyCode::Quote,
        Key::Comma => KeyCode::Comma,
        Key::Dot => KeyCode::Period,
        Key::Slash => KeyCode::Slash,
        Key::BackQuote => KeyCode::Backquote,
        _ => KeyCode::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_keys_normalize_without_text() {
        assert_eq!(normalize_key(Key::KeyA), KeyCode::A);
        assert_eq!(normalize_key(Key::Space), KeyCode::Space);
        assert_eq!(normalize_key(Key::LeftArrow), KeyCode::ArrowLeft);
    }
}
