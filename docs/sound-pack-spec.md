# Keytone Sound Pack Specification v1

Status: stable for Keytone 0.1.

## Package

A pack is a directory containing `manifest.json` and one or more WAV files. Paths are relative to the pack root. Absolute paths, parent traversal and non-WAV sample entries are invalid. Keytone treats every pack as untrusted data and never executes pack content.

## Manifest

```json
{
  "schemaVersion": 1,
  "id": "deep-cream",
  "name": "Deep Cream",
  "author": "Example Author",
  "version": "1.0.0",
  "description": "Deep creamy linear keyboard sound.",
  "license": "CC0-1.0",
  "tags": ["creamy", "linear"],
  "preview": "preview.wav",
  "samples": {
    "press": {
      "default": ["samples/press/default/01.wav"],
      "alpha": ["samples/press/alpha/01.wav"],
      "space": ["samples/press/space/01.wav"],
      "key:enter": ["samples/press/enter/01.wav"]
    },
    "release": {
      "default": ["samples/release/default/01.wav"]
    }
  }
}
```

Required fields are `schemaVersion`, `id`, `name`, `author`, `version`, `description`, `license` and `samples.press`. An ID contains ASCII letters, numbers, hyphens or underscores.

`preview` and `tags` are optional. Release mappings are optional.

## Mappings

Valid categories are:

- `default`
- `alpha`
- `number`
- `space`
- `enter`
- `backspace`
- `modifier`
- `arrow`
- `function`

An explicit physical-key mapping uses `key:<key-code>`, where the key code is the kebab-case serialized Keytone code (for example `key:a`, `key:space`, `key:shift-left`). Selection order is explicit key, category, then default. One entry may list multiple WAVs; Keytone selects among them per event.

## WAV support

Keytone v0.1 accepts PCM integer WAV through 32-bit and 32-bit float WAV at 8 kHz or above, with one to eight channels. Multichannel files are downmixed to mono on load because the physical-key model supplies spatial position. Source sample rate is converted during playback.

Keep individual samples short and trim leading silence. A typical pack should use 44.1 or 48 kHz source audio.

## Licensing

Authors must set an accurate SPDX-style license identifier where possible and include any required attribution beside the pack. Do not redistribute recordings extracted from commercial applications, products or sample libraries without explicit permission.
