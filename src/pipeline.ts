import type { Candidate } from "./analysis";
/** Multipart contract shared with Phase 4; automatic rendering follows later. */
export const OUTPUT_PROFILE = Object.freeze({
  count: 10,
  minSeconds: 61,
  maxSeconds: 90,
  width: 1080,
  height: 1920,
  subtitleLanguage: "en",
  format: "mp4",
} as const);
export type PipelineStage =
  "transcription" | "ranking" | "reframing" | "subtitles" | "export";
export type ClipCandidate = Candidate;
export interface LocalPipeline {
  analyze(sourcePath: string, signal: AbortSignal): Promise<ClipCandidate[]>;
  export(
    sourcePath: string,
    clips: ClipCandidate[],
    outputDirectory: string,
    signal: AbortSignal,
  ): Promise<string[]>;
}
export const VIDEO_EXTENSIONS = ["mp4", "mov", "m4v", "webm"] as const;
export function isVideoName(name: string) {
  return VIDEO_EXTENSIONS.some((ext) => name.toLowerCase().endsWith(`.${ext}`));
}
export function formatDuration(value: number) {
  if (!Number.isFinite(value) || value < 0) return "–";
  const seconds = Math.floor(value);
  return [
    Math.floor(seconds / 3600),
    Math.floor(seconds / 60) % 60,
    seconds % 60,
  ]
    .map((n) => String(n).padStart(2, "0"))
    .join(":");
}
