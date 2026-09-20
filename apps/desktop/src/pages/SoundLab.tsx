import { Copy, RotateCcw, Save, Sparkles } from "lucide-react";
import { RangeControl } from "../components/RangeControl";
import { SpatialKeyboard } from "../components/SpatialKeyboard";
import type { AppSnapshot, Effects } from "../types";

interface SoundLabProps {
  snapshot: AppSnapshot;
  effects: Effects;
  setEffects: (effects: Effects) => void;
  commitEffects: (effects: Effects) => void;
  onReset: () => void;
  onSave: () => void;
  onDuplicate: () => void;
  onLoadPreset: (name: string) => void;
}

const percent = (value: number) => `${Math.round(value * 100)}%`;
const signedPercent = (value: number) => `±${Math.round(value * 100)}%`;
const decibels = (value: number) => `${value > 0 ? "+" : ""}${value.toFixed(1)} dB`;
const semitones = (value: number) => `${value > 0 ? "+" : ""}${value.toFixed(1)} st`;

export function SoundLab({ snapshot, effects, setEffects, commitEffects, onReset, onSave, onDuplicate, onLoadPreset }: SoundLabProps) {
  const update = (key: keyof Effects, value: number) => {
    const next = { ...effects, [key]: value };
    setEffects(next);
  };
  return (
    <div className="page lab-page">
      <header className="page-header compact">
        <div><p className="eyebrow">KEYTONE / SOUND LAB</p><h1>Shape the <em>feel.</em></h1></div>
        <div className="header-actions">
          <button className="secondary-button" onClick={onReset}><RotateCcw size={16} /> Reset</button>
          <button className="primary-button" onClick={onSave}><Save size={16} /> Save preset</button>
        </div>
      </header>
      <div className="lab-status">
        <Sparkles size={15} /><span>Changes are live</span><i />
        <label className="preset-picker">
          <span>Preset</span>
          <select value={snapshot.settings.activePreset ?? ""} onChange={(event) => onLoadPreset(event.currentTarget.value)}>
            <option value="" disabled>Unsaved custom preset</option>
            {snapshot.presets.map((preset) => <option key={preset.name} value={preset.name}>{preset.name}</option>)}
          </select>
        </label>
      </div>
      <div className="lab-grid">
        <section className="control-panel master-panel">
          <div className="panel-heading"><span>01</span><div><h2>Voice</h2><p>Level, tuning and human variation</p></div></div>
          <RangeControl label="Master" value={effects.masterVolume} min={0} max={1} step={0.01} format={percent} onChange={(v) => update("masterVolume", v)} />
          <RangeControl label="Pitch" value={effects.pitch} min={-12} max={12} step={0.1} format={semitones} onChange={(v) => update("pitch", v)} />
          <RangeControl label="Pitch randomness" value={effects.pitchRandomness} min={0} max={0.12} step={0.005} format={signedPercent} onChange={(v) => update("pitchRandomness", v)} />
          <RangeControl label="Volume randomness" value={effects.volumeRandomness} min={0} max={0.2} step={0.005} format={signedPercent} onChange={(v) => update("volumeRandomness", v)} />
        </section>
        <section className="control-panel tone-panel">
          <div className="panel-heading"><span>02</span><div><h2>Tone</h2><p>Simple musical shelves</p></div></div>
          <div className="eq-visual"><span style={{ height: `${46 + effects.bass * 2}%` }} /><span style={{ height: "48%" }} /><span style={{ height: `${46 + effects.treble * 2}%` }} /></div>
          <RangeControl label="Bass" value={effects.bass} min={-12} max={12} step={0.5} format={decibels} onChange={(v) => update("bass", v)} />
          <RangeControl label="Treble" value={effects.treble} min={-12} max={12} step={0.5} format={decibels} onChange={(v) => update("treble", v)} />
          <RangeControl label="Room" value={effects.reverb} min={0} max={0.5} step={0.01} format={percent} onChange={(v) => update("reverb", v)} />
        </section>
        <section className="control-panel spatial-panel">
          <div className="panel-heading"><span>03</span><div><h2>Spatial field</h2><p>Place every key across the stereo image</p></div></div>
          <SpatialKeyboard strength={effects.spatial} />
          <RangeControl label="Spatial strength" value={effects.spatial} min={0} max={1} step={0.01} format={percent} onChange={(v) => update("spatial", v)} />
        </section>
      </div>
      <footer className="lab-footer"><button className="quiet-button" onClick={onDuplicate}><Copy size={15} /> Duplicate preset</button><button className="apply-button" onClick={() => commitEffects(effects)}>Apply now</button></footer>
    </div>
  );
}
