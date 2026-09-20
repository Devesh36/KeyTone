import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { createServer } from "vite";
import react from "@vitejs/plugin-react";

let server, downloads, demo, keyboard, SwitchDemo, App;
before(async () => {
  server = await createServer({ configFile: false, plugins: [react()], server: { middlewareMode: true, hmr: false, watch: null } });
  downloads = await server.ssrLoadModule("/src/lib/downloads.ts");
  demo = await server.ssrLoadModule("/src/lib/demo.ts");
  keyboard = await server.ssrLoadModule("/src/lib/keyboard.ts");
  ({ SwitchDemo } = await server.ssrLoadModule("/src/components/SwitchDemo.tsx"));
  ({ default: App } = await server.ssrLoadModule("/src/App.tsx"));
});
after(async () => { await server?.close(); });

test("phones, Android tablets, iPadOS desktop mode, and unknown devices get no misleading desktop recommendation", () => {
  for (const args of [["iPhone", "iPhone", 5], ["Android", "Linux armv8", 5], ["iPad", "iPad", 5], ["Macintosh", "MacIntel", 5], ["unknown", "unknown", 0]]) {
    assert.equal(downloads.detectPlatform(...args), null);
  }
  assert.equal(downloads.detectPlatform("Macintosh", "MacIntel", 0), "macos");
  assert.equal(downloads.detectPlatform("Windows", "Win32", 0), "windows");
  assert.equal(downloads.detectPlatform("Linux", "Linux x86_64", 0), "linux");
});

const asset = (name) => ({ name, size: 1024, browser_download_url: `https://github.com/Devesh36/KeyTone/releases/download/v0.1.3/${name}` });
test("asset selection never offers another OS installer when a platform's build is missing", () => {
  const mac = asset("Keytone_universal.dmg");
  const win = asset("Keytone_x64.msi");
  const linux = asset("Keytone_amd64.AppImage");
  assert.equal(downloads.pickAsset("windows", [mac, linux]), undefined);
  assert.equal(downloads.pickAsset("linux", [mac, win]), undefined);
  assert.equal(downloads.pickAsset("macos", [win, linux]), undefined);
  for (const [platform, expected] of [["macos", mac], ["windows", win], ["linux", linux]]) {
    assert.equal(downloads.pickAsset(platform, [win, linux, mac]), expected);
  }
});

test("malformed or untrusted release URLs fall back to the release page", () => {
  assert.equal(downloads.isRelease({ tag_name: "v0.1.3", assets: [asset("Keytone.dmg")] }), true);
  for (const value of [null, {}, { tag_name: "v0.1.3", assets: [null] }, { tag_name: "v0.1.3", assets: [{ ...asset("keytone.dmg"), browser_download_url: "javascript:alert(1)" }] }]) {
    assert.equal(downloads.isRelease(value), false);
  }
});

test("demo maps physical key positions but reserves browser shortcuts and navigation", () => {
  for (const code of ["KeyA", "Digit4", "Space", "ArrowDown", "Enter", "ShiftLeft", "Backspace", "BracketLeft", "Slash"]) assert.equal(demo.demoKey(code), code);
  for (const key of ["Tab", "Escape", "F1", "F5", "F12", "MetaLeft", "AltRight", "ControlLeft", "ContextMenu", "Unknown"]) assert.equal(demo.demoKey(key), null);
  for (const modifier of ["repeat", "ctrlKey", "metaKey", "altKey", "isComposing"]) assert.equal(demo.demoKey("KeyA", { [modifier]: true }), null);
});

