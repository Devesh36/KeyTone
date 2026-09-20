//! macOS keycode-only event tap.
//!
//! We deliberately do not ask the Text Input Source APIs to translate a
//! physical key into text. Besides being unnecessary for Keytone's privacy
//! model, those APIs are main-queue-bound on current macOS releases.

#![allow(unsafe_code)]

use std::collections::HashSet;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use keytone_core::{KeyCode, KeyEvent, KeyState};
use objc2_core_foundation::{kCFRunLoopCommonModes, CFMachPort, CFRunLoop};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType,
};

use crate::InputError;

const KEYBOARD_EVENT_MASK: u64 = (1 << CGEventType::KeyDown.0)
    | (1 << CGEventType::KeyUp.0)
    | (1 << CGEventType::FlagsChanged.0);

struct CallbackContext {
    active: Arc<AtomicBool>,
    play_repeats: Arc<AtomicBool>,
    pressed: HashSet<KeyCode>,
    emit: Box<dyn Fn(KeyEvent) + Send>,
}

pub(super) fn listen(
    active: Arc<AtomicBool>,
    play_repeats: Arc<AtomicBool>,
    emit: impl Fn(KeyEvent) + Send + 'static,
    on_ready: impl Fn() + Send + 'static,
) -> Result<(), InputError> {
    let context = Box::new(CallbackContext {
        active,
        play_repeats,
        pressed: HashSet::new(),
        emit: Box::new(emit),
    });
    let context = Box::into_raw(context);

    // SAFETY: `context` remains alive until after the run loop exits, and the
    // callback returns the borrowed event unchanged as required by CoreGraphics.
    let tap = unsafe {
        CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            KEYBOARD_EVENT_MASK,
            Some(event_callback),
            context.cast::<c_void>(),
        )
    };
    let Some(tap) = tap else {
        // SAFETY: event-tap creation failed, so CoreGraphics cannot retain or
        // use the context pointer.
        unsafe { drop(Box::from_raw(context)) };
        return Err(InputError::Listener(
            "could not create the macOS keyboard event tap".into(),
        ));
    };

    let Some(source) = CFMachPort::new_run_loop_source(None, Some(&tap), 0) else {
        // SAFETY: no run-loop source exists, so the callback cannot run.
        unsafe { drop(Box::from_raw(context)) };
        return Err(InputError::Listener(
            "could not create the macOS keyboard run-loop source".into(),
        ));
    };
    let Some(run_loop) = CFRunLoop::current() else {
        // SAFETY: the event tap was never attached to a run loop.
        unsafe { drop(Box::from_raw(context)) };
        return Err(InputError::Listener(
            "could not access the keyboard listener run loop".into(),
        ));
    };

    // SAFETY: this is an immutable CoreFoundation constant valid for the
    // process lifetime.
    let common_modes = unsafe { kCFRunLoopCommonModes };
    run_loop.add_source(Some(&source), common_modes);
    CGEvent::tap_enable(&tap, true);
    on_ready();
    CFRunLoop::run();

    // SAFETY: the run loop has exited and the source/tap are about to drop, so
    // CoreGraphics can no longer invoke the callback with this pointer.
    unsafe { drop(Box::from_raw(context)) };
    Ok(())
}

unsafe extern "C-unwind" fn event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: NonNull<CGEvent>,
    user_info: *mut c_void,
) -> *mut CGEvent {
    let event_ptr = event.as_ptr();
    if user_info.is_null() {
        return event_ptr;
    }

    // Never allow an unexpected Rust panic to cross the CoreGraphics callback
    // boundary. A failed event is simply ignored.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `listen` passes a live `CallbackContext` for the entire run
        // loop lifetime, and this callback is serialized by that run loop.
        let context = unsafe { &mut *user_info.cast::<CallbackContext>() };
        if !context.active.load(Ordering::Relaxed) {
            return;
        }

        // SAFETY: CoreGraphics supplies a non-null event valid for the callback.
        let event = unsafe { event.as_ref() };
        let keycode = CGEvent::integer_value_field(Some(event), CGEventField::KeyboardEventKeycode);
        let Ok(keycode) = u16::try_from(keycode) else {
            return;
        };
        let key = key_from_macos_keycode(keycode);
        if key == KeyCode::Unknown {
            return;
        }

        let state = match event_type {
            CGEventType::KeyDown => {
                let is_repeat = !context.pressed.insert(key);
                if is_repeat && !context.play_repeats.load(Ordering::Relaxed) {
                    return;
                }
                KeyState::Pressed
            }
            CGEventType::KeyUp => {
                context.pressed.remove(&key);
                KeyState::Released
            }
            CGEventType::FlagsChanged => {
                if context.pressed.remove(&key) {
                    KeyState::Released
                } else {
                    context.pressed.insert(key);
                    KeyState::Pressed
                }
            }
            _ => return,
        };

        (context.emit)(KeyEvent {
            key,
            state,
            timestamp: Instant::now(),
        });
    }));

    event_ptr
}

