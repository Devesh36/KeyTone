# Demo pack sources

Keytone generates its seven starter packs into the writable application-data directory on first launch. The synthesis formulas are in `crates/keytone-packs/src/lib.rs`; no third-party or proprietary recordings are used.

- Creamy: damped low-frequency sinusoid plus a short noise transient
- Clicky: fast-decay noise plus a high resonant component
- Retro Terminal: quantized noise transient plus a mid-frequency body
- Deep Thock: layered low resonators plus a damped case transient
- Tactile Workshop: separated tactile-bump and bottom-out transients
- Alloy Linear: crisp contact noise plus bright case resonances
- Spring Clack: snap transient plus long, beating spring resonances

All generated demo audio is dedicated under CC0-1.0. The source code that generates it remains MIT licensed.

## Recorded switch packs

The twelve `kbsim-*` directories contain recordings adapted from
[tplai/kbsim](https://github.com/tplai/kbsim) at the pinned commit documented in each
pack's `SOURCE.md`. The upstream MP3 files are MIT licensed and are converted to mono,
48 kHz, 16-bit PCM WAV so Keytone can decode and preload them without an MP3 decoder in
the real-time application.

Each recorded pack includes the upstream MIT license. See the repository-level
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) for attribution.
