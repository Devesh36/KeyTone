use std::time::Instant;

use serde::{Deserialize, Serialize};

/// A physical-key identity. It intentionally contains no typed characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Escape,
    Tab,
    CapsLock,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    MetaLeft,
    MetaRight,
    Space,
    Enter,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,
    Backquote,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyState {
    Pressed,
    Released,
}

/// Ephemeral input event. `timestamp` is monotonic and never serialized or persisted.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub state: KeyState,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyCategory {
    Default,
    Alpha,
    Number,
    Space,
    Enter,
    Backspace,
    Modifier,
    Arrow,
    Function,
}

impl KeyCode {
    #[must_use]
    pub const fn category(self) -> KeyCategory {
        match self {
            Self::A
            | Self::B
            | Self::C
            | Self::D
            | Self::E
            | Self::F
            | Self::G
            | Self::H
            | Self::I
            | Self::J
            | Self::K
            | Self::L
            | Self::M
            | Self::N
            | Self::O
            | Self::P
            | Self::Q
            | Self::R
            | Self::S
            | Self::T
            | Self::U
            | Self::V
            | Self::W
            | Self::X
            | Self::Y
            | Self::Z => KeyCategory::Alpha,
            Self::Digit0
            | Self::Digit1
            | Self::Digit2
            | Self::Digit3
            | Self::Digit4
            | Self::Digit5
            | Self::Digit6
            | Self::Digit7
            | Self::Digit8
            | Self::Digit9 => KeyCategory::Number,
            Self::Space => KeyCategory::Space,
            Self::Enter => KeyCategory::Enter,
            Self::Backspace | Self::Delete => KeyCategory::Backspace,
            Self::ShiftLeft
            | Self::ShiftRight
            | Self::ControlLeft
            | Self::ControlRight
            | Self::AltLeft
            | Self::AltRight
            | Self::MetaLeft
            | Self::MetaRight
            | Self::CapsLock => KeyCategory::Modifier,
            Self::ArrowUp | Self::ArrowDown | Self::ArrowLeft | Self::ArrowRight => {
                KeyCategory::Arrow
            }
            Self::F1
            | Self::F2
            | Self::F3
            | Self::F4
            | Self::F5
            | Self::F6
            | Self::F7
            | Self::F8
            | Self::F9
            | Self::F10
            | Self::F11
            | Self::F12 => KeyCategory::Function,
            _ => KeyCategory::Default,
        }
    }

    /// Approximate horizontal position in the range `[-1, 1]` for stereo placement.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Grouping by keyboard row is easier to audit visually.
    pub fn horizontal_position(self) -> f32 {
        let position = match self {
            Self::Escape | Self::Backquote | Self::Tab | Self::CapsLock | Self::ShiftLeft => 0.0,
            Self::Digit1 | Self::Q | Self::A | Self::Z => 1.0,
            Self::Digit2 | Self::W | Self::S | Self::X => 2.0,
            Self::Digit3 | Self::E | Self::D | Self::C => 3.0,
            Self::Digit4 | Self::R | Self::F | Self::V => 4.0,
            Self::Digit5 | Self::T | Self::G | Self::B => 5.0,
            Self::Digit6 | Self::Y | Self::H | Self::N | Self::Space => 6.0,
            Self::Digit7 | Self::U | Self::J | Self::M => 7.0,
            Self::Digit8 | Self::I | Self::K | Self::Comma => 8.0,
            Self::Digit9 | Self::O | Self::L | Self::Period => 9.0,
            Self::Digit0 | Self::P | Self::Semicolon | Self::Slash => 10.0,
            Self::Minus | Self::BracketLeft | Self::Quote | Self::AltRight => 11.0,
            Self::Equal | Self::BracketRight | Self::Backslash | Self::MetaRight => 12.0,
            Self::Backspace | Self::Enter | Self::ShiftRight | Self::ArrowLeft => 13.0,
            Self::ArrowDown | Self::ArrowUp => 14.0,
            Self::Delete
            | Self::Insert
            | Self::Home
            | Self::End
            | Self::PageUp
            | Self::PageDown => 14.5,
            Self::ArrowRight => 15.0,
            Self::F1 | Self::F2 => 2.0,
            Self::F3 | Self::F4 => 4.0,
            Self::F5 | Self::F6 => 7.0,
            Self::F7 | Self::F8 => 9.0,
            Self::F9 | Self::F10 => 12.0,
            Self::F11 | Self::F12 => 14.0,
            Self::ControlLeft | Self::AltLeft | Self::MetaLeft => 2.0,
            Self::ControlRight => 13.0,
            Self::Unknown => 7.5,
        };
        (position / 7.5 - 1.0_f32).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_are_specific_before_default() {
        assert_eq!(KeyCode::A.category(), KeyCategory::Alpha);
        assert_eq!(KeyCode::Space.category(), KeyCategory::Space);
        assert_eq!(KeyCode::ArrowLeft.category(), KeyCategory::Arrow);
        assert_eq!(KeyCode::Tab.category(), KeyCategory::Default);
    }

    #[test]
    fn keyboard_coordinates_have_expected_direction() {
        assert!(KeyCode::Q.horizontal_position() < 0.0);
        assert!(KeyCode::P.horizontal_position() > 0.0);
        assert!(KeyCode::G.horizontal_position().abs() < 0.4);
        assert!((-1.0..=1.0).contains(&KeyCode::ArrowRight.horizontal_position()));
    }
}