#[must_use]
pub(super) const fn key_from_macos_keycode(code: u16) -> KeyCode {
    match code {
        0 => KeyCode::A,
        1 => KeyCode::S,
        2 => KeyCode::D,
        3 => KeyCode::F,
        4 => KeyCode::H,
        5 => KeyCode::G,
        6 => KeyCode::Z,
        7 => KeyCode::X,
        8 => KeyCode::C,
        9 => KeyCode::V,
        11 => KeyCode::B,
        12 => KeyCode::Q,
        13 => KeyCode::W,
        14 => KeyCode::E,
        15 => KeyCode::R,
        16 => KeyCode::Y,
        17 => KeyCode::T,
        18 => KeyCode::Digit1,
        19 => KeyCode::Digit2,
        20 => KeyCode::Digit3,
        21 => KeyCode::Digit4,
        22 => KeyCode::Digit6,
        23 => KeyCode::Digit5,
        24 => KeyCode::Equal,
        25 => KeyCode::Digit9,
        26 => KeyCode::Digit7,
        27 => KeyCode::Minus,
        28 => KeyCode::Digit8,
        29 => KeyCode::Digit0,
        30 => KeyCode::BracketRight,
        31 => KeyCode::O,
        32 => KeyCode::U,
        33 => KeyCode::BracketLeft,
        34 => KeyCode::I,
        35 => KeyCode::P,
        36 | 76 => KeyCode::Enter,
        37 => KeyCode::L,
        38 => KeyCode::J,
        39 => KeyCode::Quote,
        40 => KeyCode::K,
        41 => KeyCode::Semicolon,
        42 => KeyCode::Backslash,
        43 => KeyCode::Comma,
        44 => KeyCode::Slash,
        45 => KeyCode::N,
        46 => KeyCode::M,
        47 => KeyCode::Period,
        48 => KeyCode::Tab,
        49 => KeyCode::Space,
        50 => KeyCode::Backquote,
        51 => KeyCode::Backspace,
        53 => KeyCode::Escape,
        54 => KeyCode::MetaRight,
        55 => KeyCode::MetaLeft,
        56 => KeyCode::ShiftLeft,
        57 => KeyCode::CapsLock,
        58 => KeyCode::AltLeft,
        59 => KeyCode::ControlLeft,
        60 => KeyCode::ShiftRight,
        61 => KeyCode::AltRight,
        62 => KeyCode::ControlRight,
        96 => KeyCode::F5,
        97 => KeyCode::F6,
        98 => KeyCode::F7,
        99 => KeyCode::F3,
        100 => KeyCode::F8,
        101 => KeyCode::F9,
        103 => KeyCode::F11,
        109 => KeyCode::F10,
        111 => KeyCode::F12,
        114 => KeyCode::Insert,
        115 => KeyCode::Home,
        116 => KeyCode::PageUp,
        117 => KeyCode::Delete,
        118 => KeyCode::F4,
        119 => KeyCode::End,
        120 => KeyCode::F2,
        121 => KeyCode::PageDown,
        122 => KeyCode::F1,
        123 => KeyCode::ArrowLeft,
        124 => KeyCode::ArrowRight,
        125 => KeyCode::ArrowDown,
        126 => KeyCode::ArrowUp,
        _ => KeyCode::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_physical_ansi_keys_without_text_translation() {
        assert_eq!(key_from_macos_keycode(0), KeyCode::A);
        assert_eq!(key_from_macos_keycode(12), KeyCode::Q);
        assert_eq!(key_from_macos_keycode(49), KeyCode::Space);
        assert_eq!(key_from_macos_keycode(123), KeyCode::ArrowLeft);
        assert_eq!(key_from_macos_keycode(u16::MAX), KeyCode::Unknown);
    }
}
