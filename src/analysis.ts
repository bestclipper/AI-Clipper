import type { ModelStatus } from "./transcription";
export interface ClipSegment {
  firstUnit: number;
  lastUnit: number;
  start: number;
  end: number;
  duration: number;
}
export interface Candidate {
  id: string;
  title: string;
  mode: "single_scene" | "smart_story";
  segments: ClipSegment[];
  totalDuration: number;
  score: number;
  category: string;
  reason: string;
  relation: string;
  connectionScore: number;
  ratings: Record<string, number>;
  evidence: string[];
  timestampsApproximate: boolean;
  evaluationMethod?: "qwen" | "deterministic";
}
export interface Analysis {
  finished?: boolean;
  status?:
    | "running"
    | "interrupted"
    | "completed"
    | "completed_with_errors"
    | "insufficient_scenes";
  selectedIds?: string[];
  selectionMessage?: string;
  failures?: {
    stage: string;
    id: string;
    attempts: number;
    code: string;
    recovered: boolean;
  }[];
  complete: boolean;
  model: string;
  nextWindow: number;
  totalWindows: number;
  evaluated: number;
  totalPlans: number;
  skippedContext?: number;
  candidates: Candidate[];
  cpuFallbackUsed: boolean;
}
export interface AnalysisStatus {
  model: ModelStatus;
  componentReady: boolean;
  componentError: string | null;
}
export interface AnalysisProgress {
  phase: string;
  percent: number;
  message?: string;
  receivedBytes?: number;
  totalBytes?: number;
}
// Preview uses the same source intervals; no new file or full video buffer.
export function validPlaylist(parts: ClipSegment[], duration: number): boolean {
  return (
    parts.length >= 1 &&
    parts.length <= 5 &&
    parts.every(
      (p, i) =>
        Number.isFinite(p.start) &&
        Number.isFinite(p.end) &&
        p.start >= 0 &&
        p.end > p.start &&
        p.end <= duration + 0.01 &&
        (i === 0 || p.start >= parts[i - 1].end),
    )
  );
}
export function playlistStep(
  parts: ClipSegment[],
  index: number,
  time: number,
): { index: number; seek?: number; finished: boolean } {
  const part = parts[index];
  if (!part) return { index, finished: true };
  if (time < part.start - 0.1)
    return { index, seek: part.start, finished: false };
  if (time >= part.end - 0.025)
    return index + 1 < parts.length
      ? { index: index + 1, seek: parts[index + 1].start, finished: false }
      : { index, finished: true };
  return { index, finished: false };
}

export function analysisLabel(a: Analysis | null, active: boolean): string {
  if (active) return "Analyse läuft";
  if (!a) return "Noch keine Analyse vorhanden";
  if (!(a.finished ?? a.complete))
    return "Analyse unterbrochen · Fortsetzen möglich";
  if (a.status === "completed_with_errors")
    return "Analyse teilweise fehlgeschlagen · Verarbeitung beendet";
  if ((a.selectedIds?.length ?? 0) < 5)
    return "Analyse abgeschlossen · Keine fünf geeigneten Szenen gefunden";
  return "Analyse abgeschlossen · Fünf Szenen ausgewählt";
}
