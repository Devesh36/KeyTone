# Privacy and threat model

Global keyboard access is sensitive. Keytone minimizes what it observes and what it retains.

## Guarantees

- A normalized event contains physical key identity, pressed/released state and a monotonic timestamp.
- Keytone does not use printable characters supplied by the operating system.
- Events are handled independently. No component constructs words, lines or sequences.
- Key events and key history are never written to disk.
- Key events never cross the Tauri IPC boundary and are not exposed to React.
- No analytics, telemetry, account, cloud API or network service exists.
- Release builds do not log individual keys. Development timing uses only aggregate counts and durations.

## Stored data

Keytone stores user preferences, effect values, preset names, pack identifiers and an optional output-device name in the OS application-data directory. Imported sound files live beside that configuration. Corrupt settings are ignored safely.

## Pack boundary

Packs are untrusted data. The loader accepts a strict JSON schema and relative WAV paths, rejects traversal and unsupported formats, and never executes code from a pack. Import copies only files referenced by the validated manifest.

## Out of scope

An already-compromised local account, malicious OS kernel, injected process or altered Keytone binary can violate these guarantees. Keytone cannot capture events a platform security policy refuses to deliver, and it does not bypass that policy.
