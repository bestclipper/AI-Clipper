use clipper_media::models::{download_spec, spec_status, QWEN, WHISPER};
use std::{path::PathBuf, sync::atomic::AtomicBool};
fn main() {
    let root = PathBuf::from(std::env::args().nth(1).expect("model root"));
    let spec = if std::env::args().nth(2).as_deref() == Some("qwen") {
        QWEN
    } else {
        WHISPER
    };
    if spec_status(&root, spec).ready {
        println!("Cached model verified: size and SHA-256 match.");
        return;
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut last = 0.;
    let result = runtime.block_on(download_spec(
        &root,
        if std::env::args().nth(2).as_deref() == Some("qwen") {
            QWEN
        } else {
            WHISPER
        },
        &AtomicBool::new(false),
        |p| {
            if p.percent - last >= 5. || p.percent == 100. {
                last = p.percent;
                println!("{}: {:.0}%", p.phase, p.percent);
            }
        },
    ));
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
