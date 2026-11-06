import { useState } from "react";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { Download, X } from "lucide-react";
import { formatDuration } from "./pipeline";
import {
  validateSelection,
  type MediaInfo,
  type ExportProgress,
  type ExportResult,
} from "./export";
interface Props {
  sourcePath?: string;
  media: MediaInfo | null;
  currentTime: () => number;
  onBusy: (value: boolean) => void;
}
export default function ExportPanel({
  sourcePath,
  media,
  currentTime,
  onBusy,
}: Props) {
  const [start, setStart] = useState("0");
  const [duration, setDuration] = useState("61");
  const [active, setActive] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const [progress, setProgress] = useState<ExportProgress>({
    phase: "preparing",
    percent: 0,
  });
  const [error, setError] = useState("");
  const [result, setResult] = useState<ExportResult | null>(null);
  const validation = media
    ? validateSelection(Number(start), Number(duration), media.duration)
    : "Bitte zuerst ein Video importieren.";
  const native = isTauri();
  const ready = !!sourcePath && !!media && !validation && native;
  const labels = {
    preparing: "Quelle prüfen / Exportordner auswählen …",
    encoding: "Video wird geschnitten …",
    verifying: "MP4 wird vollständig überprüft …",
    complete: "Export geprüft und abgeschlossen",
  };
  async function exportClip() {
    if (!ready || active) return;
    setActive(true);
    onBusy(true);
    setCancelling(false);
    setError("");
    setResult(null);
    setProgress({ phase: "preparing", percent: 0 });
    const channel = new Channel<ExportProgress>();
    channel.onmessage = (value) => setProgress(value);
    try {
      const value = await invoke<ExportResult | null>("export_clip", {
        sourcePath,
        startSeconds: Number(start),
        durationSeconds: Number(duration),
        onProgress: channel,
      });
      setResult(value);
    } catch (e) {
      setError(String(e));
    } finally {
      setActive(false);
      onBusy(false);
      setCancelling(false);
    }
  }
  async function cancel() {
    setCancelling(true);
    try {
      await invoke("cancel_export");
    } catch (e) {
      setError(String(e));
      setCancelling(false);
    }
  }
  return (
    <section className="export-panel" aria-labelledby="export-heading">
      <div className="panel-heading">
        <h2 id="export-heading">
          <Download size={18} /> Testclip exportieren
        </h2>
        <span>PHASE 02 · LOKAL</span>
      </div>
      <div className="export-body">
        <p>
          Wähle einen Abschnitt. AI Clipper schneidet die Bildmitte auf 9:16 zu
          und exportiert 1080 × 1920 Pixel als H.264 / AAC.
        </p>
        <fieldset disabled={active} className="range-fields">
          <legend className="sr-only">Videoabschnitt</legend>
          <label>
            Start (Sekunden)
            <input
              type="number"
              min="0"
              step="0.1"
              value={start}
              onChange={(e) => {
                setStart(e.target.value);
                setResult(null);
              }}
            />
          </label>
          <button
            className="secondary"
            disabled={!media}
            onClick={() => {
              setStart(currentTime().toFixed(1));
              setResult(null);
            }}
          >
            Playerposition übernehmen
          </button>
          <label>
            Länge (Sekunden)
            <input
              type="number"
              min="61"
              max="90"
              step="1"
              value={duration}
              onChange={(e) => {
                setDuration(e.target.value);
                setResult(null);
              }}
            />
          </label>
        </fieldset>
        {media && (
          <p className="range-summary">
            Abschnitt: {formatDuration(Number(start))} –{" "}
            {formatDuration(Number(start) + Number(duration))} ·{" "}
            {media.hasAudio
              ? "Originalton wird als AAC übernommen"
              : "Quelle ohne Ton – Export bleibt stumm"}
          </p>
        )}
        {validation && <p className="range-warning">{validation}</p>}
        {!native && (
          <p className="range-warning">
            Der lokale FFmpeg-Export ist in der installierten Mac-App verfügbar.
            Die Browser-Vorschau lädt keine Videos hoch.
          </p>
        )}
        <div className="export-actions">
          <button disabled={!ready || active} onClick={exportClip}>
            <Download size={16} /> Abschnitt als MP4 exportieren
          </button>
          {active && (
            <button
              className="secondary"
              disabled={cancelling}
              onClick={cancel}
            >
              <X size={16} />
              {cancelling ? "Wird abgebrochen …" : "Abbrechen"}
            </button>
          )}
        </div>
        {active && (
          <div className="export-progress" role="status">
            <div>
              {cancelling
                ? "Export wird abgebrochen …"
                : labels[progress.phase]}{" "}
              <strong>{Math.floor(progress.percent)} %</strong>
            </div>
            <progress
              aria-label="Exportfortschritt"
              max={100}
              value={progress.percent}
            />
            <small>
              Ein Export gleichzeitig · Originaldatei bleibt unverändert
            </small>
          </div>
        )}
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
        {result && (
          <div className="export-success" role="status">
            <strong>MP4 exportiert und geprüft</strong>
            <p>{result.outputPath}</p>
            <small>
              {result.width} × {result.height} · {result.duration.toFixed(2)}{" "}
              Sekunden · H.264{result.hasAudio ? " / AAC" : " · ohne Ton"}
            </small>
          </div>
        )}
        <p className="export-note">
          FFmpeg ist in der App enthalten. Nach dem Klick wählst du den
          Exportordner aus. Automatische Szenenauswahl und intelligentes
          Reframing folgen später.
        </p>
      </div>
    </section>
  );
}
