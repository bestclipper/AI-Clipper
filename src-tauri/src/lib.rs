use clipper_media::{ExportResult, MediaInfo, Progress, Tools};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{ipc::Channel, Manager, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct ExportState {
    active: AtomicBool,
    cancelled: Arc<AtomicBool>,
}
struct ActiveGuard(tauri::AppHandle);
impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0
            .state::<ExportState>()
            .active
            .store(false, Ordering::SeqCst);
    }
}
fn tools() -> Result<Tools, String> {
    let directory = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("App-Verzeichnis fehlt.")?
        .to_path_buf();
    let ffmpeg = directory.join("ffmpeg");
    let ffprobe = directory.join("ffprobe");
    if !ffmpeg.is_file() || !ffprobe.is_file() {
        return Err("Die mitgelieferten FFmpeg-Werkzeuge fehlen. Bitte die vollständige App aus der DMG installieren.".into());
    }
    Ok(Tools {
        ffmpeg,
        ffprobe,
        encoder: "h264_videotoolbox".into(),
    })
}
fn checked_source(app: &tauri::AppHandle, path: &str) -> Result<PathBuf, String> {
    let path = Path::new(path);
    if !path.is_absolute() || !app.asset_protocol_scope().is_allowed(path) {
        return Err("Bitte die Quelldatei zuerst über Video auswählen importieren.".into());
    }
    let path = path
        .canonicalize()
        .map_err(|_| "Die Quelldatei ist nicht mehr erreichbar.")?;
    if !path.is_file() {
        return Err("Die Quelle ist keine Datei.".into());
    }
    Ok(path)
}
#[tauri::command]
async fn probe_source(app: tauri::AppHandle, source_path: String) -> Result<MediaInfo, String> {
    let source = checked_source(&app, &source_path)?;
    tauri::async_runtime::spawn_blocking(move || tools()?.probe(&source, &AtomicBool::new(false)))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
fn cancel_export(state: State<'_, ExportState>) {
    state.cancelled.store(true, Ordering::SeqCst);
}
#[tauri::command]
async fn export_clip(
    app: tauri::AppHandle,
    source_path: String,
    start_seconds: f64,
    duration_seconds: f64,
    on_progress: Channel<Progress>,
) -> Result<Option<ExportResult>, String> {
    let source = checked_source(&app, &source_path)?;
    let state = app.state::<ExportState>();
    if state
        .active
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Es läuft bereits ein Export. Bitte warten oder abbrechen.".into());
    }
    state.cancelled.store(false, Ordering::SeqCst);
    let cancel = state.cancelled.clone();
    let guard = ActiveGuard(app.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        let tools = tools()?;
        let info = tools.probe(&source, &cancel)?;
        clipper_media::validate_range(start_seconds, duration_seconds, info.duration)?;
        let folder = app
            .dialog()
            .file()
            .set_title("Exportordner auswählen")
            .blocking_pick_folder();
        let Some(folder) = folder else {
            return Ok(None);
        };
        let folder = folder
            .into_path()
            .map_err(|_| "Bitte einen lokalen Ordner auswählen.")?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let output = folder.join(format!("AI-Clipper-{stamp}.mp4"));
        tools
            .export(
                &source,
                &output,
                start_seconds,
                duration_seconds,
                &cancel,
                |progress| {
                    if on_progress.send(progress).is_err() {
                        cancel.store(true, Ordering::SeqCst);
                    }
                },
            )
            .map(Some)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ExportState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            probe_source,
            export_clip,
            cancel_export
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<ExportState>();
                if state.active.load(Ordering::SeqCst) {
                    // Keep the process alive long enough to kill FFmpeg and delete partial output.
                    api.prevent_close();
                    state.cancelled.store(true, Ordering::SeqCst);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("AI Clipper konnte nicht gestartet werden")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let state = app.state::<ExportState>();
                if state.active.load(Ordering::SeqCst) {
                    api.prevent_exit();
                    state.cancelled.store(true, Ordering::SeqCst);
                }
            }
        });
}
