import { useState } from "react";
import { Check, Play } from "lucide-react";
import type { PackInfo } from "../types";
import { MechanicalSwitch } from "./MechanicalSwitch";

interface PackCardProps {
  pack: PackInfo;
  index: number;
  active: boolean;
  onActivate: (id: string) => void;
  onPreview: (id: string) => void;
}

export function PackCard({ pack, index, active, onActivate, onPreview }: PackCardProps) {
  const [strike, setStrike] = useState(0);
  const preview = () => {
    setStrike((current) => current + 1);
    onPreview(pack.id);
  };
  const switchType = pack.tags.find((tag) => ["linear", "tactile", "clicky"].includes(tag.toLowerCase())) ?? "mechanical";

  return (
    <article className={`pack-card ${active ? "active" : ""}`}>
      <button className={`pack-hero pack-${pack.id}`} onClick={preview} aria-label={`Preview ${pack.name} sound`}>
        <span className="pack-index">{String(index + 1).padStart(2, "0")}</span>
        <span className="switch-type">{switchType}</span>
        <MechanicalSwitch packId={pack.id} strike={strike} />
        <span className="switch-preview-hint"><Play size={9} fill="currentColor" /> Press to preview</span>
      </button>
      <div className="pack-content">
        <div className="pack-title"><div><h2>{pack.name}</h2><p>by {pack.author}</p></div>{active && <span className="active-chip"><Check size={12} /> Active</span>}</div>
        <p className="pack-description">{pack.description}</p>
        <div className="tag-row">{pack.tags.map((tag) => <span key={tag}>{tag}</span>)}</div>
        <div className="pack-meta"><span>{pack.license}</span><span>{pack.hasReleaseSamples ? "Press + release" : "Press only"}</span></div>
        <div className="pack-actions"><button className="preview-button" onClick={preview} aria-label={`Preview ${pack.name}`}><Play size={15} fill="currentColor" /> Preview</button><button className="activate-button" disabled={active} onClick={() => onActivate(pack.id)}>{active ? "In use" : "Activate"}</button></div>
      </div>
    </article>
  );
}
