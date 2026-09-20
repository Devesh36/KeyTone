export interface Effects {
  masterVolume: number;
  pitch: number;
  pitchRandomness: number;
  volumeRandomness: number;
  bass: number;
  treble: number;
  reverb: number;
  spatial: number;
}

export interface Settings {
  engineEnabled: boolean;
  activePack: string;
  activePreset: string | null;
  effects: Effects;
  releaseSounds: boolean;
  playRepeats: boolean;
  launchAtStartup: boolean;
  keepRunningOnClose: boolean;
  outputDevice: string | null;
}

export interface Preset {
  schemaVersion: number;
  name: string;
  pack: string;
  effects: Effects;
}

export interface PackInfo {
  id: string;
  name: string;
  author: string;
  version: string;
  description: string;
  license: string;
  tags: string[];
  hasReleaseSamples: boolean;
}

export interface AudioStats {
  status: "stopped" | "running" | "deviceError";
  scheduledEvents: number;
  droppedEvents: number;
  averageSchedulingMicros: number;
  lastError: string | null;
}

export interface AppSnapshot {
  settings: Settings;
  presets: Preset[];
  packs: PackInfo[];
  audio: AudioStats;
  outputDevices: string[];
  permission: "unknown" | "granted" | "missing";
  permissionInstructions: string;
  warnings: string[];
}

export type Page = "home" | "lab" | "packs" | "settings";
