use clipper_media::Tools;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
fn tools() -> Tools {
    Tools {
        ffmpeg: std::env::var_os("AI_CLIPPER_TEST_FFMPEG")
            .expect("Set AI_CLIPPER_TEST_FFMPEG")
            .into(),
        ffprobe: std::env::var_os("AI_CLIPPER_TEST_FFPROBE")
            .expect("Set AI_CLIPPER_TEST_FFPROBE")
            .into(),
        encoder: "libx264".into(),
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "ai clipper test {}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixture(tools: &Tools, dir: &Temp, audio: bool, portrait: bool) -> PathBuf {
    let p = dir.0.join("source video.mp4");
    let mut cmd = Command::new(&tools.ffmpeg);
    cmd.args([
        "-v",
        "error",
        "-f",
        "lavfi",
        "-i",
        if portrait {
            "color=c=blue:s=180x320:r=10:d=63"
        } else {
            "color=c=red:s=320x180:r=10:d=63"
        },
    ]);
    if audio {
        cmd.args([
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=63",
        ]);
    }
    cmd.args([
        "-c:v",
        "libx264",
        "-preset",
        "ultrafast",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
    ])
    .arg(&p);
    assert!(cmd.status().unwrap().success());
    p
}
#[test]
#[ignore = "Requires real FFmpeg/FFprobe; CI explicitly runs ignored integration tests"]
fn cuts_real_video_preserves_audio_and_checks_output() {
    let t = tools();
    let dir = Temp::new();
    let source = fixture(&t, &dir, true, false);
    let output = dir.0.join("clip with spaces.mp4");
    let cancel = AtomicBool::new(false);
    let mut events = Vec::new();
    let result = t
        .export(&source, &output, 1., 61., &cancel, |p| events.push(p))
        .unwrap();
    assert_eq!((result.width, result.height), (1080, 1920));
    assert!(result.has_audio);
    assert!((result.duration - 61.).abs() < 0.1);
    assert_eq!(events.last().unwrap().percent, 100.);
    assert!(events.iter().any(|p| p.phase == "verifying"));
    assert!(events.windows(2).all(|p| p[0].percent <= p[1].percent));
    // Existing exports and original source must never be overwritten.
    let before = fs::read(&output).unwrap();
    assert!(t
        .export(&source, &output, 0., 61., &cancel, |_| {})
        .is_err());
    assert_eq!(before, fs::read(&output).unwrap());
    let original = fs::metadata(&source).unwrap().len();
    assert!(t
        .export(&source, &source, 0., 61., &cancel, |_| {})
        .is_err());
    assert_eq!(original, fs::metadata(&source).unwrap().len());
}
#[test]
#[ignore = "Requires real FFmpeg/FFprobe; CI explicitly runs ignored integration tests"]
fn handles_portrait_silent_source_and_cancellation() {
    let t = tools();
    let dir = Temp::new();
    let source = fixture(&t, &dir, false, true);
    let cancel = AtomicBool::new(false);
    let output = dir.0.join("silent.mp4");
    let result = t
        .export(&source, &output, 0., 61., &cancel, |_| {})
        .unwrap();
    assert!(!result.has_audio);
    let partial = dir.0.join("cancelled.mp4");
    assert!(t
        .export(&source, &partial, 0., 61., &cancel, |p| {
            if p.phase == "encoding" && p.percent > 1.0 {
                cancel.store(true, Ordering::SeqCst);
            }
        })
        .is_err());
    assert!(!partial.exists());
    cancel.store(false, Ordering::SeqCst);
    let invalid = dir.0.join("invalid.mp4");
    assert!(t
        .export(&source, &invalid, 10., 61., &cancel, |_| {})
        .is_err());
    assert!(!invalid.exists());
    let corrupt = dir.0.join("broken.mp4");
    fs::write(&corrupt, b"not a video").unwrap();
    assert!(t
        .export(&corrupt, &invalid, 0., 61., &cancel, |_| {})
        .is_err());
    assert!(!invalid.exists());
}
