use crate::models::{model_path, verify_file, MODEL_BYTES, MODEL_ID, MODEL_SHA256};
use crate::{run, Tools};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub approximate: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub words: Vec<Word>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub schema_version: u32,
    pub source_key: String,
    pub source_path: String,
    pub model: String,
    pub language: String,
    pub engine: String,
    pub duration: f64,
    pub words_approximate: bool,
    pub alignment_status: String,
    pub segments: Vec<Segment>,
    pub cpu_fallback_used: bool,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionProgress {
    pub phase: String,
    pub percent: f64,
    pub chunk: usize,
    pub total_chunks: usize,
}
pub fn source_key(path: &Path) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|_| "Quelldatei nicht erreichbar.")?;
    let mut hash = Sha256::new();
    hash.update(
        path.canonicalize()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .as_bytes(),
    );
    hash.update(meta.len().to_le_bytes());
    hash.update(
        meta.modified()
            .map_err(|e| e.to_string())?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
            .to_le_bytes(),
    );
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = [0u8; 65536];
    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
    hash.update(&buffer[..n]);
    file.seek(SeekFrom::Start(meta.len().saturating_sub(65536)))
        .map_err(|e| e.to_string())?;
    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
    hash.update(&buffer[..n]);
    Ok(format!("{:x}", hash.finalize()))
}
fn transcript_path(root: &Path, key: &str) -> PathBuf {
    root.join("projects").join(format!("{key}.json"))
}
pub fn load_transcript(root: &Path, source: &Path) -> Result<Option<Transcript>, String> {
    let key = source_key(source)?;
    let path = transcript_path(root, &key);
    if !path.exists() {
        return Ok(None);
    }
    if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
        return Err("Gespeichertes Transkript ist zu groß.".into());
    }
    let t: Transcript =
        serde_json::from_reader(fs::File::open(path).map_err(|e| e.to_string())?)
            .map_err(|_| "Gespeichertes Transkript ist beschädigt. Bitte neu transkribieren.")?;
    if t.schema_version != 1 || t.source_key != key || !valid_segments(&t.segments, t.duration) {
        return Err("Gespeichertes Transkript enthält ungültige Zeitstempel.".into());
    }
    Ok(Some(t))
}
pub fn save_transcript(root: &Path, t: &Transcript) -> Result<(), String> {
    if !valid_segments(&t.segments, t.duration) {
        return Err("Ungültige Transkript-Zeitstempel.".into());
    }
    let dir = root.join("projects");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut f = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut f, t).map_err(|e| e.to_string())?;
    if f.as_file().metadata().map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
        return Err("Transkript überschreitet die unterstützte Projektgröße.".into());
    }
    f.as_file().sync_all().map_err(|e| e.to_string())?;
    f.persist(transcript_path(root, &t.source_key))
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn chunk_plan(duration: f64, chunk_secs: f64) -> Result<Vec<(f64, f64)>, String> {
    if !duration.is_finite()
        || duration <= 0.
        || !chunk_secs.is_finite()
        || chunk_secs <= 0.
        || chunk_secs > 300.
        || duration > 7. * 24. * 3600.
    {
        return Err("Ungültige Videodauer (maximal 7 Tage).".into());
    }
    let count = (duration / chunk_secs).ceil() as usize;
    let width = duration / count as f64;
    let chunks = (0..count)
        .map(|index| {
            let start = index as f64 * width;
            (start, (duration - start).min(width))
        })
        .collect();
    Ok(chunks)
}
fn valid_segments(segments: &[Segment], duration: f64) -> bool {
    duration.is_finite()
        && duration > 0.
        && segments.iter().all(|s| {
            s.start.is_finite()
                && s.end.is_finite()
                && s.start >= 0.
                && s.end >= s.start
                && s.end <= duration + 0.05
                && s.words.iter().all(|w| {
                    w.start.is_finite()
                        && w.end.is_finite()
                        && w.start >= s.start
                        && w.end >= w.start
                        && w.end <= s.end + 0.01
                })
        })
        && segments.windows(2).all(|s| s[0].start <= s[1].start)
}
// whisper.cpp JSON offsets are milliseconds, not samples or centiseconds.
pub fn parse_segments(
    raw: &serde_json::Value,
    offset: f64,
    length: f64,
) -> Result<Vec<Segment>, String> {
    let values = raw["transcription"]
        .as_array()
        .ok_or("Whisper hat kein gültiges JSON-Transkript erzeugt.")?;
    let mut result = Vec::new();
    for value in values {
        let text = value["text"]
            .as_str()
            .ok_or("Transkript-Text fehlt.")?
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }
        let start = value["offsets"]["from"]
            .as_f64()
            .ok_or("Segmentstart fehlt.")?
            / 1000.;
        let end = value["offsets"]["to"]
            .as_f64()
            .ok_or("Segmentende fehlt.")?
            / 1000.;
        if !start.is_finite()
            || !end.is_finite()
            || start < 0.
            || end < start
            || start > length + 0.1
        {
            return Err("Whisper-Zeitstempel sind ungültig.".into());
        }
        let start = offset + start.min(length);
        let end = offset + end.min(length);
        let mut words: Vec<Word> = Vec::new();
        for token in value["tokens"].as_array().into_iter().flatten() {
            let Some(text) = token["text"].as_str() else {
                continue;
            };
            if text.starts_with("[_") || text.starts_with("<|") || text.is_empty() {
                continue;
            }
            let (Some(a), Some(b)) = (
                token["offsets"]["from"].as_f64(),
                token["offsets"]["to"].as_f64(),
            ) else {
                continue;
            };
            if a < 0. || b < a {
                continue;
            }
            let a = (offset + a / 1000.).clamp(start, end);
            let b = (offset + b / 1000.).clamp(a, end);
            if text.starts_with(char::is_whitespace) || words.is_empty() {
                words.push(Word {
                    text: text.trim().into(),
                    start: a,
                    end: b,
                    approximate: true,
                });
            } else if let Some(word) = words.last_mut() {
                word.text.push_str(text);
                word.end = word.end.max(b);
            }
        }
        result.push(Segment {
            text,
            start,
            end,
            words,
        });
    }
    if !valid_segments(&result, offset + length) {
        return Err("Transkript-Zeitstempel konnten nicht validiert werden.".into());
    }
    Ok(result)
}
pub fn extract_audio(
    tools: &Tools,
    source: &Path,
    wav: &Path,
    start: f64,
    length: f64,
    cancel: &AtomicBool,
) -> Result<(), String> {
    if !start.is_finite() || start < 0. || !length.is_finite() || length <= 0. || length > 300. {
        return Err("Ungültiger Audioabschnitt.".into());
    }
    let args = [
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-y",
        "-threads",
        "2",
        "-protocol_whitelist",
        "file,pipe",
        "-ss",
    ]
    .iter()
    .map(|s| s.to_string())
    .chain([
        start.to_string(),
        "-i".into(),
        source.to_string_lossy().into_owned(),
        "-t".into(),
        length.to_string(),
    ])
    .chain(
        [
            "-map",
            "0:a:0",
            "-vn",
            "-sn",
            "-dn",
            "-af",
            "aresample=async=1:first_pts=0",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-c:a",
            "pcm_s16le",
            "-f",
            "wav",
        ]
        .iter()
        .map(|s| s.to_string()),
    )
    .chain([wav.to_string_lossy().into_owned()])
    .collect::<Vec<_>>();
    let result = run(
        &tools.ffmpeg,
        &args,
        cancel,
        |_| {},
        Duration::from_secs(600),
    )
    .map(|_| ());
    if result.is_err() {
        let _ = fs::remove_file(wav);
    }
    result
}
pub fn transcribe(
    tools: &Tools,
    whisper: &Path,
    root: &Path,
    source: &Path,
    cancel: &AtomicBool,
    notify: impl FnMut(TranscriptionProgress),
) -> Result<Transcript, String> {
    transcribe_chunked(tools, whisper, root, source, cancel, 300., notify)
}
pub fn transcribe_chunked(
    tools: &Tools,
    whisper: &Path,
    root: &Path,
    source: &Path,
    cancel: &AtomicBool,
    chunk_secs: f64,
    mut notify: impl FnMut(TranscriptionProgress),
) -> Result<Transcript, String> {
    verify_file(&model_path(root), MODEL_BYTES, MODEL_SHA256, cancel)?;
    let info = tools.probe(source, cancel)?;
    if !info.has_audio {
        return Err("Das Video hat keine Tonspur und kann nicht transkribiert werden.".into());
    }
    let key = source_key(source)?;
    let raw = tools.probe_json(source, cancel)?;
    let audio = raw["streams"]
        .as_array()
        .and_then(|streams| streams.iter().find(|s| s["codec_type"] == "audio"));
    let audio_end = audio
        .and_then(|a| {
            crate::number(&a["duration"])
                .map(|d| d + crate::number(&a["start_time"]).unwrap_or(0.).max(0.))
        })
        .unwrap_or(info.duration)
        .min(info.duration);
    let plan = chunk_plan(audio_end, chunk_secs)?;
    let temp_parent = root.join("temporary");
    fs::create_dir_all(&temp_parent).map_err(|e| e.to_string())?;
    let temp = tempfile::tempdir_in(&temp_parent).map_err(|e| e.to_string())?;
    let mut transcript = Transcript {
        schema_version: 1,
        source_key: key.clone(),
        source_path: source.to_string_lossy().into_owned(),
        model: MODEL_ID.into(),
        language: "en".into(),
        engine: "whisper.cpp v1.8.7".into(),
        duration: info.duration,
        words_approximate: true,
        alignment_status: "unreviewed".into(),
        segments: Vec::new(),
        cpu_fallback_used: false,
    };
    for (index, (start, length)) in plan.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err("Transkription abgebrochen.".into());
        }
        let base = 100. * start / audio_end;
        notify(TranscriptionProgress {
            phase: "extracting".into(),
            percent: base.min(99.),
            chunk: index + 1,
            total_chunks: plan.len(),
        });
        let wav = temp.path().join("chunk.wav");
        extract_audio(tools, source, &wav, *start, *length, cancel)?;
        let output = temp.path().join("transcript");
        let json = output.with_extension("json");
        let _ = fs::remove_file(&json);
        let mut args = vec![
            "-m".into(),
            model_path(root).to_string_lossy().into_owned(),
            "-f".into(),
            wav.to_string_lossy().into_owned(),
            "-l".into(),
            "en".into(),
            "-t".into(),
            "4".into(),
            "-p".into(),
            "1".into(),
            "-bs".into(),
            "1".into(),
            "-bo".into(),
            "1".into(),
            "-ojf".into(),
            "-pp".into(),
            "-of".into(),
            output.to_string_lossy().into_owned(),
        ];
        let mut last = base;
        let mut callback = |line: &str| {
            let p = line
                .split("progress =")
                .nth(1)
                .or_else(|| line.split("progress=").nth(1))
                .and_then(|s| s.trim().trim_end_matches('%').parse::<f64>().ok())
                .unwrap_or(0.)
                .clamp(0., 100.);
            let percent = (100. * (start + length * p / 100.) / audio_end)
                .min(99.)
                .max(last);
            last = percent;
            notify(TranscriptionProgress {
                phase: "transcribing".into(),
                percent,
                chunk: index + 1,
                total_chunks: plan.len(),
            });
        };
        callback("");
        let first = run(
            whisper,
            &args,
            cancel,
            &mut callback,
            Duration::from_secs(1800),
        );
        if first.is_err() && !cancel.load(Ordering::SeqCst) {
            args.push("-ng".into());
            let _ = fs::remove_file(&json);
            transcript.cpu_fallback_used = true;
            run(
                whisper,
                &args,
                cancel,
                &mut callback,
                Duration::from_secs(1800),
            )
            .map_err(|e| {
                format!("Whisper konnte den Abschnitt auch im CPU-Modus nicht transkribieren: {e}")
            })?;
        } else {
            first?;
        }
        if fs::metadata(&json)
            .map_err(|_| "Whisper hat keine Ergebnisdatei erzeugt.")?
            .len()
            > 16 * 1024 * 1024
        {
            return Err("Whisper-Ergebnis ist unerwartet groß.".into());
        }
        let raw: serde_json::Value =
            serde_json::from_reader(fs::File::open(&json).map_err(|e| e.to_string())?)
                .map_err(|_| "Whisper-Ergebnis ist beschädigt.")?;
        transcript
            .segments
            .extend(parse_segments(&raw, *start, *length)?);
        fs::remove_file(wav).map_err(|e| e.to_string())?;
        fs::remove_file(json).map_err(|e| e.to_string())?;
    }
    if cancel.load(Ordering::SeqCst) {
        return Err("Transkription abgebrochen.".into());
    }
    if source_key(source)? != key {
        return Err("Die Quelldatei wurde während der Transkription verändert. Das Ergebnis wurde nicht gespeichert.".into());
    }
    save_transcript(root, &transcript)?;
    notify(TranscriptionProgress {
        phase: "complete".into(),
        percent: 100.,
        chunk: plan.len(),
        total_chunks: plan.len(),
    });
    Ok(transcript)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn long_audio_plan_is_bounded() {
        let p = chunk_plan(10815., 300.).unwrap();
        assert_eq!(p.len(), 37);
        assert!((p[36].0 + p[36].1 - 10815.).abs() < 0.01);
        assert!(p.iter().all(|(_, d)| *d <= 300.));
        assert!(chunk_plan(f64::NAN, 300.).is_err());
    }
    #[test]
    fn parses_milliseconds_and_approximate_words() {
        let v = serde_json::json!({"transcription":[{"text":" Hello world","offsets":{"from":100,"to":2000},"tokens":[{"text":" Hello","offsets":{"from":100,"to":800}},{"text":" world","offsets":{"from":800,"to":1900}}]}]});
        let s = parse_segments(&v, 300., 5.).unwrap();
        assert_eq!(s[0].start, 300.1);
        assert_eq!(s[0].end, 302.);
        assert_eq!(s[0].words.len(), 2);
        assert!(s[0].words[0].approximate);
        let bad =
            serde_json::json!({"transcription":[{"text":"bad","offsets":{"from":2000,"to":100}}]});
        assert!(parse_segments(&bad, 0., 5.).is_err());
    }
    #[test]
    fn save_load_and_source_change() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("v.mp4");
        fs::write(&source, b"video").unwrap();
        assert!(load_transcript(dir.path(), &source).unwrap().is_none());
        let t = Transcript {
            schema_version: 1,
            source_key: source_key(&source).unwrap(),
            source_path: source.to_string_lossy().into_owned(),
            model: MODEL_ID.into(),
            language: "en".into(),
            engine: "test".into(),
            duration: 10.,
            words_approximate: true,
            alignment_status: "unreviewed".into(),
            segments: vec![],
            cpu_fallback_used: false,
        };
        save_transcript(dir.path(), &t).unwrap();
        assert!(load_transcript(dir.path(), &source).unwrap().is_some());
        fs::write(&source, b"changed").unwrap();
        assert!(load_transcript(dir.path(), &source).unwrap().is_none());
    }
}

pub fn check_whisper(path: &Path) -> Result<(), String> {
    let mut recognized = false;
    run(
        path,
        &["-h".into()],
        &AtomicBool::new(false),
        |line| {
            if line.contains("--model") {
                recognized = true;
            }
        },
        Duration::from_secs(15),
    )?;
    if !recognized {
        return Err("Die mitgelieferte Whisper-Komponente antwortet nicht wie erwartet.".into());
    }
    Ok(())
}
