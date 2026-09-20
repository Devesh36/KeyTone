# Landing-page compatibility checks

The website is a static Vite/React app. Its existing Vercel hosting setup is unchanged.

## Automated checks

Run `pnpm --dir apps/website lint`, `typecheck`, `test`, and `build`.
Tests cover mobile/desktop recommendations, platform-specific release assets,
untrusted API responses, scoped key normalization, the complete 82-key layout,
equal row widths, unique key codes, initial accessible markup,
and the preview audio lifecycle using a mocked Web Audio context.
These are not browser-rendering or hardware-audio tests.

The build targets ES2020, Chrome/Edge 90, Firefox 90, and Safari 14 syntax.
This is a compilation target, not a claim that every browser/version is tested.
The intended support baseline is current Chrome, Edge, Firefox, and Safari,
including Android Chrome and iOS Safari. Internet Explorer is not supported.

## Manual browser checklist

Test Chrome, Firefox, Edge, and Safari at 320, 375, 390, 640, 768, 960, 1024,
1280, and 1440 CSS pixels, plus iOS/Android portrait and landscape:

- No page-level horizontal scrolling or clipped headlines, cards, or download labels.
  Below 640px, the keyboard itself scrolls horizontally so the keycaps stay usable.
- Menu opens and closes; links close it; Escape closes it and restores focus.
- Tab navigation exposes a visible focus outline and skips the collapsed menu.
- Tap a keycap or focus the demo and type: one finite press animation.
  Rapid typing lights up overlapping keys, without storing any key history.
- The 82-key board has one Tab stop. Arrow keys move between focused keycaps;
  Tab leaves the board. Enter/Space plays the focused keycap.
- Enter/Space activates a focused button once; Tab, browser shortcuts, and
  repeated/composing input do not trigger extra demo sounds.
- Switching profiles changes the Escape, Enter, and spacebar colors and subsequent sound.
- Sound is off initially. Enable it with a gesture, try rapid overlapping
  presses, mute, and verify switching away from the tab/window silences it.
- Keyboard activity outside the demo does not trigger it or get recorded.
- Sound failure leaves the animation usable and shows a helpful status.
- Reduced-motion preference disables animation; pinch zoom and 200% page zoom
  remain usable. Test high-contrast mode and fallback fonts too.
- Mobile visitors see desktop-platform choices, not a recommended phone installer.
- With GitHub unavailable or rate-limited, downloads fall back to GitHub Releases.

## Implementation notes

The 75% mechanical keyboard uses CSS keycap geometry, bevels, and finite press
transforms with static, readable defaults. Function keys, Tab, Escape, and system
modifiers can be clicked in the preview without intercepting browser shortcuts.
Header blur is
optional, gated by `@supports`, with a solid background fallback. Touch controls
use native buttons; none depends on hover. Animation is event-driven, not an
infinite background loop.

Browser audio is an explicitly enabled, small demonstration—not the native Rust
engine. It resumes from a user gesture, caches three bundled MIT-licensed WAV
samples, caps overlapping sources, and closes its context on unmount. Samples
and attribution are self-contained in `apps/website`, including when Vercel
uses that directory as the project root.

References: [Web Audio best practices](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Best_practices),
[reduced-motion preference](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion).
