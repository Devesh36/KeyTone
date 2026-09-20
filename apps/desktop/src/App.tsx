import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Sidebar } from "./components/Sidebar";
import { useKeytone } from "./hooks/useKeytone";
import { api } from "./lib/api";
import { Home } from "./pages/Home";
import { Packs } from "./pages/Packs";
import { Settings } from "./pages/Settings";
import { SoundLab } from "./pages/SoundLab";
import type { Effects, Page } from "./types";

export default function App() {
  const [page, setPage] = useState<Page>("home");
  const { snapshot, setSnapshot, error, setError, busy, act, updateEffects } = useKeytone();
  const [draft, setDraft] = useState<Effects | null>(null);

  useEffect(() => {
    if (snapshot && !draft) setDraft(snapshot.settings.effects);
  }, [snapshot, draft]);

  if (!snapshot || !draft) {
    return <div className="loading"><div className="loading-mark">K</div><p>Starting the audio engine…</p></div>;
  }

  const commitEffects = async (effects: Effects) => {
    const next = await updateEffects(effects);
    setDraft(next.settings.effects);
  };

  const importPack = async (directory: boolean) => {
    if (!api.isTauri()) {
      setError("Pack import is available in the desktop application.");
      return;
    }
    const selected = await open({
      directory,
      multiple: false,
      title: directory ? "Choose a Keytone sound-pack folder" : "Choose a Keytone manifest.json",
      filters: directory ? undefined : [{ name: "Keytone manifest", extensions: ["json"] }],
    });
    if (typeof selected === "string") await act(() => api.importPack(selected));
  };

  const savePreset = async () => {
    const name = window.prompt("Preset name", snapshot.settings.activePreset ?? "My preset");
    if (name?.trim()) {
      await commitEffects(draft);
      await act(() => api.savePreset(name.trim()));
    }
  };

  const content = {
    home: <Home snapshot={snapshot} busy={busy} onEngine={(enabled) => void act(() => api.setEngine(enabled))} onVolume={(masterVolume) => { const next = { ...draft, masterVolume }; setDraft(next); void commitEffects(next); }} onNavigate={setPage} onTest={() => void api.testSound()} />,
    lab: <SoundLab snapshot={snapshot} effects={draft} setEffects={(next) => { setDraft(next); void api.updateEffects(next).then(setSnapshot); }} commitEffects={(next) => void commitEffects(next)} onReset={() => void act(() => api.resetEffects()).then((next) => setDraft(next.settings.effects))} onSave={() => void savePreset()} onDuplicate={() => { const name = snapshot.settings.activePreset; if (name) void act(() => api.duplicatePreset(name)); }} onLoadPreset={(name) => void act(() => api.loadPreset(name)).then((next) => setDraft(next.settings.effects))} />,
    packs: <Packs snapshot={snapshot} onActivate={(id) => void act(() => api.activatePack(id))} onPreview={(id) => void api.previewPack(id)} onImportFolder={() => void importPack(true)} onImportManifest={() => void importPack(false)} />,
    settings: <Settings snapshot={snapshot} onPreferences={(release, repeats, startup, keepRunning) => void act(() => api.updatePreferences(release, repeats, startup, keepRunning))} onOutput={(name) => void act(() => api.selectOutputDevice(name))} onKeyboardAccess={() => void act(() => api.openKeyboardSettings())} />,
  }[page];

  return (
    <div className="app-shell">
      <div className="titlebar" data-tauri-drag-region><span>KEYTONE</span><i data-tauri-drag-region /></div>
      <Sidebar page={page} onNavigate={setPage} />
      <main>{content}</main>
      {error && <div className="toast" role="alert"><span>{error}</span><button onClick={() => setError(null)}>Dismiss</button></div>}
    </div>
  );
}
