import { FolderPlus } from "lucide-react";
import { PackCard } from "../components/PackCard";
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
        {snapshot.packs.map((pack, index) => (
          <PackCard key={pack.id} pack={pack} index={index} active={pack.id === snapshot.settings.activePack} onActivate={onActivate} onPreview={onPreview} />
        ))}
      </div>
    </div>
  );
}
