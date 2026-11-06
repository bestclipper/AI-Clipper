export interface MediaInfo {
  duration: number;
  width: number;
  height: number;
  hasAudio: boolean;
}
export interface ExportProgress {
  phase: "preparing" | "encoding" | "verifying" | "complete";
  percent: number;
}
export interface ExportResult extends MediaInfo {
  outputPath: string;
}
export function validateSelection(
  start: number,
  duration: number,
  total: number,
): string | null {
  if (!Number.isFinite(start) || start < 0)
    return "Bitte einen gültigen Startzeitpunkt ab 0 Sekunden eingeben.";
  if (!Number.isFinite(duration) || duration < 61 || duration > 90)
    return "Die Clip-Länge muss zwischen 61 und 90 Sekunden liegen.";
  if (!Number.isFinite(total) || start + duration > total + 0.01)
    return "Der Abschnitt reicht über das Videoende hinaus. Mindestens 61 Sekunden werden benötigt.";
  return null;
}
