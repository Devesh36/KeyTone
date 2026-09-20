import { useEffect, useMemo, useState, type CSSProperties } from "react";
import {
  Apple,
  ArrowDown,
  ArrowRight,
  ArrowUpRight,
  AudioLines,
  Check,
  Download,
  Github,
  Laptop,
  Layers3,
  Menu,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Terminal,
  Waves,
  X,
  Zap,
  type LucideIcon,
} from "lucide-react";

const REPOSITORY_URL = "https://github.com/Devesh36/KeyTone";
const RELEASES_URL = `${REPOSITORY_URL}/releases/latest`;
const RELEASE_API_URL = "https://api.github.com/repos/Devesh36/KeyTone/releases/latest";

type Platform = "macos" | "windows" | "linux";

type DownloadOption = {
  platform: Platform;
  title: string;
  detail: string;
  extension: string;
  icon: LucideIcon;
};

type ReleaseAsset = {
  name: string;
  browser_download_url: string;
  size: number;
};

type ReleaseResponse = {
  tag_name: string;
  assets: ReleaseAsset[];
};

const downloadOptions: DownloadOption[] = [
  {
    platform: "macos",
    title: "macOS",
    detail: "macOS 11 or newer",
    extension: ".dmg · Universal",
    icon: Apple,
  },
  {
    platform: "windows",
    title: "Windows",
    detail: "Windows 10 or newer",
    extension: ".msi · 64-bit",
    icon: Laptop,
  },
  {
    platform: "linux",
    title: "Linux",
    detail: "Modern 64-bit distributions",
    extension: ".AppImage · x64",
    icon: Terminal,
  },
];

const features = [
  {
    icon: Zap,
    number: "01",
    title: "Feels immediate",
    copy: "A Rust audio path, preloaded samples, and a lock-free trigger queue keep sound close to the keystroke.",
  },
  {
    icon: Layers3,
    number: "02",
    title: "Never robotic",
    copy: "Layered press and release samples gain tiny pitch and volume variations with every stroke.",
  },
  {
    icon: Waves,
    number: "03",
    title: "Placed in space",
    copy: "Q sits left, P sits right, and every key finds its natural place in a restrained stereo field.",
  },
  {
    icon: SlidersHorizontal,
    number: "04",
    title: "Tuned by you",
    copy: "Shape pitch, bass, treble, room, randomness, and spatial strength while you type.",
  },
];

