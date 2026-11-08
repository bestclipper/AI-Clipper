use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

pub const MODEL_ID: &str = "base.en";
pub const MODEL_BYTES: u64 = 147_964_211;
pub const MODEL_SHA256: &str = "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002";
pub const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.en.bin";
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub id: String,
    pub ready: bool,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProgress {
    pub phase: String,
    pub received_bytes: u64,
    pub total_bytes: u64,
    pub percent: f64,
}
pub fn model_path(root: &Path) -> PathBuf {
    root.join("models/ggml-base.en.bin")
}
pub fn sha256_file(path: &Path, cancel: &AtomicBool) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err("Vorgang abgebrochen.".into());
        }
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub fn verify_file(path: &Path, bytes: u64, hash: &str, cancel: &AtomicBool) -> Result<(), String> {
    if fs::metadata(path)
        .map_err(|_| "Modell fehlt. Bitte zuerst base.en herunterladen.")?
        .len()
        != bytes
        || sha256_file(path, cancel)? != hash
    {
        return Err("Modell ist unvollständig oder beschädigt. Bitte erneut herunterladen.".into());
    }
    Ok(())
}
pub fn model_status(root: &Path) -> ModelStatus {
    ModelStatus {
        id: MODEL_ID.into(),
        ready: verify_file(
            &model_path(root),
            MODEL_BYTES,
            MODEL_SHA256,
            &AtomicBool::new(false),
        )
        .is_ok(),
        bytes: MODEL_BYTES,
        sha256: MODEL_SHA256.into(),
    }
}
pub async fn download_model(
    root: &Path,
    cancel: &AtomicBool,
    mut notify: impl FnMut(ModelProgress),
) -> Result<ModelStatus, String> {
    let path = model_path(root);
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let mut temp =
        tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let client = reqwest::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(20))
        .read_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(3600))
        .build()
        .map_err(|e| e.to_string())?;
    let response = tokio::select! { r=client.get(MODEL_URL).send()=>r.map_err(|e|format!("Modell-Download nicht erreichbar: {e}"))?, _=wait_cancel(cancel)=>return Err("Download abgebrochen.".into()) };
    let response = response
        .error_for_status()
        .map_err(|e| format!("Modell-Download fehlgeschlagen: {e}"))?;
    let mut stream = response.bytes_stream();
    let mut received = 0u64;
    let mut hash = Sha256::new();
    let mut last = 0.0;
    while let Some(chunk) = tokio::select! { data=stream.next()=>data, _=wait_cancel(cancel)=>return Err("Download abgebrochen.".into()) }
    {
        let chunk = chunk.map_err(|e| format!("Modell-Download unterbrochen: {e}"))?;
        received += chunk.len() as u64;
        if received > MODEL_BYTES {
            return Err("Download hat eine unerwartete Dateigröße.".into());
        }
        temp.write_all(&chunk)
            .map_err(|e| format!("Modell konnte nicht gespeichert werden: {e}"))?;
        hash.update(&chunk);
        let percent = received as f64 / MODEL_BYTES as f64 * 99.;
        if percent - last >= 0.25 || received == MODEL_BYTES {
            last = percent;
            notify(ModelProgress {
                phase: "downloading".into(),
                received_bytes: received,
                total_bytes: MODEL_BYTES,
                percent,
            });
        }
    }
    notify(ModelProgress {
        phase: "verifying".into(),
        received_bytes: received,
        total_bytes: MODEL_BYTES,
        percent: 99.,
    });
    if cancel.load(Ordering::SeqCst) {
        return Err("Download abgebrochen.".into());
    }
    if received != MODEL_BYTES || format!("{:x}", hash.finalize()) != MODEL_SHA256 {
        return Err(
            "Integritätsprüfung fehlgeschlagen. Das Modell wurde nicht installiert.".into(),
        );
    }
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    temp.persist(&path).map_err(|e| e.to_string())?;
    notify(ModelProgress {
        phase: "complete".into(),
        received_bytes: received,
        total_bytes: MODEL_BYTES,
        percent: 100.,
    });
    Ok(ModelStatus {
        id: MODEL_ID.into(),
        ready: true,
        bytes: MODEL_BYTES,
        sha256: MODEL_SHA256.into(),
    })
}
async fn wait_cancel(cancel: &AtomicBool) {
    loop {
        if cancel.load(Ordering::SeqCst) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integrity_missing_corrupt_and_valid() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("m.bin");
        let c = AtomicBool::new(false);
        assert!(verify_file(&p, 3, "x", &c).is_err());
        fs::write(&p, b"abc").unwrap();
        assert!(verify_file(&p, 4, "x", &c).is_err());
        assert!(verify_file(&p, 3, "bad", &c).is_err());
        assert!(verify_file(
            &p,
            3,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            &c
        )
        .is_ok());
        c.store(true, Ordering::SeqCst);
        assert!(sha256_file(&p, &c).is_err());
    }
}
