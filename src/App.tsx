import { useEffect, useRef, useState } from "react";
import { convertFileSrc, isTauri, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Scissors,
  Film,
  Upload,
  ShieldCheck,
  ArrowUpRight,
  Sparkles,
  Captions,
  Scan,
  Download,
  Check,
  MonitorPlay,
  X,
} from "lucide-react";
import {
  formatDuration,
  isVideoName,
  OUTPUT_PROFILE,
  VIDEO_EXTENSIONS,
} from "./pipeline";
import ExportPanel from "./ExportPanel";
import type { MediaInfo } from "./export";
type Source = {
  id: number;
  name: string;
  url: string;
  blob: boolean;
  nativePath?: string;
};
export default function App() {
  const [source, setSource] = useState<Source | null>(null);
  const [meta, setMeta] = useState<{
    duration: number;
    width: number;
    height: number;
  } | null>(null);
  const [error, setError] = useState("");
  const [exportBusy, setExportBusy] = useState(false);
  const [nativeMedia, setNativeMedia] = useState<MediaInfo | null>(null);
  const video = useRef<HTMLVideoElement>(null);
  const [busy, setBusy] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const sourceRevision = useRef(0);
  useEffect(
    () => () => {
      if (source?.blob) URL.revokeObjectURL(source.url);
    },
    [source],
  );
  function replace(next: Omit<Source, "id">) {
    setError("");
    setMeta(null);
    setNativeMedia(null);
    setSource({ ...next, id: ++sourceRevision.current });
  }
  async function importVideo() {
    if (!isTauri()) {
      input.current?.click();
      return;
    }
    setBusy(true);
    try {
      const path = await open({
        multiple: false,
        directory: false,
        title: "Video auswählen",
        filters: [{ name: "Videos", extensions: [...VIDEO_EXTENSIONS] }],
      });
      if (typeof path === "string") {
        if (!isVideoName(path)) {
          setError("Bitte eine MP4-, MOV-, M4V- oder WebM-Datei auswählen.");
          return;
        }
        replace({
          name: path.split(/[\\/]/).pop() || "Video",
          url: convertFileSrc(path),
          blob: false,
          nativePath: path,
        });
        try {
          const info = await invoke<MediaInfo>("probe_source", {
            sourcePath: path,
          });
          setNativeMedia(info);
        } catch (e) {
          setError(`FFmpeg-Importprüfung: ${String(e)}`);
        }
      }
    } catch {
      setError(
        "Die Datei konnte nicht geöffnet werden. Bitte erneut auswählen.",
      );
    } finally {
      setBusy(false);
    }
  }
  return (
    <div className="shell">
      <aside className="sidebar">
        <a className="brand" href="#">
          <span className="brand-icon">
            <Scissors size={23} />
          </span>
          AI Clipper<span className="beta">BETA</span>
        </a>
        <div className="workspace-label">WORKSPACE</div>
        <div className="nav-active">
          <Film size={18} /> Mein Projekt <span>01</span>
        </div>
        <div className="sidebar-note">
          <ShieldCheck size={20} />
          <strong>Deine Videos bleiben hier.</strong>
          <p>
            Kein Upload. Keine Cloud-API.
            <br />
            Alles auf deinem Mac.
          </p>
        </div>
        <div className="device">
          <span className="dot" /> Apple Silicon <small>Phase 2 · v0.2.0</small>
        </div>
      </aside>
      <main>
        <header>
          <div className="breadcrumb">
            Workspace <span>/</span> Neues Projekt
          </div>
          <span className="local">
            <span className="dot" /> Lokal & privat
          </span>
        </header>
        <div className="content">
          <div className="page-title">
            <div>
              <div className="eyebrow">GROSSE MOMENTE. KLEINE CLIPS.</div>
              <h1>
                Dein nächster Clip beginnt hier<span>.</span>
              </h1>
              <p>Importiere deinen Stream und entdecke, was drinsteckt.</p>
            </div>
            <span className="phase">
              PHASE 02 <span>Lokaler Videoschnitt</span>
            </span>
          </div>
          <div className="steps">
            <div className="step current">
              <b>1</b> Video importieren
            </div>
            <div className="step">
              <b>2</b> KI-Analyse <small>Später</small>
            </div>
            <div className="step">
              <b>3</b> Testclip exportieren
            </div>
          </div>
          <div className="editor-grid">
            <section className="preview-panel">
              <div className="panel-heading">
                <h2>
                  <MonitorPlay size={18} /> Video-Vorschau
                </h2>
                <span>
                  {source ? "QUELLVIDEO" : "BEREIT FÜR DEINEN STREAM"}
                </span>
              </div>
              <div className={"preview " + (source ? "has-video" : "")}>
                {source ? (
                  <video
                    ref={video}
                    key={source.id}
                    src={source.url}
                    controls
                    playsInline
                    preload="metadata"
                    aria-label="Video-Vorschau"
                    onLoadedMetadata={(e) => {
                      const v = e.currentTarget;
                      if (
                        !Number.isFinite(v.duration) ||
                        v.duration <= 0 ||
                        !v.videoWidth
                      ) {
                        setError(
                          "Diese Datei enthält kein abspielbares Video.",
                        );
                        return;
                      }
                      setMeta({
                        duration: v.duration,
                        width: v.videoWidth,
                        height: v.videoHeight,
                      });
                    }}
                    onError={() => {
                      setMeta(null);
                      setError(
                        "Dieses Video kann nicht wiedergegeben werden. Versuche MP4 mit H.264-Video und AAC-Audio.",
                      );
                    }}
                  />
                ) : (
                  <div className="empty">
                    <div className="upload-icon">
                      <Upload size={30} />
                    </div>
                    <h3>Dein Stream. Dein Ausgangspunkt.</h3>
                    <p>
                      Wähle ein Video von deinem Mac aus.
                      <br />
                      Es wird nichts hochgeladen.
                    </p>
                    <button onClick={importVideo} disabled={busy || exportBusy}>
                      <Upload size={16} />{" "}
                      {busy ? "Wird geöffnet …" : "Video auswählen"}
                    </button>
                    <small>
                      MP4, MOV, M4V, WebM · Codec-Unterstützung abhängig von
                      macOS
                    </small>
                  </div>
                )}
              </div>
              <div className="source-bar">
                <Film size={19} />
                <div>
                  <strong>
                    {source?.name || "Noch kein Video importiert"}
                  </strong>
                  <small>
                    {meta
                      ? `${formatDuration(meta.duration)} · ${meta.width} × ${meta.height} px`
                      : source
                        ? "Metadaten werden nach dem Laden angezeigt"
                        : "Deine Originaldatei bleibt unverändert"}
                  </small>
                </div>
                {source && (
                  <>
                    <button
                      className="secondary"
                      onClick={importVideo}
                      disabled={busy || exportBusy}
                    >
                      Ändern
                    </button>
                    <button
                      className="icon-button"
                      aria-label="Video entfernen"
                      disabled={exportBusy}
                      onClick={() => {
                        setSource(null);
                        setNativeMedia(null);
                        setMeta(null);
                        setError("");
                      }}
                    >
                      <X size={18} />
                    </button>
                  </>
                )}
              </div>
            </section>
            <aside className="settings">
              <div className="panel-heading">
                <h2>Clip-Profil</h2>
                <span className="tag">ZIEL</span>
              </div>
              <p className="settings-intro">
                Für deine besten Momente.
                <br />
                Für den vertikalen Feed.
              </p>
              <div className="profile-visual">
                <div className="phone">
                  <Scissors size={23} />
                  <span>9:16</span>
                </div>
                <div>
                  <strong>Vertical video</strong>
                  <small>1080 × 1920 Pixel</small>
                  <span className="profile-badge">SHORT-FORM</span>
                </div>
              </div>
              <dl>
                <div>
                  <dt>Anzahl Clips</dt>
                  <dd>{OUTPUT_PROFILE.count} Clips</dd>
                </div>
                <div>
                  <dt>Länge pro Clip</dt>
                  <dd>61–90 Sekunden</dd>
                </div>
                <div>
                  <dt>Untertitel</dt>
                  <dd>Englisch</dd>
                </div>
                <div>
                  <dt>Exportformat</dt>
                  <dd>MP4</dd>
                </div>
              </dl>
              <div className="info">
                <Sparkles size={18} />
                <p>
                  Das Zielprofil für die spätere KI-Auswahl. Einzelne Abschnitte
                  kannst du jetzt unten exportieren.
                </p>
              </div>
              <button className="generate" disabled>
                <Sparkles size={17} /> 5 Clips erstellen <span>Später</span>
              </button>
            </aside>
          </div>
          <ExportPanel
            key={source?.id ?? 0}
            sourcePath={source?.nativePath}
            media={
              nativeMedia ??
              (source?.blob && meta ? { ...meta, hasAudio: false } : null)
            }
            currentTime={() => video.current?.currentTime ?? 0}
            onBusy={setExportBusy}
          />
          {error && (
            <div role="alert" className="error">
              {error}
            </div>
          )}
          <input
            ref={input}
            type="file"
            accept=".mp4,.mov,.m4v,.webm"
            hidden
            onChange={(e) => {
              const file = e.target.files?.[0];
              e.target.value = "";
              if (!file) return;
              if (!isVideoName(file.name)) {
                setError("Bitte eine unterstützte Videodatei auswählen.");
                return;
              }
              replace({
                name: file.name,
                url: URL.createObjectURL(file),
                blob: true,
              });
            }}
          />
          <section className="roadmap">
            <div className="roadmap-title">
              <h2>Vom langen Stream zum Highlight</h2>
              <span>
                WAS ALS NÄCHSTES KOMMT <ArrowUpRight size={14} />
              </span>
            </div>
            <div className="feature-grid">
              {[
                {
                  icon: Check,
                  title: "Lokaler Import",
                  text: "Video auswählen und direkt ansehen.",
                  ready: true,
                },
                {
                  icon: Scan,
                  title: "Die besten Szenen",
                  text: "KI findet automatisch fünf Highlights.",
                },
                {
                  icon: Captions,
                  title: "Englische Untertitel",
                  text: "Lokal transkribiert. Direkt im Clip.",
                },
                {
                  icon: Download,
                  title: "Bereit zum Teilen",
                  text: "Ein Testclip als vertikale MP4 exportieren.",
                  ready: true,
                },
              ].map((item) => (
                <article key={item.title}>
                  <item.icon size={20} />
                  <span className={item.ready ? "available" : ""}>
                    {item.ready ? "VERFÜGBAR" : "GEPLANT"}
                  </span>
                  <h3>{item.title}</h3>
                  <p>{item.text}</p>
                </article>
              ))}
            </div>
          </section>
          <footer>
            <span>
              <ShieldCheck size={14} /> Lokal entwickelt. Privat gedacht.
            </span>
            <span>Keine Abos. Keine API-Schlüssel.</span>
          </footer>
        </div>
      </main>
    </div>
  );
}
