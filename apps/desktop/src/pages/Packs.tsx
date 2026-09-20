import { Check, FolderPlus, Play, Waves } from "lucide-react";
import type { AppSnapshot } from "../types";

interface PacksProps {
  snapshot: AppSnapshot;
  onActivate: (id: string) => void;
  onPreview: (id: string) => void;
  onImportFolder: () => void;
  onImportManifest: () => void;
}

export function Packs({ snapshot, onActivate, onPreview, onImportFolder, onImportManifest }: PacksProps) {
  return (
    <div className="page packs-page">
      <header className="page-header compact">
        <div><p className="eyebrow">KEYTONE / LIBRARY</p><h1>Your <em>sound.</em></h1></div>
        <div className="header-actions">
          <button className="secondary-button" onClick={onImportManifest}>Choose manifest</button>
          <button className="primary-button" onClick={onImportFolder}><FolderPlus size={17} /> Import folder</button>
        </div>
      </header>
      <div className="library-summary"><span>{snapshot.packs.length} installed packs</span><p>Local files only · Keytone Sound Pack v1</p></div>
      <div className="pack-grid">
        {snapshot.packs.map((pack, index) => {
          const active = pack.id === snapshot.settings.activePack;
          return (
            <article className={`pack-card ${active ? "active" : ""}`} key={pack.id}>
              <div className={`pack-hero pack-${pack.id}`}><span className="pack-index">0{index + 1}</span><Waves size={42} strokeWidth={1} /></div>
              <div className="pack-content">
                <div className="pack-title"><div><h2>{pack.name}</h2><p>by {pack.author}</p></div>{active && <span className="active-chip"><Check size={12} /> Active</span>}</div>
                <p className="pack-description">{pack.description}</p>
                <div className="tag-row">{pack.tags.map((tag) => <span key={tag}>{tag}</span>)}</div>
                <div className="pack-meta"><span>{pack.license}</span><span>{pack.hasReleaseSamples ? "Press + release" : "Press only"}</span></div>
                <div className="pack-actions"><button className="preview-button" onClick={() => onPreview(pack.id)}><Play size={15} fill="currentColor" /> Preview</button><button className="activate-button" disabled={active} onClick={() => onActivate(pack.id)}>{active ? "In use" : "Activate"}</button></div>
              </div>
            </article>
          );
        })}
      </div>
    </div>
  );
}
