export interface Word {
  text: string;
  start: number;
  end: number;
  approximate: boolean;
}
export interface Segment {
  text: string;
  start: number;
  end: number;
  words: Word[];
}
export interface Transcript {
  schemaVersion: number;
  model: string;
  language: string;
  duration: number;
  wordsApproximate: boolean;
  alignmentStatus: string;
  cpuFallbackUsed: boolean;
  segments: Segment[];
}
export interface ModelStatus {
  id: string;
  ready: boolean;
  bytes: number;
  sha256: string;
}
export interface AsrStatus {
  model: ModelStatus;
  componentReady: boolean;
  componentError: string | null;
}
export interface AsrProgress {
  phase: string;
  percent: number;
  chunk?: number;
  totalChunks?: number;
  receivedBytes?: number;
  totalBytes?: number;
}
export function timestamp(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "–";
  const ms = Math.round(seconds * 1000);
  return `${String(Math.floor(ms / 3600000)).padStart(2, "0")}:${String(Math.floor(ms / 60000) % 60).padStart(2, "0")}:${String(Math.floor(ms / 1000) % 60).padStart(2, "0")}.${String(ms % 1000).padStart(3, "0")}`;
}
