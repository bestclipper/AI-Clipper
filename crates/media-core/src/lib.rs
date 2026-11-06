use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

pub const FILTER: &str = "scale=w='ceil(iw*sar/2)*2':h=ih,setsar=1,crop=w='floor(min(iw,ih*9/16)/2)*2':h='floor(min(ih,iw*16/9)/2)*2':x='(iw-ow)/2':y='(ih-oh)/2',scale=1080:1920:flags=bilinear,setsar=1,fps=30";
#[derive(Clone)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
    pub encoder: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub has_audio: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub phase: String,
    pub percent: f64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub output_path: String,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub has_audio: bool,
}
fn progress(phase: &str, percent: f64) -> Progress {
    Progress {
        phase: phase.into(),
        percent,
    }
}
fn number(v: &serde_json::Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str()?.parse().ok())
}
pub fn validate_range(start: f64, duration: f64, source_duration: f64) -> Result<(), String> {
    if !start.is_finite()
        || !duration.is_finite()
        || !source_duration.is_finite()
        || start < 0.0
        || !(61.0..=90.0).contains(&duration)
    {
        return Err("Start muss positiv oder 0 sein; die Clip-Länge muss zwischen 61 und 90 Sekunden liegen.".into());
    }
    if start + duration > source_duration + 0.01 {
        return Err("Der gewählte Abschnitt reicht über das Videoende hinaus.".into());
    }
    Ok(())
}
fn bounded_read(mut reader: impl Read) -> Vec<u8> {
    let mut tail = Vec::new();
    let mut chunk = [0u8; 4096];
    while let Ok(n) = reader.read(&mut chunk) {
        if n == 0 {
            break;
        }
        tail.extend_from_slice(&chunk[..n]);
        if tail.len() > 32768 {
            tail.drain(..tail.len() - 32768);
        }
    }
    tail
}
// No shell, no inherited stdin, bounded logs and one process at a time.
fn run(
    tool: &Path,
    args: &[String],
    cancel: &AtomicBool,
    mut on_line: impl FnMut(&str),
    timeout: Duration,
) -> Result<Vec<u8>, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("Export abgebrochen.".into());
    }
    let mut child = Command::new(tool)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Medienwerkzeug konnte nicht gestartet werden: {e}"))?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let (tx, rx) = mpsc::sync_channel(128);
    let out_thread = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            match line {
                Ok(line) => {
                    if tx.send(line).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    let err_thread = thread::spawn(move || bounded_read(stderr));
    let began = Instant::now();
    let mut output = Vec::new();
    let mut interrupted = None;
    let status = loop {
        if cancel.load(Ordering::Relaxed) || began.elapsed() > timeout {
            interrupted = Some(if cancel.load(Ordering::Relaxed) {
                "Export abgebrochen."
            } else {
                "Das Medienwerkzeug hat das Zeitlimit überschritten."
            });
            let _ = child.kill();
            break child.wait();
        }
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                on_line(&line);
                if output.len() < 1048576 {
                    output.extend_from_slice(line.as_bytes());
                    output.push(b'\n');
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => thread::sleep(Duration::from_millis(25)),
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(e);
            }
        }
    };
    // Drain before joining so the bounded producer cannot block on a full channel.
    for line in rx {
        on_line(&line);
        if output.len() < 1048576 {
            output.extend_from_slice(line.as_bytes());
            output.push(b'\n');
        }
    }
    let _ = out_thread.join();
    let errors = err_thread.join().unwrap_or_default();
    if let Some(message) = interrupted {
        return Err(message.into());
    }
    let status = status.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!(
            "Videoverarbeitung fehlgeschlagen: {}",
            String::from_utf8_lossy(&errors)
        ));
    }
    Ok(output)
}
impl Tools {
    fn probe_json(&self, source: &Path, cancel: &AtomicBool) -> Result<serde_json::Value, String> {
        let args = vec!["-v", "error", "-protocol_whitelist", "file,pipe", "-show_entries", "format=duration:stream=codec_type,codec_name,width,height,sample_aspect_ratio,duration", "-of", "json", "-i"].into_iter().map(String::from).chain([source.to_string_lossy().into_owned()]).collect::<Vec<_>>();
        let bytes = run(
            &self.ffprobe,
            &args,
            cancel,
            |_| {},
            Duration::from_secs(60),
        )?;
        serde_json::from_slice(&bytes)
            .map_err(|_| "Videometadaten konnten nicht gelesen werden.".into())
    }
    pub fn probe(&self, source: &Path, cancel: &AtomicBool) -> Result<MediaInfo, String> {
        info(&self.probe_json(source, cancel)?)
    }
    pub fn export(
        &self,
        source: &Path,
        output: &Path,
        start: f64,
        duration: f64,
        cancel: &AtomicBool,
        mut notify: impl FnMut(Progress),
    ) -> Result<ExportResult, String> {
        notify(progress("preparing", 0.0));
        let input = self.probe(source, cancel)?;
        validate_range(start, duration, input.duration)?;
        // create_new prevents replacement of any existing file, including the source.
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)
            .map_err(|e| format!("Zieldatei konnte nicht neu angelegt werden: {e}"))?;
        let result = (|| {
            let mut args: Vec<String> = [
                "-hide_banner",
                "-loglevel",
                "error",
                "-nostdin",
                "-y",
                "-threads",
                "2",
                "-filter_threads",
                "1",
                "-protocol_whitelist",
                "file,pipe",
                "-ss",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            args.extend([
                start.to_string(),
                "-i".into(),
                source.to_string_lossy().into_owned(),
                "-t".into(),
                duration.to_string(),
                "-map".into(),
                "0:v:0".into(),
                "-map".into(),
                "0:a:0?".into(),
                "-sn".into(),
                "-dn".into(),
                "-vf".into(),
                FILTER.into(),
                "-c:v".into(),
                self.encoder.clone(),
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-b:v".into(),
                "6000k".into(),
                "-maxrate".into(),
                "8000k".into(),
                "-bufsize".into(),
                "8000k".into(),
                "-threads".into(),
                "2".into(),
            ]);
            if self.encoder == "libx264" {
                args.extend(["-preset".into(), "ultrafast".into()]);
            }
            if self.encoder == "h264_videotoolbox" {
                args.extend([
                    "-allow_sw".into(),
                    "1".into(),
                    "-realtime".into(),
                    "1".into(),
                ]);
            }
            args.extend([
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "192k".into(),
                "-ar".into(),
                "48000".into(),
                "-af".into(),
                "apad".into(),
                "-map_metadata".into(),
                "-1".into(),
                "-map_chapters".into(),
                "-1".into(),
                "-movflags".into(),
                "+faststart".into(),
                "-progress".into(),
                "pipe:1".into(),
                "-nostats".into(),
                "-f".into(),
                "mp4".into(),
                output.to_string_lossy().into_owned(),
            ]);
            notify(progress("encoding", 1.0));
            let mut last = 1.0f64;
            run(
                &self.ffmpeg,
                &args,
                cancel,
                |line| {
                    if let Some(value) = line
                        .strip_prefix("out_time_us=")
                        .and_then(|s| s.parse::<f64>().ok())
                    {
                        let percent =
                            (1.0 + value / 1_000_000.0 / duration * 88.0).clamp(last, 89.0);
                        last = percent;
                        notify(progress("encoding", percent));
                    }
                },
                Duration::from_secs(3600),
            )?;
            notify(progress("verifying", 90.0));
            let raw = self.probe_json(output, cancel)?;
            let exported = verify_metadata(&raw, duration, input.has_audio)?;
            let args: Vec<String> = [
                "-hide_banner",
                "-loglevel",
                "error",
                "-xerror",
                "-nostdin",
                "-threads",
                "2",
                "-protocol_whitelist",
                "file,pipe",
                "-i",
            ]
            .iter()
            .map(|s| s.to_string())
            .chain([output.to_string_lossy().into_owned()])
            .chain(
                ["-map", "0:v:0", "-map", "0:a:0?", "-f", "null", "-"]
                    .iter()
                    .map(|s| s.to_string()),
            )
            .collect();
            run(
                &self.ffmpeg,
                &args,
                cancel,
                |_| {},
                Duration::from_secs(600),
            )?;
            if cancel.load(Ordering::Relaxed) {
                return Err("Export abgebrochen.".into());
            }
            notify(progress("complete", 100.0));
            Ok(ExportResult {
                output_path: output.to_string_lossy().into_owned(),
                duration: exported.duration,
                width: exported.width,
                height: exported.height,
                has_audio: exported.has_audio,
            })
        })();
        if result.is_err() {
            let _ = fs::remove_file(output);
        }
        result
    }
}
fn info(value: &serde_json::Value) -> Result<MediaInfo, String> {
    let streams = value["streams"]
        .as_array()
        .ok_or("Keine Videospuren gefunden.")?;
    let video = streams
        .iter()
        .find(|s| s["codec_type"] == "video")
        .ok_or("Keine Videospur gefunden.")?;
    let duration = number(&video["duration"])
        .or_else(|| number(&value["format"]["duration"]))
        .ok_or("Videodauer ist unbekannt.")?;
    let width = video["width"].as_u64().unwrap_or(0) as u32;
    let height = video["height"].as_u64().unwrap_or(0) as u32;
    if !duration.is_finite() || duration <= 0.0 || width == 0 || height == 0 {
        return Err("Ungültige Videometadaten.".into());
    }
    Ok(MediaInfo {
        duration,
        width,
        height,
        has_audio: streams.iter().any(|s| s["codec_type"] == "audio"),
    })
}
pub fn verify_metadata(
    value: &serde_json::Value,
    duration: f64,
    had_audio: bool,
) -> Result<MediaInfo, String> {
    let data = info(value)?;
    let streams = value["streams"].as_array().unwrap();
    let video = streams.iter().find(|s| s["codec_type"] == "video").unwrap();
    let audio = streams.iter().find(|s| s["codec_type"] == "audio");
    if data.width != 1080
        || data.height != 1920
        || video["codec_name"] != "h264"
        || video["sample_aspect_ratio"] != "1:1"
        || (data.duration - duration).abs() > 0.1
        || data.duration < 60.99
        || data.duration > 90.1
        || data.has_audio != had_audio
        || audio.is_some_and(|s| s["codec_name"] != "aac")
    {
        return Err(
            "Exportprüfung fehlgeschlagen: Auflösung, Codecs, Tonspur oder Dauer stimmen nicht."
                .into(),
        );
    }
    if had_audio {
        let audio_duration = number(&audio.unwrap()["duration"]).ok_or("Tonspurdauer fehlt.")?;
        if (audio_duration - duration).abs() > 0.15 {
            return Err("Die exportierte Tonspur ist zu kurz oder zu lang.".into());
        }
    }
    Ok(data)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_ranges() {
        for (s, d, t) in [
            (f64::NAN, 61., 100.),
            (-1., 61., 100.),
            (0., 60., 100.),
            (0., 91., 100.),
            (40., 61., 100.),
            (0., 61., f64::INFINITY),
        ] {
            assert!(validate_range(s, d, t).is_err());
        }
        assert!(validate_range(39., 61., 100.).is_ok());
        assert!(validate_range(0., 90., 90.).is_ok());
    }
    #[test]
    fn rejects_wrong_export_metadata() {
        let mut v = serde_json::json!({"format":{"duration":"61"},"streams":[{"codec_type":"video","codec_name":"h264","width":1080,"height":1920,"sample_aspect_ratio":"1:1","duration":"61"}]});
        assert!(verify_metadata(&v, 61., false).is_ok());
        assert!(verify_metadata(&v, 61., true).is_err());
        v["streams"][0]["codec_name"] = "hevc".into();
        assert!(verify_metadata(&v, 61., false).is_err());
    }
}
