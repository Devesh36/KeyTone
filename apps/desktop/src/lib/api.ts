import { invoke } from "@tauri-apps/api/core";
import type { AppSnapshot, Effects } from "../types";

const defaultEffects: Effects = {
  masterVolume: 0.82,
  pitch: 0,
  pitchRandomness: 0.02,
  volumeRandomness: 0.03,
  bass: 2,
  treble: 0,
  reverb: 0.08,
  spatial: 0.72,
};

let browserState: AppSnapshot = {
  settings: {
    engineEnabled: true,
    activePack: "creamy",
    activePreset: "Late Night Cream",
    effects: defaultEffects,
    releaseSounds: true,
    playRepeats: false,
    launchAtStartup: false,
    keepRunningOnClose: true,
    outputDevice: null,
  },
  presets: [{ schemaVersion: 1, name: "Late Night Cream", pack: "creamy", effects: defaultEffects }],
  packs: [
    { id: "creamy", name: "Creamy", author: "Keytone", version: "1.0.0", description: "Warm, damped linear switches with a soft bottom-out.", license: "CC0-1.0", tags: ["creamy", "linear"], hasReleaseSamples: true },
    { id: "clicky", name: "Clicky", author: "Keytone", version: "1.0.0", description: "A bright tactile click with a short, crisp decay.", license: "CC0-1.0", tags: ["bright", "tactile"], hasReleaseSamples: true },
    { id: "retro", name: "Retro Terminal", author: "Keytone", version: "1.0.0", description: "Dry, mid-forward typewriter-inspired impacts.", license: "CC0-1.0", tags: ["retro", "dry"], hasReleaseSamples: true },
  ],
  audio: { status: "running", scheduledEvents: 1248, droppedEvents: 0, averageSchedulingMicros: 43, lastError: null },
  outputDevices: ["System Default"],
  permission: "granted",
  permissionInstructions: "Enable Keytone in System Settings → Privacy & Security → Input Monitoring.",
  warnings: [],
};

function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(name, args);
}

export const api = {
  getState: () => isTauri() ? command<AppSnapshot>("get_state") : Promise.resolve(browserState),
  setEngine: async (enabled: boolean) => {
    if (isTauri()) return command<AppSnapshot>("set_engine", { enabled });
    browserState = { ...browserState, settings: { ...browserState.settings, engineEnabled: enabled } };
    return browserState;
  },
  updateEffects: async (effects: Effects) => {
    if (isTauri()) return command<AppSnapshot>("update_effects", { effects });
    browserState = { ...browserState, settings: { ...browserState.settings, effects } };
    return browserState;
  },
  updatePreferences: async (releaseSounds: boolean, playRepeats: boolean, launchAtStartup: boolean, keepRunningOnClose: boolean) => {
    if (isTauri()) return command<AppSnapshot>("update_preferences", { releaseSounds, playRepeats, launchAtStartup, keepRunningOnClose });
    browserState = { ...browserState, settings: { ...browserState.settings, releaseSounds, playRepeats, launchAtStartup, keepRunningOnClose } };
    return browserState;
  },
  activatePack: async (id: string) => {
    if (isTauri()) return command<AppSnapshot>("activate_pack", { id });
    browserState = { ...browserState, settings: { ...browserState.settings, activePack: id, activePreset: null } };
    return browserState;
  },
  previewPack: (id: string) => isTauri() ? command<void>("preview_pack", { id }) : Promise.resolve(),
  importPack: (path: string) => command<AppSnapshot>("import_pack", { path }),
  savePreset: async (name: string) => isTauri() ? command<AppSnapshot>("save_preset", { name }) : browserState,
  loadPreset: (name: string) => command<AppSnapshot>("load_preset", { name }),
  duplicatePreset: (name: string) => command<AppSnapshot>("duplicate_preset", { name }),
  resetEffects: async () => {
    if (isTauri()) return command<AppSnapshot>("reset_effects");
    browserState = { ...browserState, settings: { ...browserState.settings, effects: defaultEffects } };
    return browserState;
  },
  selectOutputDevice: (name: string | null) => command<AppSnapshot>("select_output_device", { name }),
  testSound: () => isTauri() ? command<void>("test_sound") : Promise.resolve(),
  openKeyboardSettings: () => isTauri() ? command<AppSnapshot>("open_keyboard_settings") : Promise.resolve(browserState),
  isTauri,
};
