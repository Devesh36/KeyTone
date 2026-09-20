import { ArrowRight, Radio, SlidersHorizontal, Volume2 } from "lucide-react";
import { Toggle } from "../components/Toggle";
import type { AppSnapshot, Page } from "../types";

interface HomeProps {
  snapshot: AppSnapshot;
  busy: boolean;
  onEngine: (enabled: boolean) => void;
  onVolume: (volume: number) => void;
  onNavigate: (page: Page) => void;
  onTest: () => void;
}

export function Home({ snapshot, busy, onEngine, onVolume, onNavigate, onTest }: HomeProps) {
  const { settings } = snapshot;
  const pack = snapshot.packs.find((item) => item.id === settings.activePack);
  const live = settings.engineEnabled && snapshot.inputConnected && snapshot.audio.status === "running";
  return (
    <div className="page home-page">
      <header className="page-header home-heading">
        <div>
          <p className="eyebrow">KEYTONE / LISTEN</p>
          <h1>Make every keystroke<br /><em>sound yours.</em></h1>
        </div>
        <div className={`engine-orb ${live ? "active" : ""}`}>
          <div><Radio size={22} /><span>{live ? "LIVE" : settings.engineEnabled ? "CHECK ACCESS" : "MUTED"}</span></div>
        </div>
      </header>

      {!snapshot.inputConnected && (
        <section className="keyboard-access-alert" role="alert">
          <div><strong>Keyboard listener is disconnected</strong><span>{snapshot.permission === "missing" ? "Enable keyboard access in Settings, then restart Keytone." : "Access may be approved, but capture has not connected. Open Settings to restart and check it."}</span></div>
          <button type="button" onClick={() => onNavigate("settings")}>Fix access <ArrowRight size={15} /></button>
        </section>
      )}
      {snapshot.audio.lastError && <div className="warning-banner" role="alert">Audio output: {snapshot.audio.lastError}. Choose an output device in Settings.</div>}

      <section className="engine-strip">
        <div><span className="section-kicker">ENGINE</span><strong>{live ? "Sound follows your keyboard" : settings.engineEnabled ? "Setup needs attention" : "Keystrokes are silent"}</strong></div>
        <div className="engine-control"><span className={settings.engineEnabled ? "online" : ""}>{settings.engineEnabled ? "ON" : "OFF"}</span><Toggle checked={settings.engineEnabled} onChange={onEngine} label="Sound engine" disabled={busy} /></div>
      </section>

      <div className="home-grid">
        <button className="sound-card" onClick={() => onNavigate("packs")}>
          <div className="sound-card-top"><span className="section-kicker">CURRENT SOUND</span><ArrowRight size={18} /></div>
          <div className={`pack-art pack-${pack?.id ?? "creamy"}`}><span /><span /><span /><span /><span /></div>
          <div className="sound-card-copy"><div><h2>{pack?.name ?? "No pack"}</h2><p>{pack?.tags.join(" • ")}</p></div><span className="change-link">Change</span></div>
        </button>

        <section className="volume-card">
          <div className="card-icon"><Volume2 size={20} /></div>
          <span className="section-kicker">MASTER VOLUME</span>
          <div className="volume-number">{Math.round(settings.effects.masterVolume * 100)}<small>%</small></div>
          <input aria-label="Master volume" type="range" min="0" max="1" step="0.01" value={settings.effects.masterVolume} style={{ "--range-progress": `${settings.effects.masterVolume * 100}%` } as React.CSSProperties} onChange={(event) => onVolume(Number(event.currentTarget.value))} />
          <button className="quiet-button" onClick={onTest}>Test sound</button>
        </section>

        <button className="preset-card" onClick={() => onNavigate("lab")}>
          <div><span className="section-kicker">CURRENT PRESET</span><h3>{settings.activePreset ?? "Custom settings"}</h3><p>{pack?.name} · Spatial {Math.round(settings.effects.spatial * 100)}%</p></div>
          <span className="round-arrow"><SlidersHorizontal size={19} /></span>
        </button>
      </div>
      <p className="privacy-note"><span /> Individual key events become sound locally. Nothing is recorded or sent.</p>
    </div>
  );
}
