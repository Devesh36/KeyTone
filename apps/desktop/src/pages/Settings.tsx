import { ExternalLink, Headphones, LockKeyhole, MonitorSpeaker } from "lucide-react";
import { Toggle } from "../components/Toggle";
import type { AppSnapshot } from "../types";

interface SettingsProps {
  snapshot: AppSnapshot;
  onPreferences: (release: boolean, repeats: boolean, startup: boolean, keepRunning: boolean) => void;
  onOutput: (name: string | null) => void;
  onKeyboardAccess: () => void;
}

export function Settings({ snapshot, onPreferences, onOutput, onKeyboardAccess }: SettingsProps) {
  const s = snapshot.settings;
  const preference = (patch: Partial<Pick<typeof s, "releaseSounds" | "playRepeats" | "launchAtStartup" | "keepRunningOnClose">>) => {
    onPreferences(patch.releaseSounds ?? s.releaseSounds, patch.playRepeats ?? s.playRepeats, patch.launchAtStartup ?? s.launchAtStartup, patch.keepRunningOnClose ?? s.keepRunningOnClose);
  };
  return (
    <div className="page settings-page">
      <header className="page-header compact"><div><p className="eyebrow">KEYTONE / SETTINGS</p><h1>Quietly in <em>control.</em></h1></div></header>
      {snapshot.warnings.map((warning) => <div className="warning-banner" key={warning}>{warning}</div>)}
      <div className="settings-stack">
        <section className="settings-section">
          <div className="settings-heading"><Headphones size={20} /><div><h2>Playback</h2><p>Audio behavior and output</p></div></div>
          <label className="select-row"><span><strong>Output device</strong><small>Restart the stream on another device</small></span><select value={s.outputDevice ?? ""} onChange={(event) => onOutput(event.target.value || null)}><option value="">System default</option>{snapshot.outputDevices.map((device) => <option key={device}>{device}</option>)}</select></label>
          <SettingToggle title="Release sounds" detail="Play a separate sample when a key lifts" value={s.releaseSounds} onChange={(v) => preference({ releaseSounds: v })} />
          <SettingToggle title="OS key repeat" detail="Off prevents held keys from machine-gunning" value={s.playRepeats} onChange={(v) => preference({ playRepeats: v })} />
        </section>
        <section className="settings-section">
          <div className="settings-heading"><MonitorSpeaker size={20} /><div><h2>Application</h2><p>How Keytone behaves on your desktop</p></div></div>
          <SettingToggle title="Keep running when closed" detail="Close hides the window; Quit exits fully" value={s.keepRunningOnClose} onChange={(v) => preference({ keepRunningOnClose: v })} />
          <SettingToggle title="Launch at startup" detail="Preference saved; OS registration is planned" value={s.launchAtStartup} onChange={(v) => preference({ launchAtStartup: v })} />
        </section>
        <section className="settings-section privacy-section">
          <div className="settings-heading"><LockKeyhole size={20} /><div><h2>Keyboard access & privacy</h2><p>Status: <strong>{snapshot.permission}</strong></p></div></div>
          <p>Keytone receives individual physical key press/release events and immediately turns them into audio triggers. It never builds words, stores key history, sends events, or uses a network service.</p>
          <div className={`permission-help permission-${snapshot.permission}`}>
            <span>{snapshot.permissionInstructions}</span>
            {snapshot.permission === "missing" ? (
              <button type="button" onClick={onKeyboardAccess}>Open Input Monitoring <ExternalLink size={14} /></button>
            ) : (
              <span className="permission-ready">Keyboard listening is ready</span>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}

function SettingToggle({ title, detail, value, onChange }: { title: string; detail: string; value: boolean; onChange: (value: boolean) => void }) {
  return <div className="setting-row"><span><strong>{title}</strong><small>{detail}</small></span><Toggle checked={value} onChange={onChange} label={title} /></div>;
}
