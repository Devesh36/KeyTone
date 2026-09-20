import { useEffect, useRef, useState, type CSSProperties, type KeyboardEvent } from "react";
import { AudioLines, Volume2, VolumeX } from "lucide-react";
import { DemoAudio, demoKey, demoProfiles } from "../lib/demo";
import { keyboardKeys, keyboardRows } from "../lib/keyboard";

export function SwitchDemo({ profileIndex, onProfileChange }: { profileIndex: number; onProfileChange: (index: number) => void }) {
  const [activeKeys, setActiveKeys] = useState<Set<string>>(() => new Set());
  const [focusKey, setFocusKey] = useState("Escape");
  const [sound, setSound] = useState<"off" | "loading" | "on">("off");
  const [error, setError] = useState("");
  const audio = useRef<DemoAudio | null>(null);
  const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>());
  const mounted = useRef(false);
  const profile = demoProfiles[profileIndex];

  useEffect(() => {
    mounted.current = true;
    const pending = timers.current;
    const silence = () => { audio.current?.mute(); setSound("off"); setActiveKeys(new Set()); };
    const visibility = () => { if (document.hidden) silence(); };
    window.addEventListener("blur", silence);
    document.addEventListener("visibilitychange", visibility);
    return () => {
      mounted.current = false;
      for (const timer of pending.values()) clearTimeout(timer);
      pending.clear();
      audio.current?.dispose();
      audio.current = null;
      window.removeEventListener("blur", silence);
      document.removeEventListener("visibilitychange", visibility);
    };
  }, []);

  const press = (key: string) => {
    setActiveKeys((keys) => new Set(keys).add(key));
    clearTimeout(timers.current.get(key));
    timers.current.set(key, setTimeout(() => {
      setActiveKeys((keys) => { const next = new Set(keys); next.delete(key); return next; });
      timers.current.delete(key);
    }, 220));
    void audio.current?.play(profileIndex).catch(() => {
      if (!mounted.current) return;
      audio.current?.mute(); setSound("off");
      setError("Sound was paused by your browser. Try enabling it again.");
    });
  };

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const target = event.target as HTMLElement;
    if (event.target !== event.currentTarget && !target.closest("[data-demo-key]")) return;
    if (event.ctrlKey || event.metaKey || event.altKey || event.nativeEvent.isComposing) return;
    // One tab stop for the board; arrow keys navigate its buttons.
    const button = target.closest<HTMLButtonElement>("[data-demo-key]");
    if (button && event.code.startsWith("Arrow")) {
      const row = keyboardRows.findIndex((keys) => keys.some(({ code }) => code === button.dataset.demoKey));
      const column = keyboardRows[row].findIndex(({ code }) => code === button.dataset.demoKey);
      let next = button.dataset.demoKey;
      if (event.code === "ArrowLeft" || event.code === "ArrowRight") {
        const index = keyboardKeys.findIndex(({ code }) => code === next);
        next = keyboardKeys[(index + (event.code === "ArrowLeft" ? -1 : 1) + keyboardKeys.length) % keyboardKeys.length].code;
      } else {
        const nextRow = keyboardRows[(row + (event.code === "ArrowUp" ? -1 : 1) + keyboardRows.length) % keyboardRows.length].filter(({ code }) => code !== "gap");
        next = nextRow[Math.min(column, nextRow.length - 1)].code;
      }
      event.preventDefault();
      event.currentTarget.querySelector<HTMLButtonElement>(`[data-demo-key="${next}"]`)?.focus();
      return;
    }
    if (event.repeat) {
      if (demoKey(event.code) || (button && event.code === "Space")) event.preventDefault();
      return;
    }
    // Let native buttons handle Space/Enter without generating duplicate clicks.
    if (target.closest("button") && (event.code === "Space" || event.code === "Enter")) return;
    const key = demoKey(event.code, event.nativeEvent);
    if (!key) return;
    event.preventDefault(); press(key);
  };

  const toggleSound = async () => {
    setError("");
    if (sound === "on") { audio.current?.mute(); setSound("off"); return; }
    const engine = audio.current ??= new DemoAudio();
    setSound("loading");
    try {
      await engine.enable();
      if (!mounted.current) return;
      if (document.hidden) engine.mute();
      setSound(engine.enabled ? "on" : "off");
    } catch (reason) {
      if (!mounted.current) return;
      setSound("off");
      setError(reason instanceof Error ? reason.message : "Couldn't start audio. The animation still works.");
    }
  };

  return (
    <div className={`switch-demo${activeKeys.size ? " is-active" : ""}`} id="demo" tabIndex={0} role="group" aria-label="Interactive keyboard demo" aria-describedby="demo-help" onKeyDown={onKeyDown} style={{ "--switch-color": profile.color, "--stem-color": profile.stem } as CSSProperties}>
      <div className="demo-head"><span className="eyebrow"><AudioLines size={14} /> SWITCH PLAYGROUND</span><span className="demo-indicator"><i /> Interactive</span></div>
      <div className="keyboard-caption"><span><strong>Your desk. Your sound.</strong><small>75% LAYOUT / {keyboardKeys.length} KEYS</small></span><span className="keyboard-edition">KT—75</span></div>
      <div className="keyboard-scroll" role="group" aria-label="Mechanical keyboard; scroll horizontally on small screens">
        <div className="mechanical-board">
          <div className="keyboard-nameplate" aria-hidden="true"><span>KEYTONE <b> / CUSTOM SERIES</b></span><span className="keyboard-leds"><i /><i /><i /></span></div>
          {keyboardRows.map((row, index) => <div className="keyboard-row" key={index}>
            {row.map((key) => key.code === "gap" ? <span key={key.code} style={{ flex: key.units }} aria-hidden="true" /> : <button type="button" data-demo-key={key.code} key={key.code} tabIndex={focusKey === key.code ? 0 : -1} onFocus={() => setFocusKey(key.code)} className={`keyboard-key key-${key.tone}${activeKeys.has(key.code) ? " is-pressed" : ""}${key.code === "Space" ? " key-space" : ""}`} style={{ flex: key.units }} aria-label={`Play ${key.name} key`} onClick={() => press(key.code)}><span>{key.label}</span></button>)}
          </div>)}
          <div className="keyboard-case-edge" aria-hidden="true"><i /><span>BUILT FOR THE FEELING.</span><i /></div>
        </div>
      </div>
      <p className="keyboard-scroll-hint">Swipe the keyboard to explore every key →</p>
      <div className="keyboard-profile-info">
        <span className="stage-spec">{profile.name}<small>{profile.character}</small></span>
        <div className="demo-meter" aria-hidden="true">{[30, 55, 38, 76, 100, 62, 42, 70, 30].map((height, index) => <i key={index} style={{ "--bar-height": `${height}%` } as CSSProperties} />)}</div>
      </div>
      <div className="demo-profiles" role="group" aria-label="Choose demo switch">
        {demoProfiles.map((item, index) => <button type="button" key={item.id} aria-pressed={index === profileIndex} onClick={() => { onProfileChange(index); setActiveKeys(new Set()); }}><i style={{ background: item.color }} />{item.name}</button>)}
      </div>
      <div className="demo-footer"><p id="demo-help">Click any key, or focus here and type.<small>Arrow keys navigate keycaps; Enter plays. Shortcuts stay yours.</small><small>Only active inside this demo. Nothing recorded.</small></p><button type="button" className="sound-toggle" disabled={sound === "loading"} aria-pressed={sound === "on"} onClick={() => void toggleSound()}>{sound === "on" ? <Volume2 size={16} /> : <VolumeX size={16} />}{sound === "loading" ? "Loading…" : sound === "on" ? "Sound on" : "Enable sound"}</button></div>
      {error && <p className="demo-error" role="status">{error}</p>}
      <p className="demo-disclaimer">Browser preview · The desktop audio engine runs in Rust.<br /><a href="./demo-audio-license.txt">Samples: Thomas Lai / kbsim · MIT</a></p>
    </div>
  );
}
