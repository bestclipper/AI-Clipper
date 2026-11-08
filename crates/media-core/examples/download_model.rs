use clipper_media::models::download_model;
use std::{path::PathBuf, sync::atomic::AtomicBool};
fn main() {
    let root = PathBuf::from(std::env::args().nth(1).expect("model root"));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut last = 0.;
    let result = runtime.block_on(download_model(&root, &AtomicBool::new(false), |p| {
        if p.percent - last >= 5. || p.percent == 100. {
            last = p.percent;
            println!("{}: {:.0}%", p.phase, p.percent);
        }
    }));
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
