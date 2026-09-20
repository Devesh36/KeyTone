import type { CSSProperties } from "react";

// Original, stylized artwork, not product photography. Imported packs receive
// a stable color too; no artwork or network request is required in a manifest.
const packColors: Record<string, [housing: string, stem: string]> = {
  creamy: ["#c4bbaa", "#f3e6cb"],
  clicky: ["#aab4c5", "#6b9fdf"],
  retro: ["#98a296", "#edbd78"],
  "deep-thock": ["#9181ad", "#c1a0ee"],
  tactile: ["#b5a08a", "#e9b665"],
  alloy: ["#93aaa9", "#b9ece3"],
  spring: ["#b988a1", "#f3adc6"],
  typewriter: ["#9aa59a", "#ead3a0"],
  "kbsim-alpaca": ["#bb859a", "#efb2c8"],
  "kbsim-blackink": ["#65747b", "#9daeb7"],
  "kbsim-bluealps": ["#9bb5b6", "#77c7e0"],
  "kbsim-boxnavy": ["#b2beca", "#6384b9"],
  "kbsim-cream": ["#c5bda9", "#f0e6c9"],
  "kbsim-holypanda": ["#c3c7b2", "#f0a667"],
  "kbsim-mxblack": ["#77818b", "#505d6b"],
  "kbsim-mxblue": ["#8a9cac", "#6daee9"],
  "kbsim-mxbrown": ["#a49a8f", "#c9976e"],
  "kbsim-redink": ["#ae727b", "#f28b94"],
  "kbsim-topre": ["#a3a898", "#d7debd"],
  "kbsim-turquoise": ["#8db4b1", "#75e0d2"],
};
const fallbackColors: [string, string][] = [
  ["#94aaa2", "#d9fa76"], ["#a18ead", "#d3b0f0"],
  ["#9baaba", "#a5d5ef"], ["#b49a87", "#edc399"],
];

// Resolve the shading here so older system WebViews don't need CSS color-mix.
function shade(color: string, weight: number, target = "#14191b"): string {
  const channels = [1, 3, 5].map((offset) => Math.round(
    parseInt(color.slice(offset, offset + 2), 16) * weight
      + parseInt(target.slice(offset, offset + 2), 16) * (1 - weight),
  ));
  return `rgb(${channels.join(", ")})`;
}

interface MechanicalSwitchProps {
  packId: string;
  strike?: number;
}

export function MechanicalSwitch({ packId, strike = 0 }: MechanicalSwitchProps) {
  const hash = Array.from(packId).reduce((value, char) => (value * 31 + char.charCodeAt(0)) >>> 0, 0);
  const [housing, stem] = Object.prototype.hasOwnProperty.call(packColors, packId)
    ? packColors[packId]
    : fallbackColors[hash % fallbackColors.length];
  const style = {
    "--switch-housing": housing,
    "--switch-stem": stem,
    "--switch-outline": shade(housing, 0.27),
    "--switch-glint": shade(housing, 0.4, "#ffffff"),
    "--switch-stem-glint": shade(stem, 0.45, "#ffffff"),
    "--switch-base-left": shade(housing, 0.56),
    "--switch-base-right": shade(housing, 0.38),
    "--switch-rim": shade(housing, 0.85, "#ffffff"),
    "--switch-left": shade(housing, 0.83),
    "--switch-right": shade(housing, 0.65),
    "--switch-highlight": shade(housing, 0.82, "#ffffff"),
    "--switch-latch": shade(housing, 0.7),
    "--switch-stem-left": shade(stem, 0.88),
    "--switch-stem-right": shade(stem, 0.67),
  } as CSSProperties;

  return (
    <svg key={strike} className={`mechanical-switch${strike ? " is-striking" : ""}`} style={style} viewBox="0 0 200 180" fill="none" aria-hidden="true" focusable="false">
      <ellipse className="switch-shadow" cx="102" cy="155" rx="63" ry="12" fill="#000" opacity=".28" />
      <g stroke="var(--switch-outline)" strokeWidth="1.8" strokeLinejoin="round">
        {/* Lower housing: two shaded faces and molded support ribs. */}
        <path className="switch-base-left" d="M39 101 103 134 103 155 45 125Z" />
        <path className="switch-base-right" d="M103 134 163 100 157 125 103 155Z" />
        <path d="m52 109 3 18 9 5-2-18m12 6 1 18 9 5-1-18m38 1-1 18 9-5 2-18m12-7-2 18 9-5 3-19" stroke="var(--switch-outline)" opacity=".65" />
        <path className="switch-rim" d="m31 96 67-36 72 36-67 40Z" />
        <path className="switch-base-left" d="m31 96 72 36v6l-72-36Z" />
        <path className="switch-base-right" d="m103 132 67-36v6l-67 36Z" />

        {/* Sloped upper housing and recessed stem socket. */}
        <path className="switch-top" d="m53 62 46-25 47 24 12 31-55 30-59-30Z" />
        <path className="switch-left" d="m53 62 50 26v34L44 92Z" />
        <path className="switch-right" d="m103 88 43-27 12 31-55 30Z" />
        <path className="switch-highlight" d="m55 62 44-23 44 23-40 24Z" />
        <path className="switch-socket" d="m72 63 27-15 29 15-26 16Z" />
        <path d="m72 63 30 16 26-16" stroke="var(--switch-outline)" strokeWidth="3" />
        <path className="switch-left" d="m51 77 12 6-3 12-13-6Zm30 17 12 6v12l-14-7Z" />
        <path className="switch-right" d="m137 79 12-7 4 13-13 8Zm-23 16 12-7 2 13-14 8Z" />
        <path d="m55 64-6 26 25 13m71-39 9 26-19 11" stroke="var(--switch-glint)" strokeWidth="1.2" opacity=".55" />
        <path className="switch-latch" d="m88 115 15 8 13-7v9l-13 8-15-8Z" />

        {/* The stem moves into the socket; the housing stays planted. */}
        <g className="switch-stem">
          <path className="switch-stem-left" d="m81 43 20 11v20l-9-5v-9l-11-6Z" />
          <path className="switch-stem-right" d="m101 54 20-11v12l-11 6v9l-9 4Z" />
          <path className="switch-stem-top" d="m81 43 8-5-7-4 10-6 8 4 9-5 10 6-8 5 10 5-10 6-10-5-10 5Z" />
          <path d="m85 43 6 3 10-5 10 5 6-3m-29-9 12 6 13-7" stroke="var(--switch-stem-glint)" strokeWidth="1.2" />
        </g>
      </g>
    </svg>
  );
}
