use clipper_media::{
    analysis::{self, Llama},
    audio, models,
    scenes::*,
    Tools,
};
use std::{path::PathBuf, sync::atomic::AtomicBool};
fn units() -> Vec<Unit> {
    [(18.,45.,"I bet my entire allowance that I can beat Max at the moon race. If I lose, I will wear the ridiculous chicken suit on tomorrow's stream. Remember this promise, Max, because I never lose this race."),(45.,55.,"Okay, the moon race is starting now. Let us focus on the game."),(170.,220.,"Max just crossed the finish line first. I lost the moon race! I promised to wear the chicken suit if I lost, so tomorrow I am wearing it. Max, you got me. Chat is never going to let me forget that ridiculous promise."),(220.,240.,"We are ending the moon race challenge here. Thanks for watching, everyone. See you tomorrow.")].into_iter().enumerate().map(|(id,(start,end,text))|Unit{id,start,end,text:text.into(),approximate:true}).collect()
}
#[test]
#[ignore = "Downloads actual 1.28 GB Qwen; explicitly run on free GitHub runner"]
fn real_qwen_semantic_linking_and_rejection() {
    let root = PathBuf::from(std::env::var_os("AI_CLIPPER_TEST_QWEN_ROOT").unwrap());
    let binary = PathBuf::from(std::env::var_os("AI_CLIPPER_TEST_LLAMA").unwrap());
    let model = models::spec_path(&root, models::QWEN);
    models::verify_file(
        &model,
        models::QWEN.bytes,
        models::QWEN.sha256,
        &AtomicBool::new(false),
    )
    .unwrap();
    analysis::check_llama(&binary).unwrap();
    let flag = AtomicBool::new(false);
    let mut llama = Llama {
        binary: &binary,
        model: &model,
        root: &root,
        cancel: &flag,
        cpu_fallback_used: false,
    };
    let u = units();
    let events = llama.detect(&u, &[0, 1, 2, 3], &[]).unwrap();
    println!(
        "Actual detected events: {}",
        serde_json::to_string(&events).unwrap()
    );
    assert!(!events.is_empty());
    let plan = Plan {
        id: "distant_77".into(),
        parts: vec![Span { first: 0, last: 0 }, Span { first: 2, last: 2 }],
        event_ids: vec![],
    };
    let j = llama.judge(&plan, &u).unwrap();
    println!(
        "Actual positive judgment: {}",
        serde_json::to_string(&j).unwrap()
    );
    let c = validate_candidate(&plan, j, &u, 240.)
        .expect("Qwen must recognize the actual promise and payoff");
    assert_eq!(c.total_duration, 77.);
    assert_eq!(c.mode, "smart_story");
    let mut unrelated = u.clone();
    unrelated[2].text="This recipe uses two cups of flour and three eggs. Stir the mixture until smooth, then put it in the oven for twenty minutes. My grandmother taught me to bake this bread when I was a child. The weather outside is beautiful today.".into();
    unrelated[3].text = "Let the loaf cool before slicing it.".into();
    let j = llama.judge(&plan, &unrelated).unwrap();
    println!(
        "Actual negative judgment: {}",
        serde_json::to_string(&j).unwrap()
    );
    assert!(
        validate_candidate(&plan, j, &unrelated, 240.).is_err(),
        "Qwen must reject unrelated recipe and game promise"
    );
    let single = Plan {
        id: "single_70".into(),
        parts: vec![Span { first: 2, last: 3 }],
        event_ids: vec![],
    };
    let j = llama.judge(&single, &u).unwrap();
    assert!(j.ratings.valid());
    assert_eq!(plan_segments(&single, &u, 240.).unwrap()[0].duration, 70.);
    // A pre-cancelled inference must never launch or allocate a model.
    let cancelled = AtomicBool::new(true);
    let mut stopped = Llama {
        binary: &binary,
        model: &model,
        root: &root,
        cancel: &cancelled,
        cpu_fallback_used: false,
    };
    assert!(stopped
        .detect(&u, &[0, 1], &[])
        .unwrap_err()
        .contains("abgebrochen"));
}
#[test]
#[ignore = "Requires FFmpeg, explicitly run in media-tests"]
fn real_audio_intensity_and_cleanup() {
    let tools = Tools {
        ffmpeg: std::env::var_os("AI_CLIPPER_TEST_FFMPEG").unwrap().into(),
        ffprobe: std::env::var_os("AI_CLIPPER_TEST_FFPROBE").unwrap().into(),
        encoder: "libx264".into(),
    };
    let root = tempfile::tempdir().unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/sample.mp4");
    let cues = audio::analyze_audio(
        &tools,
        &source,
        root.path(),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert!(!cues.is_empty());
    assert!(cues
        .iter()
        .all(|c| c.rms_db.is_finite() && (0.0..=1.0).contains(&c.silence_ratio)));
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    assert!(
        audio::analyze_audio(&tools, &source, root.path(), &AtomicBool::new(true), |_| {}).is_err()
    );
}
