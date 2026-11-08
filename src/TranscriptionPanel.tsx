import { useEffect, useRef, useState } from "react";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { Captions, Download, X } from "lucide-react";
import {
  timestamp,
  type AsrStatus,
  type AsrProgress,
  type Transcript,
  type ModelStatus,
} from "./transcription";
interface Props {
  sourcePath?: string;
  hasAudio?: boolean;
  otherBusy: boolean;
  onBusy: (value: boolean) => void;
  onSeek: (seconds: number) => void;
}
export default function TranscriptionPanel({
  sourcePath,
  hasAudio,
  otherBusy,
  onBusy,
  onSeek,
}: Props) {
  const [status, setStatus] = useState<AsrStatus | null>(null);
  const [transcript, setTranscript] = useState<Transcript | null>(null);
  const [active, setActive] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const [error, setError] = useState("");
  const [restored, setRestored] = useState(false);
  const [loading, setLoading] = useState(true);
  const [progress, setProgress] = useState<AsrProgress>({
    phase: "preparing",
    percent: 0,
  });
  const [filter, setFilter] = useState("");
  const [visible, setVisible] = useState(100);
  const generation = useRef(0);
  const native = isTauri();
  useEffect(() => {
    let alive = true;
    if (!native) {
      setLoading(false);
      return;
    }
    setLoading(true);
    Promise.allSettled([
      invoke<AsrStatus>("asr_status"),
      sourcePath
        ? invoke<Transcript | null>("load_transcript", { sourcePath })
        : Promise.resolve(null),
    ]).then(([health, saved]) => {
      if (!alive) return;
      if (health.status === "fulfilled") {
        setStatus(health.value);
        if (health.value.componentError) setError(health.value.componentError);
      } else setError(String(health.reason));
      if (saved.status === "fulfilled") {
        setTranscript(saved.value);
        setRestored(!!saved.value);
      } else setError(String(saved.reason));
      setLoading(false);
    });
    return () => {
      alive = false;
    };
  }, [sourcePath, native]);
  async function execute(download: boolean) {
    if (active || otherBusy) return;
    const job = ++generation.current;
    setActive(true);
    setCancelling(false);
    onBusy(true);
    setError("");
    setProgress({ phase: download ? "downloading" : "preparing", percent: 0 });
    const channel = new Channel<AsrProgress>();
    channel.onmessage = (p) => {
      if (generation.current === job) setProgress(p);
    };
    try {
      if (download) {
        const model = await invoke<ModelStatus>("download_whisper_model", {
          onProgress: channel,
        });
        setStatus((previous) => (previous ? { ...previous, model } : null));
      } else {
        const value = await invoke<Transcript>("transcribe_video", {
          sourcePath,
          onProgress: channel,
        });
        setTranscript(value);
        setRestored(false);
      }
    } catch (e) {
      setError(String(e));
      if (!download) {
        try {
          setStatus(await invoke<AsrStatus>("asr_status"));
        } catch {
          /* Keep the original error. */
        }
      }
    } finally {
      generation.current++;
      setActive(false);
      setCancelling(false);
      onBusy(false);
    }
  }
  async function cancel() {
    setCancelling(true);
    try {
      await invoke("cancel_operation");
    } catch (e) {
      setError(String(e));
      setCancelling(false);
    }
  }
  const labels: Record<string, string> = {
    preparing: "Modell und Quelle werden geprüft …",
    downloading: "Englisches Modell wird heruntergeladen …",
    verifying: "Modell-Integrität wird geprüft …",
    extracting: "Audioabschnitt wird vorbereitet …",
    transcribing: "Englische Sprache wird transkribiert …",
    complete: "Abgeschlossen",
  };
  const segments =
    transcript?.segments.filter((s) =>
      s.text.toLowerCase().includes(filter.toLowerCase()),
    ) ?? [];
  return (
    <section
      id="transcription"
      className="transcription-panel"
      aria-labelledby="transcription-heading"
    >
      <div className="panel-heading">
        <h2 id="transcription-heading">
          <Captions size={18} /> KI-Analyse · Englische Transkription
        </h2>
        <span>PHASE 03 · LOKAL</span>
      </div>
      <div className="export-body">
        <p>
          whisper.cpp erkennt englische Sprache lokal. Video und Ton werden
          nicht hochgeladen.
        </p>
        <div className="model-row">
          <label>
            Whisper-Modell
            <select aria-label="Whisper-Modell" value="base.en" disabled>
              <option value="base.en">base.en · Standard · ca. 148 MB</option>
              <option disabled value="small.en">
                small.en · später verfügbar
              </option>
            </select>
          </label>
          <span
            className={status?.model.ready ? "model-ready" : "range-warning"}
          >
            {loading
              ? "Komponenten werden geprüft …"
              : status?.model.ready
                ? "Modell lokal vorhanden und geprüft"
                : "Modell noch nicht installiert"}
          </span>
        </div>
        {status?.componentReady && (
          <p className="component-ready">
            whisper.cpp wurde von der App erfolgreich gestartet · Metal
            automatisch, CPU-Fallback verfügbar
          </p>
        )}
        {!native && (
          <p className="range-warning">
            Transkription und Modell-Download sind in der installierten Mac-App
            verfügbar.
          </p>
        )}
        <div className="export-actions">
          {!status?.model.ready && (
            <button
              disabled={
                !native ||
                loading ||
                !status?.componentReady ||
                active ||
                otherBusy
              }
              onClick={() => execute(true)}
            >
              <Download size={16} /> base.en kostenlos herunterladen
            </button>
          )}
          <button
            disabled={
              !native ||
              loading ||
              !status?.componentReady ||
              !status.model.ready ||
              !sourcePath ||
              hasAudio === false ||
              active ||
              otherBusy
            }
            onClick={() => execute(false)}
          >
            <Captions size={16} /> Transkription starten
          </button>
          {active && (
            <button
              className="secondary"
              onClick={cancel}
              disabled={cancelling}
            >
              <X size={16} />
              {cancelling
                ? "Wird abgebrochen …"
                : "Transkription / Download abbrechen"}
            </button>
          )}
        </div>
        {!sourcePath && (
          <p className="range-warning">Bitte zuerst ein Video importieren.</p>
        )}
        {hasAudio === false && sourcePath && (
          <p className="range-warning">Das Video hat keine Tonspur.</p>
        )}
        {active && (
          <div className="export-progress" role="status">
            <div>
              {cancelling
                ? "Vorgang wird abgebrochen …"
                : (labels[progress.phase] ?? progress.phase)}
              <strong>{Math.floor(progress.percent)} %</strong>
            </div>
            <progress
              aria-label="Transkriptionsfortschritt"
              max="100"
              value={progress.percent}
            />
            {progress.totalChunks && (
              <small>
                Abschnitt {progress.chunk} von {progress.totalChunks}
              </small>
            )}
            {progress.totalBytes && (
              <small>
                {((progress.receivedBytes ?? 0) / 1e6).toFixed(1)} /{" "}
                {(progress.totalBytes / 1e6).toFixed(1)} MB
              </small>
            )}
          </div>
        )}
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
        {transcript && (
          <div className="transcript">
            <h3>
              {restored
                ? "Gespeichertes Transkript wiederhergestellt"
                : "Transkript lokal gespeichert"}
            </h3>
            <p className="timing-note">
              Automatisch erkannt, nicht manuell geprüft. Segment- und
              Wortzeiten sind Näherungen. Wortzeiten sind für spätere Korrektur
              vorbereitet und noch keine synchronisierten Untertitel.
            </p>
            {transcript.cpuFallbackUsed && (
              <p>
                Mindestens ein Abschnitt wurde mit CPU-Fallback verarbeitet.
              </p>
            )}
            <label>
              Transkript durchsuchen
              <input
                type="search"
                value={filter}
                onChange={(e) => {
                  setFilter(e.target.value);
                  setVisible(100);
                }}
                placeholder="Text suchen …"
              />
            </label>
            <div
              className="transcript-list"
              aria-label="Transkript mit Zeitstempeln"
            >
              {segments.length === 0 ? (
                <p>
                  {transcript.segments.length === 0
                    ? "Keine englische Sprache erkannt."
                    : "Keine passenden Textstellen."}
                </p>
              ) : (
                segments.slice(0, visible).map((segment, i) => (
                  <article key={`${segment.start}-${i}`}>
                    <button
                      className="time-button"
                      onClick={() => onSeek(segment.start)}
                      aria-label={`Zu ${timestamp(segment.start)} springen`}
                    >
                      {timestamp(segment.start)} – {timestamp(segment.end)}
                    </button>
                    <p>{segment.text}</p>
                    {segment.words.length > 0 && (
                      <details>
                        <summary>Ungefähre Wortzeiten</summary>
                        <div className="word-times">
                          {segment.words.map((word, j) => (
                            <button
                              key={j}
                              className="secondary"
                              onClick={() => onSeek(word.start)}
                              title={`${timestamp(word.start)} – ${timestamp(word.end)} · ungefähr`}
                            >
                              {word.text}{" "}
                              <small>≈ {timestamp(word.start)}</small>
                            </button>
                          ))}
                        </div>
                      </details>
                    )}
                  </article>
                ))
              )}
            </div>
            {segments.length > visible && (
              <button
                className="secondary"
                onClick={() => setVisible((count) => count + 100)}
              >
                Weitere Textstellen anzeigen ({visible} / {segments.length})
              </button>
            )}
          </div>
        )}
        <p className="export-note">
          Einmaliger kostenloser Modell-Download. Danach offline nutzbar. Pro
          Vorgang läuft nur ein Modellprozess; Audio wird in Abschnitten von
          höchstens fünf Minuten verarbeitet. Erneuter Import derselben
          unveränderten Videodatei stellt das Transkript wieder her. Noch keine
          Szenenauswahl oder Untertitel-Erstellung.
        </p>
      </div>
    </section>
  );
}
