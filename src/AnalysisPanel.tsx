import { useEffect, useState } from "react";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { Sparkles, Download, Play } from "lucide-react";
import { analysisLabel } from "./analysis";
import { timestamp, type ModelStatus } from "./transcription";
import type {
  Analysis,
  AnalysisStatus,
  AnalysisProgress,
  Candidate,
} from "./analysis";
interface Props {
  sourcePath?: string;
  otherBusy: boolean;
  onBusy: (value: boolean) => void;
  onPreview: (clip: Candidate) => void;
  refresh: number;
}
const ratings: Record<string, string> = {
  entertainment: "Unterhaltung · 25 %",
  surprise: "Überraschung / Emotion · 20 %",
  hook: "Hook · 20 %",
  clarity: "Verständlichkeit · 15 %",
  tension: "Spannung · 10 %",
  ending: "Abschluss · 10 %",
};
export default function AnalysisPanel({
  sourcePath,
  otherBusy,
  onBusy,
  onPreview,
  refresh,
}: Props) {
  const [status, setStatus] = useState<AnalysisStatus | null>(null),
    [analysis, setAnalysis] = useState<Analysis | null>(null),
    [loading, setLoading] = useState(true),
    [active, setActive] = useState(false),
    [cancelling, setCancelling] = useState(false),
    [error, setError] = useState(""),
    [visible, setVisible] = useState(10);
  const [progress, setProgress] = useState<AnalysisProgress>({
    phase: "preparing",
    percent: 0,
  });
  const native = isTauri();
  useEffect(() => {
    let alive = true;
    if (!native) {
      setLoading(false);
      return;
    }
    setLoading(true);
    Promise.allSettled([
      invoke<AnalysisStatus>("analysis_status"),
      sourcePath
        ? invoke<Analysis | null>("load_analysis", { sourcePath })
        : Promise.resolve(null),
    ]).then(([health, saved]) => {
      if (!alive) return;
      if (health.status === "fulfilled") {
        setStatus(health.value);
        if (health.value.componentError) setError(health.value.componentError);
      } else setError(String(health.reason));
      if (saved.status === "fulfilled") setAnalysis(saved.value);
      else setError(String(saved.reason));
      setLoading(false);
    });
    return () => {
      alive = false;
    };
  }, [sourcePath, native, refresh]);
  async function execute(download: boolean) {
    if (active || otherBusy) return;
    setActive(true);
    onBusy(true);
    setCancelling(false);
    setError("");
    setProgress({ phase: download ? "downloading" : "preparing", percent: 0 });
    const channel = new Channel<AnalysisProgress>();
    channel.onmessage = setProgress;
    try {
      if (download) {
        const model = await invoke<ModelStatus>("download_qwen_model", {
          onProgress: channel,
        });
        setStatus((old) => (old ? { ...old, model } : null));
      } else {
        setAnalysis(
          await invoke<Analysis>("analyze_video", {
            sourcePath,
            onProgress: channel,
          }),
        );
      }
    } catch (e) {
      setError(String(e));
      if (sourcePath) {
        try {
          setAnalysis(
            await invoke<Analysis | null>("load_analysis", { sourcePath }),
          );
        } catch {
          /* Preserve original failure. */
        }
      }
    } finally {
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
    preparing: "Modell und Transkript werden geprüft …",
    downloading: "Qwen wird heruntergeladen …",
    verifying: "SHA-256 wird geprüft …",
    audio: "Audioanalyse",
    events: "Ereignisse sammeln",
    ranking: "Szenen bewerten",
    complete: "Abgeschlossen",
  };
  const selected = new Set(analysis?.selectedIds ?? []);
  const candidates = [...(analysis?.candidates ?? [])].sort(
    (a, b) =>
      Number(selected.has(b.id)) - Number(selected.has(a.id)) ||
      b.score - a.score,
  );
  const failed = analysis?.failures?.filter((f) => !f.recovered) ?? [];
  const recovered = analysis?.failures?.filter((f) => f.recovered) ?? [];
  return (
    <section
      id="analysis"
      className="transcription-panel"
      aria-labelledby="analysis-heading"
    >
      <div className="panel-heading">
        <h2 id="analysis-heading">
          <Sparkles size={18} /> KI-Analyse · Szenenvorschläge
        </h2>
        <span>PHASE 04 · LOKAL</span>
      </div>
      <div className="export-body">
        <p>
          Single Scene findet zusammenhängende Szenen. Smart Story verbindet 2–5
          inhaltlich zusammengehörige Stellen aus dem gesamten Video zu 61–90
          Sekunden.
        </p>
        <div className="model-row">
          <label>
            Lokales KI-Modell
            <select aria-label="KI-Modell" value="qwen" disabled>
              <option value="qwen">Qwen3 1.7B · GGUF Q4_K_M · 1,28 GB</option>
            </select>
          </label>
          <span
            className={status?.model.ready ? "model-ready" : "range-warning"}
          >
            {loading
              ? "Komponenten werden geprüft …"
              : status?.model.ready
                ? "Modell lokal vorhanden und geprüft"
                : "Einmaliger kostenloser Download erforderlich"}
          </span>
        </div>
        <p className="export-note">
          Apache 2.0 · Modell von Qwen, Quantisierung von ggml-org (llama.cpp).
          Download aus Hugging Face, danach offline. Dateigröße und fest
          hinterlegte SHA-256 werden geprüft. Video und Transkript verlassen
          deinen Mac nicht.
        </p>
        {status?.componentReady && (
          <p className="component-ready">
            llama.cpp wurde von der App erfolgreich gestartet · Metal auf Apple
            Silicon, CPU-Fallback verfügbar
          </p>
        )}
        {!native && (
          <p className="range-warning">
            Die lokale KI läuft in der installierten Mac-App.
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
              <Download size={16} /> Qwen kostenlos herunterladen · 1,28 GB
            </button>
          )}
          <button
            disabled={
              !native ||
              loading ||
              !status?.componentReady ||
              !status.model.ready ||
              !sourcePath ||
              active ||
              otherBusy
            }
            onClick={() => execute(false)}
          >
            <Sparkles size={16} />
            {analysis && !(analysis.finished ?? analysis.complete)
              ? "KI-Analyse fortsetzen"
              : "KI-Analyse starten"}
          </button>
          {active && (
            <button
              className="secondary"
              disabled={cancelling}
              onClick={cancel}
            >
              {cancelling
                ? "Wird abgebrochen …"
                : "KI-Analyse / Download abbrechen"}
            </button>
          )}
        </div>
        <p className="export-note">
          Falls noch kein Transkript vorliegt, wird zuerst Whisper base.en
          ausgeführt. Bitte das Whisper-Modell oben installieren. Es läuft immer
          nur ein rechenintensiver Vorgang. Lange Videos können auf dem M1
          längere Zeit benötigen.
        </p>
        {active && (
          <div className="export-progress" role="status">
            <div>
              {cancelling
                ? "Vorgang wird abgebrochen …"
                : (progress.message ??
                  labels[progress.phase] ??
                  progress.phase)}
              <strong>{Math.floor(progress.percent)} %</strong>
            </div>
            <progress
              aria-label="KI-Analysefortschritt"
              max="100"
              value={progress.percent}
            />
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
        {analysis && (
          <div className="scene-results">
            <h3 aria-live="polite">{analysisLabel(analysis, active)}</h3>
            {analysis.selectionMessage && <p>{analysis.selectionMessage}</p>}
            {(analysis.finished ?? analysis.complete) && (
              <p>
                <strong>{selected.size} / 5 automatisch ausgewählt</strong> ·
                höchstens 20 % zeitliche Überlappung
              </p>
            )}
            {failed.length > 0 && (
              <p className="range-warning">
                {failed.length} KI-Schritte sind dauerhaft fehlgeschlagen. Die
                Verarbeitung der übrigen Schritte ist erhalten; lokale
                Ersatzbewertungen sind markiert.
              </p>
            )}
            {recovered.length > 0 && (
              <p>
                {recovered.length} Schritte wurden nach Wiederholung erfolgreich
                verarbeitet.
              </p>
            )}
            {!!analysis.failures?.length && (
              <details>
                <summary>Hinweise zu KI-Schritten</summary>
                <ul>
                  {analysis.failures.map((f) => (
                    <li key={f.id}>
                      {f.stage === "comparison"
                        ? "Vergleich"
                        : "Transkriptfenster"}{" "}
                      · {f.attempts} Versuche ·{" "}
                      {f.recovered
                        ? "wiederhergestellt"
                        : "Ersatzbewertung / lokaler Kandidatenscan"}{" "}
                      ·{" "}
                      {(
                        {
                          incomplete_json: "Antwort unvollständig",
                          output_too_long: "Antwort zu lang",
                          timeout: "Zeitlimit",
                          context_budget: "Kontextbudget überschritten",
                          model_error: "Modell- oder Formatfehler",
                          interrupted_attempts:
                            "Vorherige Versuche unterbrochen",
                        } as Record<string, string>
                      )[f.code] ?? "KI-Fehler"}
                    </li>
                  ))}
                </ul>
              </details>
            )}
            <p>
              {analysis.nextWindow} / {analysis.totalWindows} Transkriptfenster
              · {analysis.evaluated} / {analysis.totalPlans} Vergleiche ·{" "}
              {candidates.length} Vorschläge
            </p>
            {!!analysis.skippedContext && (
              <p>
                {analysis.skippedContext} Vorschläge überschritten das
                Kontextbudget und wurden ausgelassen.
              </p>
            )}
            {analysis.cpuFallbackUsed && <p>CPU-Fallback wurde verwendet.</p>}
            {(analysis.finished ?? analysis.complete) &&
              candidates.length === 0 && (
                <p>
                  Keine ausreichend belegte, verständliche Szene mit 61–90
                  Sekunden gefunden. Es werden keine künstlichen
                  Zusammenstellungen erzwungen.
                </p>
              )}
            {candidates.slice(0, visible).map((clip) => (
              <article className="scene-card" key={clip.id}>
                {selected.has(clip.id) && (
                  <span className="model-ready">
                    Automatisch ausgewählt · Top 5
                  </span>
                )}
                {clip.evaluationMethod === "deterministic" && (
                  <p className="range-warning">
                    Lokale Ersatzbewertung · nicht von Qwen bestätigt
                  </p>
                )}
                <div className="scene-card-heading">
                  <div>
                    <span
                      className={
                        clip.mode === "smart_story"
                          ? "tag story-tag"
                          : "tag scene-tag"
                      }
                    >
                      {clip.mode === "smart_story"
                        ? "Smart Story"
                        : "Single Scene"}{" "}
                      · {clip.category}
                    </span>
                    <h3>{clip.title}</h3>
                  </div>
                  <strong className="scene-score">
                    {clip.score.toFixed(1)}
                    <small> / 10</small>
                  </strong>
                </div>
                <p>
                  {clip.totalDuration.toFixed(1)} Sekunden ·{" "}
                  {clip.segments.length} Segment
                  {clip.segments.length === 1 ? "" : "e"}
                </p>
                <p>{clip.reason}</p>
                <ol>
                  {clip.segments.map((part, i) => (
                    <li key={i}>
                      <span>
                        {timestamp(part.start)} → {timestamp(part.end)} ·{" "}
                        {part.duration.toFixed(1)} s
                      </span>
                      <blockquote>{clip.evidence[i]}</blockquote>
                    </li>
                  ))}
                </ol>
                <button
                  className="secondary"
                  disabled={active || otherBusy}
                  onClick={() => onPreview(clip)}
                >
                  <Play size={16} />{" "}
                  {clip.mode === "smart_story"
                    ? "Smart Story als Playlist ansehen"
                    : "Szene ansehen"}
                </button>
                <details>
                  <summary>Bewertung und Zusammenhang</summary>
                  <dl>
                    {Object.entries(ratings).map(([key, label]) => (
                      <div key={key}>
                        <dt>{label}</dt>
                        <dd>{clip.ratings[key]} / 10</dd>
                      </div>
                    ))}
                    {clip.mode === "smart_story" && (
                      <div>
                        <dt>Zusammenhang · {clip.relation}</dt>
                        <dd>{clip.connectionScore} / 10</dd>
                      </div>
                    )}
                  </dl>
                </details>
              </article>
            ))}
            {candidates.length > visible && (
              <button
                className="secondary"
                onClick={() => setVisible((v) => v + 10)}
              >
                Weitere Vorschläge anzeigen
              </button>
            )}
          </div>
        )}
        <p className="timing-note">
          KI-Scores sind Einschätzungen, keine Viralitätsgarantie. Zeitstempel
          aus Whisper sind ungefähr; Vorschläge bitte ansehen. Audiointensität
          ist kein verlässlicher Lach- oder Sprecherwechseldetektor. Smart
          Stories werden hier nur als Playlist im Originalvideo abgespielt.
          Mehrteiliger MP4-Export und Untertitel folgen in späteren Phasen.
        </p>
      </div>
    </section>
  );
}
