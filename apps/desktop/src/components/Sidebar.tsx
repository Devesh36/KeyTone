import { AudioLines, FlaskConical, Library, Settings } from "lucide-react";
import type { Page } from "../types";

const items = [
  { id: "home" as const, label: "Listen", icon: AudioLines },
  { id: "lab" as const, label: "Sound Lab", icon: FlaskConical },
  { id: "packs" as const, label: "Packs", icon: Library },
  { id: "settings" as const, label: "Settings", icon: Settings },
];

export function Sidebar({ page, onNavigate }: { page: Page; onNavigate: (page: Page) => void }) {
  return (
    <aside className="sidebar">
      <div className="brand-mark" aria-label="Keytone"><span>K</span></div>
      <nav aria-label="Primary navigation">
        {items.map(({ id, label, icon: Icon }) => (
          <button key={id} className={page === id ? "active" : ""} onClick={() => onNavigate(id)} title={label}>
            <Icon size={19} strokeWidth={1.7} />
            <span>{label}</span>
          </button>
        ))}
      </nav>
      <div className="privacy-dot" title="Local only"><span /> Local</div>
    </aside>
  );
}
