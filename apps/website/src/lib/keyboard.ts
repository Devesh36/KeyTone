export type DemoKeycap = { code: string; label: string; name: string; units: number; tone: "light" | "dark" | "accent" };
const key = (code: string, label: string, units = 1, tone: DemoKeycap["tone"] = "dark", name = label): DemoKeycap => ({ code, label, name, units, tone });
const letters = (row: string) => [...row].map((letter) => key(`Key${letter}`, letter, 1, "light"));

// ANSI-inspired 75% board. Each row occupies exactly 16 key units.
export const keyboardRows: DemoKeycap[][] = [
  [key("Escape", "esc", 1, "accent", "Escape"), key("gap", "", 1.5), ...Array.from({ length: 12 }, (_, i) => key(`F${i + 1}`, `F${i + 1}`)), key("Delete", "del", 1.5, "dark", "Delete")],
  [key("Backquote", "`", 1, "light"), ...[..."1234567890"].map((n) => key(`Digit${n}`, n, 1, "light")), key("Minus", "−", 1, "light"), key("Equal", "=", 1, "light"), key("Backspace", "⌫", 2, "dark", "Backspace"), key("Home", "home")],
  [key("Tab", "tab", 1.5), ...letters("QWERTYUIOP"), key("BracketLeft", "[", 1, "light"), key("BracketRight", "]", 1, "light"), key("Backslash", "\\", 1.5, "light"), key("PageUp", "pgup", 1, "dark", "Page Up")],
  [key("CapsLock", "caps", 1.75, "dark", "Caps Lock"), ...letters("ASDFGHJKL"), key("Semicolon", ";", 1, "light"), key("Quote", "'", 1, "light"), key("Enter", "enter ↵", 2.25, "accent", "Enter"), key("PageDown", "pgdn", 1, "dark", "Page Down")],
  [key("ShiftLeft", "shift", 2.25, "dark", "Left Shift"), ...letters("ZXCVBNM"), key("Comma", ",", 1, "light"), key("Period", ".", 1, "light"), key("Slash", "/", 1, "light"), key("ShiftRight", "shift", 1.75, "dark", "Right Shift"), key("ArrowUp", "↑", 1, "dark", "Up arrow"), key("End", "end")],
  [key("ControlLeft", "ctrl", 1.25, "dark", "Left Control"), key("MetaLeft", "super", 1.25, "dark", "Left Super / Command"), key("AltLeft", "alt", 1.25, "dark", "Left Alt / Option"), key("Space", "KEYTONE", 6.25, "accent", "Space"), key("AltRight", "alt", 1, "dark", "Right Alt / Option"), key("ContextMenu", "menu"), key("ControlRight", "ctrl", 1, "dark", "Right Control"), key("ArrowLeft", "←", 1, "dark", "Left arrow"), key("ArrowDown", "↓", 1, "dark", "Down arrow"), key("ArrowRight", "→", 1, "dark", "Right arrow")],
];

export const keyboardKeys = keyboardRows.flat().filter(({ code }) => code !== "gap");
const codes = new Set(keyboardKeys.map(({ code }) => code));

export function playableCode(code: string): boolean {
  // Reserve browser navigation, function keys, and system modifiers. These
  // keycaps remain playable by pointer or native button activation.
  return codes.has(code) && !/^(Tab|Escape|F\d+|Meta.*|Control.*|Alt.*|ContextMenu)$/.test(code);
}
