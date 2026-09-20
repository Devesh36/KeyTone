# Development guide

See the root [CONTRIBUTING.md](../CONTRIBUTING.md) for the contribution process.

Engine changes should preserve the real-time contract documented in [architecture.md](architecture.md). Keep OS types inside `keytone-input`, keep data validation inside `keytone-packs`, and do not route audio triggers through the webview.

Add unit tests for deterministic mapping, parsing or signal math. Native APIs belong behind interfaces that can be exercised with mocks. Never add keyboard-event logging to troubleshoot an issue; counters and timing spans are sufficient.
