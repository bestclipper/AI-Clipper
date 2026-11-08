#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if std::env::args().any(|arg| arg == "--check-whisper") {
        match ai_clipper_lib::check_bundled_whisper() {
            Ok(()) => println!("Bundled whisper.cpp started successfully"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        return;
    }
    ai_clipper_lib::run();
}
