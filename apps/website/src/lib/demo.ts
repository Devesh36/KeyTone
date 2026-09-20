import cream from "../assets/cream.wav?url";
import alpaca from "../assets/alpaca.wav?url";
import navy from "../assets/navy.wav?url";
import { playableCode } from "./keyboard";

export const demoProfiles = [
  { id: "cream", name: "NK Cream", character: "Soft · Linear · Warm", color: "#e6d5b1", stem: "#f5e9d1", sample: cream },
  { id: "alpaca", name: "Alpaca", character: "Smooth · Linear · Rounded", color: "#bf8caa", stem: "#edbad1", sample: alpaca },
  { id: "navy", name: "Box Navy", character: "Sharp · Clicky · Bright", color: "#9caecc", stem: "#6f87b6", sample: navy },
] as const;

export function demoKey(code: string, event: Partial<Pick<KeyboardEvent, "repeat" | "ctrlKey" | "metaKey" | "altKey" | "isComposing">> = {}): string | null {
  if (event.repeat || event.ctrlKey || event.metaKey || event.altKey || event.isComposing) return null;
  return playableCode(code) ? code : null;
}

// Website-only preview. The native desktop audio engine is unchanged.
export class DemoAudio {
  private context: AudioContext | null = null;
  private buffers: AudioBuffer[] = [];
  private voices = new Set<AudioBufferSourceNode>();
  private abort = new AbortController();
  private disposed = false;
  private generation = 0;
  enabled = false;

  async enable() {
    const generation = ++this.generation;
    const Audio = window.AudioContext ?? (window as Window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!Audio) throw new Error("Audio previews aren't supported here. You can still try the animation.");
    this.context ??= new Audio();
    // Resume directly from a user gesture, before fetching the local samples.
    await this.context.resume();
    const context = this.context;
    if (!this.buffers.length) {
      this.buffers = await Promise.all(demoProfiles.map(async ({ sample }) => {
        const response = await fetch(sample, { signal: this.abort.signal });
        if (!response.ok) throw new Error("Couldn't load the sound preview. Please try again.");
        return context.decodeAudioData(await response.arrayBuffer());
      }));
    }
    if (!this.disposed && generation === this.generation) this.enabled = true;
  }

  mute() {
    this.generation += 1;
    this.enabled = false;
    for (const voice of this.voices) voice.stop();
    this.voices.clear();
  }

  async play(index: number) {
    const context = this.context;
    if (!this.enabled || !context || !this.buffers[index]) return;
    if (context.state !== "running") await context.resume();
    if (!this.enabled || this.disposed || this.voices.size >= 12) return;
    const source = context.createBufferSource();
    const gain = context.createGain();
    source.buffer = this.buffers[index];
    gain.gain.value = 0.55;
    source.connect(gain);
    gain.connect(context.destination);
    this.voices.add(source);
    source.onended = () => { this.voices.delete(source); source.disconnect(); gain.disconnect(); };
    source.start();
  }

  dispose() {
    this.disposed = true;
    this.abort.abort();
    this.mute();
    if (this.context && this.context.state !== "closed") void this.context.close().catch(() => undefined);
  }
}
