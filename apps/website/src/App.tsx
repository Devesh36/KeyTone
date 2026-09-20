import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { SwitchDemo } from "./components/SwitchDemo";
import { demoProfiles } from "./lib/demo";
import { detectPlatform, isRelease, pickAsset, type Platform, type ReleaseResponse } from "./lib/downloads";
import {
  Apple,
  ArrowDown,
  ArrowUpRight,
  AudioLines,
  Check,
  CircleAlert,
  Download,
  Github,
  Laptop,
  Layers3,
  Menu,
  ShieldCheck,
  SlidersHorizontal,
  Terminal,
  Waves,
  X,
  Zap,
  type LucideIcon,
} from "lucide-react";

const REPOSITORY_URL = "https://github.com/Devesh36/KeyTone";
const RELEASES_URL = `${REPOSITORY_URL}/releases/latest`;
const RELEASE_API_URL = "https://api.github.com/repos/Devesh36/KeyTone/releases/latest";

type DownloadOption = {
  platform: Platform;
  title: string;
  detail: string;
  extension: string;
  icon: LucideIcon;
};

const downloadOptions: DownloadOption[] = [
  {
    platform: "macos",
    title: "macOS",
    detail: "macOS 11+ · Open Anyway required",
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

function App() {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuButton = useRef<HTMLButtonElement>(null);
  const header = useRef<HTMLElement>(null);
  const [demoProfile, setDemoProfile] = useState(0);
  const [release, setRelease] = useState<ReleaseResponse | null>(null);
  const platform = useMemo(() => typeof window === "undefined" || typeof navigator === "undefined" ? null : detectPlatform(navigator.userAgent, navigator.platform, navigator.maxTouchPoints), []);

  useEffect(() => {
    if (!menuOpen) return;
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") { setMenuOpen(false); menuButton.current?.focus(); }
    };
    const resize = () => { if (window.innerWidth > 960) setMenuOpen(false); };
    const outside = (event: PointerEvent) => {
      if (event.target instanceof Node && !header.current?.contains(event.target)) setMenuOpen(false);
    };
    window.addEventListener("keydown", escape);
    window.addEventListener("resize", resize);
    document.addEventListener("pointerdown", outside);
    return () => { window.removeEventListener("keydown", escape); window.removeEventListener("resize", resize); document.removeEventListener("pointerdown", outside); };
  }, [menuOpen]);

  useEffect(() => {
    const controller = new AbortController();
    fetch(RELEASE_API_URL, {
      signal: controller.signal,
      headers: { Accept: "application/vnd.github+json" },
    })
      .then((response) => (response.ok ? response.json() : Promise.reject(new Error("No release"))))
      .then((data: unknown) => { if (!controller.signal.aborted) setRelease(isRelease(data) ? data : null); })
      .catch(() => { if (!controller.signal.aborted) setRelease(null); });
    return () => controller.abort();
  }, []);

  const platformAsset = release && platform ? pickAsset(platform, release.assets) : undefined;
  const platformName = downloadOptions.find((option) => option.platform === platform)?.title ?? "your OS";

  return (
    <div className="site-shell" id="top">
      <a className="skip-link" href="#main-content">Skip to content</a>
      <header ref={header} className="site-header" onBlur={(event) => { if (event.relatedTarget instanceof Node && !event.currentTarget.contains(event.relatedTarget)) setMenuOpen(false); }}>
        <div className="nav-wrap">
          <Logo />
          <nav id="main-navigation" className={menuOpen ? "nav-links nav-links-open" : "nav-links"} aria-label="Main navigation">
            <a href="#demo" onClick={() => setMenuOpen(false)}>Try it</a>
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
            <a className="nav-download" href="#download" onClick={() => setMenuOpen(false)}>Download</a>
            <button ref={menuButton} className="menu-button" type="button" aria-label={menuOpen ? "Close menu" : "Open menu"} aria-controls="main-navigation" aria-expanded={menuOpen} onClick={() => setMenuOpen((open) => !open)}>
              {menuOpen ? <X /> : <Menu />}
            </button>
          </div>
        </div>
      </header>

      <main id="main-content" tabIndex={-1}>
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
                {platform ? `Download for ${platformName}` : "Get the desktop app"}
              </a>
              <a className="button button-secondary" href="#demo">
                <AudioLines size={18} /> Try the switches
              </a>
            </div>
            <p className="platform-note">For macOS, Windows &amp; Linux. No account. Just sound.</p>
            <div className="hero-meta">
              <span><Check size={14} /> Free & open source</span>
              <span><Check size={14} /> No account</span>
              <span><Check size={14} /> No telemetry</span>
            </div>
          </div>
          <div className="hero-visual">
            <SwitchDemo profileIndex={demoProfile} onProfileChange={setDemoProfile} />
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
              <div className="mixer-head"><span>Late Night Cream</span><span className="mixer-status">APP PREVIEW</span></div>
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
            {demoProfiles.map((profile, index) => (
              <a className={`pack-card pack-${profile.id}`} href="#demo" key={profile.id} style={{ "--pack-color": profile.color } as CSSProperties} onClick={() => { setDemoProfile(index); document.getElementById("demo")?.focus({ preventScroll: true }); }} aria-label={`Try ${profile.name} in the keyboard demo`}>
                <span className="pack-number">0{index + 1} / SWITCH PROFILE</span>
                <span className="pack-keycap" aria-hidden="true"><span>{["C", "A", "N"][index]}</span></span>
                <div><h3>{profile.name}</h3><p>{profile.character}</p></div>
                <span className="pack-try">Try this switch <ArrowUpRight size={17} /></span>
              </a>
            ))}
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
            <p>Free, open source, and built to stay out of your way.<br />A desktop app for macOS, Windows, and Linux—not iOS or Android.</p>
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
          <aside className="mac-install-note" aria-labelledby="mac-install-title">
            <CircleAlert aria-hidden="true" />
            <div>
              <span className="eyebrow">MACOS INSTALL NOTE</span>
              <h3 id="mac-install-title">Seeing “Apple could not verify Keytone”?</h3>
              <p>
                The current v0.1 build is ad-hoc signed and not yet Apple-notarized. If you downloaded Keytone from this official site, macOS lets you approve it once:
              </p>
              <ol>
                <li>Try to open Keytone, then choose <strong>Done</strong>—not Move to Bin.</li>
                <li>Open <strong>System Settings → Privacy &amp; Security</strong>.</li>
                <li>Scroll to Security and click <strong>Open Anyway</strong> beside Keytone.</li>
                <li>Authenticate, choose <strong>Open</strong>, then enable Keytone under Input Monitoring.</li>
              </ol>
              <p className="mac-install-safety">
                Only approve the copy downloaded from our official GitHub release. Never disable Gatekeeper globally.
              </p>
              <details className="permission-recovery"><summary>Already allowed access, but typing is silent?</summary><p>Quit Keytone from its menu-bar menu. In Input Monitoring, remove the old Keytone entry with −, add Applications → Keytone with +, enable it, then launch again. An update can leave the previous build’s permission behind.</p></details>
              <a href="https://support.apple.com/en-gb/102445" target="_blank" rel="noreferrer">
                Apple’s Open Anyway instructions <ArrowUpRight size={13} />
              </a>
            </div>
          </aside>
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