const keyboardRows = [
  ["esc", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "−", "⌫"],
  ["tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "[", "]"],
  ["caps", "A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "↵"],
  ["shift", "Z", "X", "C", "V", "B", "N", "M", ",", ".", "/", "shift"],
  ["fn", "ctrl", "opt", "cmd", "space", "cmd", "opt", "←", "↑", "→"],
];

const waveHeights = [28, 46, 35, 68, 52, 84, 42, 64, 92, 58, 74, 38, 80, 48, 65, 31, 54, 72, 44, 88, 60, 35, 68, 49, 78, 41, 57, 32];

function detectPlatform(): Platform {
  if (typeof navigator === "undefined") return "macos";
  const descriptor = `${navigator.platform} ${navigator.userAgent}`.toLowerCase();
  if (descriptor.includes("win")) return "windows";
  if (descriptor.includes("linux") || descriptor.includes("x11")) return "linux";
  return "macos";
}

function pickAsset(platform: Platform, assets: ReleaseAsset[]): ReleaseAsset | undefined {
  const scored = assets
    .map((asset) => {
      const name = asset.name.toLowerCase();
      let score = 0;
      if (platform === "macos" && name.endsWith(".dmg")) score = 10;
      if (platform === "windows" && name.endsWith(".msi")) score = 10;
      if (platform === "windows" && name.endsWith(".exe")) score = 8;
      if (platform === "linux" && name.endsWith(".appimage")) score = 10;
      if (platform === "linux" && name.endsWith(".deb")) score = 8;
      if (platform === "macos" && name.includes("universal")) score += 4;
      if (name.includes("x64") || name.includes("amd64") || name.includes("universal")) score += 1;
      return { asset, score };
    })
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score);
  return scored[0]?.asset;
}

function formatBytes(bytes: number): string {
  return `${Math.max(1, Math.round(bytes / 1_048_576))} MB`;
}

function Logo() {
  return (
    <a className="brand" href="#top" aria-label="Keytone home">
      <span className="brand-mark" aria-hidden="true">
        <span>K</span>
      </span>
      <span>KEYTONE</span>
    </a>
  );
}

function KeyboardVisual() {
  return (
    <div className="instrument" aria-label="A keyboard visual showing spatial audio activity">
      <div className="instrument-head">
        <div>
          <span className="eyebrow">LIVE ENGINE</span>
          <p>Spatial field</p>
        </div>
        <span className="engine-state"><i /> Active</span>
      </div>
      <div className="waveform" aria-hidden="true">
        {waveHeights.map((height, index) => (
          <i key={`${height}-${index}`} style={{ "--height": `${height}%`, "--delay": `${index * -48}ms` } as CSSProperties} />
        ))}
      </div>
      <div className="keyboard" aria-hidden="true">
        {keyboardRows.map((row, rowIndex) => (
          <div className={`key-row key-row-${rowIndex}`} key={row.join("-")}>
            {row.map((key, keyIndex) => (
              <span
                className={`key ${key.length > 2 ? "key-wide" : ""} ${key === "space" ? "key-space" : ""}`}
                key={`${key}-${keyIndex}`}
              >
                {key}
              </span>
            ))}
          </div>
        ))}
      </div>
      <div className="stereo-scale" aria-hidden="true">
        <span>L</span><i /><strong>SPATIAL 72%</strong><i /><span>R</span>
      </div>
    </div>
  );
}

function App() {
  const [menuOpen, setMenuOpen] = useState(false);
  const [release, setRelease] = useState<ReleaseResponse | null>(null);
  const platform = useMemo(detectPlatform, []);

  useEffect(() => {
    const controller = new AbortController();
    fetch(RELEASE_API_URL, {
      signal: controller.signal,
      headers: { Accept: "application/vnd.github+json" },
    })
      .then((response) => (response.ok ? response.json() : Promise.reject(new Error("No release"))))
      .then((data: ReleaseResponse) => setRelease(data))
      .catch(() => setRelease(null));
    return () => controller.abort();
  }, []);

  const platformAsset = release ? pickAsset(platform, release.assets) : undefined;
  const platformName = downloadOptions.find((option) => option.platform === platform)?.title ?? "your OS";

  return (
    <div className="site-shell" id="top">
      <header className="site-header">
        <div className="nav-wrap">
          <Logo />
          <nav className={menuOpen ? "nav-links nav-links-open" : "nav-links"} aria-label="Main navigation">
            <a href="#features" onClick={() => setMenuOpen(false)}>Features</a>
            <a href="#sound-packs" onClick={() => setMenuOpen(false)}>Sound packs</a>
            <a href="#privacy" onClick={() => setMenuOpen(false)}>Privacy</a>
            <a href={`${REPOSITORY_URL}#readme`} onClick={() => setMenuOpen(false)}>Docs</a>
          </nav>
          <div className="nav-actions">
            <a className="github-link" href={REPOSITORY_URL} target="_blank" rel="noreferrer" aria-label="Keytone on GitHub">
              <Github size={18} />
              <span>GitHub</span>
            </a>
            <a className="nav-download" href="#download">Download</a>
            <button className="menu-button" type="button" aria-label="Toggle menu" aria-expanded={menuOpen} onClick={() => setMenuOpen((open) => !open)}>
              {menuOpen ? <X /> : <Menu />}
            </button>
          </div>
        </div>
      </header>

      <main>
        <section className="hero page-section">
          <div className="hero-copy">
            <div className="hero-kicker"><AudioLines size={15} /> Local-first keyboard audio</div>
            <h1>Make every<br />keystroke<br /><em>sound yours.</em></h1>
            <p className="hero-summary">
              A low-latency mechanical keyboard audio engine with spatial sound, expressive variation, and a studio of real-time controls.
            </p>
            <div className="hero-actions">
              <a className="button button-primary" href={platformAsset?.browser_download_url ?? "#download"}>
                <Download size={18} />
                Download for {platformName}
              </a>
              <a className="button button-secondary" href={REPOSITORY_URL} target="_blank" rel="noreferrer">
                <Github size={18} /> View source
              </a>
            </div>
            <div className="hero-meta">
              <span><Check size={14} /> Free & open source</span>
              <span><Check size={14} /> No account</span>
              <span><Check size={14} /> No telemetry</span>
            </div>
          </div>
          <div className="hero-visual">
            <KeyboardVisual />
            <span className="visual-note note-top">64 voice polyphony <ArrowRight size={13} /></span>
            <span className="visual-note note-bottom"><ArrowRight size={13} /> Press + release layers</span>
          </div>
          <a className="scroll-cue" href="#features"><ArrowDown size={16} /> Explore the engine</a>
        </section>

        <section className="proof-strip" aria-label="Keytone highlights">
          <div><strong>&lt;10ms</strong><span>Latency target</span></div>
          <div><strong>64</strong><span>Overlapping voices</span></div>
          <div><strong>19</strong><span>Included sound packs</span></div>
          <div><strong>100%</strong><span>Local processing</span></div>
        </section>

        <section className="features page-section" id="features">
          <div className="section-heading">
            <div>
              <span className="eyebrow">DESIGNED FOR FLOW</span>
              <h2>Your keyboard,<br />now an instrument.</h2>
            </div>
            <p>Every detail is built to disappear into your typing—from the native input listener to the last room reflection.</p>
          </div>
          <div className="feature-grid">
            {features.map(({ icon: Icon, number, title, copy }) => (
              <article className="feature-card" key={number}>
                <div className="feature-top"><span>{number}</span><Icon size={23} /></div>
                <h3>{title}</h3>
                <p>{copy}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="sound-lab page-section">
          <div className="sound-lab-panel">
            <div className="lab-copy">
              <span className="eyebrow">SOUND LAB</span>
              <h2>Dial in your perfect feel.</h2>
              <p>Warm it up. Tighten the room. Add a hint of pitch drift. Every control updates the native audio engine as you type.</p>
              <ul>
                <li><Check size={15} /> Pitch and variation</li>
                <li><Check size={15} /> Bass and treble shaping</li>
                <li><Check size={15} /> Room and spatial strength</li>
                <li><Check size={15} /> Shareable presets</li>
              </ul>
            </div>
            <div className="mixer" aria-label="Sound Lab control preview">
              <div className="mixer-head"><span>Late Night Cream</span><span className="mixer-status"><i /> LIVE</span></div>
              {[
                ["MASTER", "82%", "82%"],
                ["PITCH", "+1.2%", "58%"],
                ["BASS", "+3.0 dB", "68%"],
                ["ROOM", "12%", "28%"],
                ["SPATIAL", "72%", "72%"],
              ].map(([label, value, width]) => (
                <div className="mixer-row" key={label}>
                  <div><span>{label}</span><strong>{value}</strong></div>
                  <div className="mixer-track"><i style={{ width }}><b /></i></div>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="packs page-section" id="sound-packs">
          <div className="section-heading packs-heading">
            <div>
              <span className="eyebrow">INCLUDED SOUND PACKS</span>
              <h2>Change switches<br />without changing switches.</h2>
            </div>
            <p>Start with nineteen distinct profiles—from deep linear warmth to crisp, unapologetic clicks.</p>
          </div>
          <div className="pack-list">
            <article className="pack-card pack-cream">
              <div className="pack-number">01</div><Sparkles />
              <div><h3>Cream</h3><p>Soft · Linear · Warm</p></div>
              <div className="pack-wave"><i /><i /><i /><i /><i /><i /><i /></div>
            </article>
            <article className="pack-card pack-panda">
              <div className="pack-number">02</div><Sparkles />
              <div><h3>Holy Panda</h3><p>Round · Tactile · Full</p></div>
              <div className="pack-wave"><i /><i /><i /><i /><i /><i /><i /></div>
            </article>
            <article className="pack-card pack-navy">
              <div className="pack-number">03</div><Sparkles />
              <div><h3>Box Navy</h3><p>Sharp · Clicky · Bright</p></div>
              <div className="pack-wave"><i /><i /><i /><i /><i /><i /><i /></div>
            </article>
          </div>
          <p className="pack-footnote">Plus Alpaca, Topre, Blue Alps, MX Black, MX Blue, MX Brown, and more.</p>
        </section>

        <section className="privacy page-section" id="privacy">
          <div className="privacy-mark"><ShieldCheck /></div>
          <span className="eyebrow">PRIVATE BY DESIGN</span>
          <h2>Your keys stay<br />between you and your keyboard.</h2>
          <p>Keytone reacts to one physical key event at a time, schedules its sound, and immediately discards it. It never constructs words, stores key history, calls a server, or tracks you.</p>
          <div className="privacy-points">
            <span><Check /> No telemetry</span>
            <span><Check /> No network required</span>
            <span><Check /> No keyboard history</span>
            <span><Check /> Open-source code</span>
          </div>
        </section>

        <section className="download-section page-section" id="download">
          <div className="download-intro">
            <span className="eyebrow">KEYTONE v{release?.tag_name.replace(/^v/, "") ?? "0.1"}</span>
            <h2>Ready when<br />your fingers are.</h2>
            <p>Free, open source, and built to stay out of your way.</p>
          </div>
          <div className="download-grid">
            {downloadOptions.map((option) => {
              const asset = release ? pickAsset(option.platform, release.assets) : undefined;
              const Icon = option.icon;
              return (
                <a className={`download-card ${option.platform === platform ? "download-card-featured" : ""}`} href={asset?.browser_download_url ?? RELEASES_URL} key={option.platform}>
                  {option.platform === platform && <span className="recommended">YOUR DEVICE</span>}
                  <Icon size={31} />
                  <div>
                    <span>Download for</span>
                    <h3>{option.title}</h3>
                    <p>{option.detail}</p>
                  </div>
                  <div className="download-card-foot">
                    <span>{asset ? `${formatBytes(asset.size)} · ${option.extension.split(" · ")[0]}` : option.extension}</span>
                    <Download size={18} />
                  </div>
                </a>
              );
            })}
          </div>
          <p className="release-note">
            {release ? `Latest release: ${release.tag_name}` : "Installers are published on GitHub Releases."}
            {" · "}<a href={`${REPOSITORY_URL}#installation`}>Build from source <ArrowUpRight size={12} /></a>
          </p>
        </section>
      </main>

      <footer>
        <div className="footer-top">
          <Logo />
          <p>Make every keystroke sound yours.</p>
          <div className="footer-links">
            <a href={REPOSITORY_URL}>GitHub <ArrowUpRight /></a>
            <a href={`${REPOSITORY_URL}/blob/main/docs/privacy.md`}>Privacy <ArrowUpRight /></a>
            <a href={`${REPOSITORY_URL}/blob/main/LICENSE`}>MIT License <ArrowUpRight /></a>
          </div>
        </div>
        <div className="footer-bottom"><span>© 2026 Keytone contributors</span><span>Built locally. Played instantly.</span></div>
      </footer>
    </div>
  );
}

export default App;
