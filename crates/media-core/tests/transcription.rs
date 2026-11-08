use clipper_media::{
    models,
    transcription::{
        check_whisper, extract_audio, load_transcript, transcribe, transcribe_chunked,
    },
    Tools,
};
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};
fn tools() -> Tools {
    Tools {
        ffmpeg: std::env::var_os("AI_CLIPPER_TEST_FFMPEG").unwrap().into(),
        ffprobe: std::env::var_os("AI_CLIPPER_TEST_FFPROBE").unwrap().into(),
        encoder: "libx264".into(),
    }
}
fn speech_video(tools: &Tools, dir: &std::path::Path, long: bool) -> PathBuf {
    let source = dir.join(if long {
        "long english video.mp4"
    } else {
        "english video.mp4"
    });
    let speech = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/jfk.wav");
    let mut cmd = Command::new(&tools.ffmpeg);
    cmd.args([
        "-v",
        "error",
        "-f",
        "lavfi",
        "-i",
        "color=c=green:s=160x90:r=2",
    ]);
    if long {
        cmd.args(["-stream_loop", "29"]);
    }
    cmd.arg("-i")
        .arg(speech)
        .args([
            "-shortest",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-c:a",
            "aac",
        ])
        .arg(&source);
    assert!(cmd.status().unwrap().success());
    source
}
#[test]
#[ignore = "Requires FFmpeg; explicitly run in CI"]
fn extracts_bounded_audio_from_long_video_and_rejects_corruption() {
    let tools = tools();
    let dir = tempfile::tempdir().unwrap();
    let source = speech_video(&tools, dir.path(), true);
    let info = tools.probe(&source, &AtomicBool::new(false)).unwrap();
    assert!(info.duration > 300.);
    let wav = dir.path().join("chunk.wav");
    extract_audio(&tools, &source, &wav, 0., 300., &AtomicBool::new(false)).unwrap();
    let bytes = std::fs::metadata(&wav).unwrap().len();
    assert!(bytes >= 9_600_000 && bytes < 9_610_000);
    let out = Command::new(&tools.ffprobe)
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(&wav)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["streams"][0]["sample_rate"], "16000");
    assert_eq!(json["streams"][0]["channels"], 1);
    assert_eq!(json["streams"][0]["codec_name"], "pcm_s16le");
    let bad = dir.path().join("bad.mp4");
    std::fs::write(&bad, b"broken audio").unwrap();
    let output = dir.path().join("bad.wav");
    assert!(extract_audio(&tools, &bad, &output, 0., 5., &AtomicBool::new(false)).is_err());
    assert!(!output.exists());
}
#[test]
#[ignore = "Requires the real base.en model and whisper.cpp; explicitly run in speech CI"]
fn real_english_speech_timestamps_restore_chunking_and_cancel() {
    let tools = tools();
    let root = PathBuf::from(std::env::var_os("AI_CLIPPER_TEST_MODEL_ROOT").expect("model root"));
    let whisper =
        PathBuf::from(std::env::var_os("AI_CLIPPER_TEST_WHISPER").expect("whisper binary"));
    check_whisper(&whisper).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let source = speech_video(&tools, dir.path(), false);
    let cancel = AtomicBool::new(false);
    let mut events = Vec::new();
    let transcript = transcribe(&tools, &whisper, &root, &source, &cancel, |p| {
        events.push(p.percent)
    })
    .unwrap();
    let text = transcript
        .segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    assert!(
        text.contains("ask not"),
        "unexpected English transcription: {text}"
    );
    assert!(text.contains("country"));
    assert!(transcript
        .segments
        .iter()
        .all(|s| s.start >= 0. && s.end <= transcript.duration + 0.05));
    assert!(transcript.segments.iter().any(|s| !s.words.is_empty()));
    assert_eq!(*events.last().unwrap(), 100.);
    assert!(events.windows(2).all(|p| p[0] <= p[1]));
    let restored = load_transcript(&root, &source).unwrap().unwrap();
    assert_eq!(restored.segments.len(), transcript.segments.len());
    let multi = transcribe_chunked(&tools, &whisper, &root, &source, &cancel, 5., |_| {}).unwrap();
    assert!(multi.segments.iter().any(|s| s.start >= 5.));
    assert!(multi.segments.windows(2).all(|s| s[0].start <= s[1].start));
    let project_file = root
        .join("projects")
        .join(format!("{}.json", multi.source_key));
    let saved = std::fs::read(&project_file).unwrap();
    let failed = transcribe_chunked(&tools, &whisper, &root, &source, &cancel, 5., |p| {
        if p.phase == "extracting" && p.chunk == 2 {
            cancel.store(true, Ordering::SeqCst);
        }
    });
    assert!(failed.is_err());
    let restored = load_transcript(&root, &source).unwrap().unwrap();
    assert_eq!(restored.segments.len(), multi.segments.len());
    assert_eq!(std::fs::read(&project_file).unwrap(), saved);
    assert_eq!(
        std::fs::read_dir(root.join("temporary")).unwrap().count(),
        0
    );
    let absent = tempfile::tempdir().unwrap();
    assert!(transcribe(
        &tools,
        &whisper,
        absent.path(),
        &source,
        &AtomicBool::new(false),
        |_| {}
    )
    .is_err());
    assert!(!models::model_status(absent.path()).ready);
}
