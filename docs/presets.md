# Presets

A preset is processing configuration plus a sound-pack reference. It does not contain audio.

```json
{
  "schemaVersion": 1,
  "name": "Late Night Cream",
  "pack": "creamy",
  "effects": {
    "masterVolume": 0.8,
    "pitch": -0.3,
    "pitchRandomness": 0.02,
    "volumeRandomness": 0.03,
    "bass": 4,
    "treble": -2,
    "reverb": 0.08,
    "spatial": 0.75
  }
}
```

Pitch is measured in semitones. Random values are fractional ranges. Bass and treble are decibels. Master, room and spatial values are normalized. Keytone clamps every effect to its supported range and replaces non-finite numeric input with safe defaults.

The UI can save (or update by name), load, duplicate and reset a preset. Loading fails visibly if its referenced pack is not installed.