test("complete 75% keyboard has unique codes, aligned rows, and all main key groups", () => {
  const { keyboardRows, keyboardKeys } = keyboard;
  assert.equal(keyboardRows.length, 6);
  assert.equal(keyboardKeys.length, 82);
  assert.equal(new Set(keyboardKeys.map(({ code }) => code)).size, 82);
  for (const row of keyboardRows) assert.equal(row.reduce((sum, key) => sum + key.units, 0), 16);
  const codes = new Set(keyboardKeys.map(({ code }) => code));
  for (const letter of "ABCDEFGHIJKLMNOPQRSTUVWXYZ") assert.ok(codes.has(`Key${letter}`));
  for (const digit of "0123456789") assert.ok(codes.has(`Digit${digit}`));
  for (let i = 1; i <= 12; i++) assert.ok(codes.has(`F${i}`));
  for (const code of ["Space", "Enter", "Tab", "CapsLock", "Backspace", "Delete", "Home", "End", "PageUp", "PageDown", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"]) assert.ok(codes.has(code));
  assert.equal(keyboardKeys.find(({ code }) => code === "Space").units, 6.25);
});

test("demo starts muted with labelled buttons, three selectable profiles, and license attribution", () => {
  for (let profileIndex = 0; profileIndex < 3; profileIndex++) {
    const markup = renderToStaticMarkup(createElement(SwitchDemo, { profileIndex, onProfileChange() {} }));
    assert.match(markup, /Enable sound/);
    assert.match(markup, /tabindex="0"/);
    assert.match(markup, /aria-describedby="demo-help"/);
    assert.match(markup, /demo-audio-license.txt/);
    assert.equal((markup.match(/aria-label="Play /g) ?? []).length, 82);
    assert.equal((markup.match(/tabindex="-1"/g) ?? []).length, 81);
    assert.match(markup, /75% LAYOUT \/ 82 KEYS/);
    assert.match(markup, /Swipe the keyboard/);
    assert.equal((markup.match(/aria-pressed="true"/g) ?? []).length, 1);
    assert.doesNotMatch(markup, /demo-stem-struck|<audio|autoplay/);
  }
});

test("landing page renders without browser APIs and includes mobile navigation and all downloads", () => {
  const markup = renderToStaticMarkup(createElement(App));
  assert.match(markup, /Get the desktop app/);
  assert.match(markup, /aria-controls="main-navigation"/);
  assert.match(markup, /Skip to content/);
  assert.match(markup, /not iOS or Android/);
  assert.equal((markup.match(/class="download-card(?: [^"]*)?"/g) ?? []).length, 3);
});

function mockAudio(t, fetchOverride) {
  const calls = [];
  const contexts = [];
  class AudioContextMock {
    state = "suspended";
    destination = {};
    sources = [];
    constructor() { contexts.push(this); }
    async resume() { calls.push("resume"); this.state = "running"; }
    async decodeAudioData() { return { decoded: true }; }
    async close() { this.state = "closed"; }
    createGain() { return { gain: { value: 1 }, connect() {}, disconnect() {} }; }
    createBufferSource() {
      const source = { connect() {}, disconnect() {}, start() { calls.push("start"); }, stop() { calls.push("stop"); this.onended?.(); } };
      this.sources.push(source);
      return source;
    }
  }
  const previous = globalThis.window;
  globalThis.window = { AudioContext: AudioContextMock };
  t.after(() => { if (previous === undefined) delete globalThis.window; else globalThis.window = previous; });
  t.mock.method(globalThis, "fetch", fetchOverride ?? (async () => { calls.push("fetch"); return { ok: true, arrayBuffer: async () => new ArrayBuffer(0) }; }));
  return { calls, contexts };
}

test("sound is opt-in; resume precedes fetch; decoded samples are cached", async (t) => {
  const { calls } = mockAudio(t);
  const engine = new demo.DemoAudio();
  await engine.play(0);
  assert.deepEqual(calls, []);
  await engine.enable();
  assert.equal(calls[0], "resume");
  assert.equal(calls.filter((call) => call === "fetch").length, 3);
  await engine.play(0);
  assert.equal(calls.at(-1), "start");
  engine.mute();
  await engine.enable();
  assert.equal(calls.filter((call) => call === "fetch").length, 3);
  engine.dispose();
});

test("polyphony is bounded and mute stops active voices", async (t) => {
  const { calls, contexts } = mockAudio(t);
  const engine = new demo.DemoAudio();
  await engine.enable();
  for (let index = 0; index < 30; index++) await engine.play(0);
  assert.equal(contexts[0].sources.length, 12);
  engine.mute();
  assert.equal(calls.filter((call) => call === "stop").length, 12);
  await engine.play(0);
  assert.equal(contexts[0].sources.length, 12);
  engine.dispose();
  assert.equal(contexts[0].state, "closed");
});

test("muting during sample loading prevents late playback enablement", async (t) => {
  const complete = [];
  mockAudio(t, () => new Promise((resolve) => complete.push(resolve)));
  const engine = new demo.DemoAudio();
  const enabling = engine.enable();
  await Promise.resolve();
  assert.equal(complete.length, 3);
  engine.mute();
  for (const resolve of complete) resolve({ ok: true, arrayBuffer: async () => new ArrayBuffer(0) });
  await enabling;
  assert.equal(engine.enabled, false);
  engine.dispose();
});

test("unsupported audio and failed sample loads produce recoverable errors", async (t) => {
  mockAudio(t, async () => ({ ok: false }));
  const engine = new demo.DemoAudio();
  await assert.rejects(engine.enable(), /Couldn't load/);
  assert.equal(engine.enabled, false);
  engine.dispose();
  globalThis.window = {};
  const unsupported = new demo.DemoAudio();
  await assert.rejects(unsupported.enable(), /aren't supported/);
  unsupported.dispose();
});
