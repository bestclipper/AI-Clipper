use crate::{
    scenes::Unit,
    transcription::{chunk_plan, extract_audio},
    Tools,
};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioCue {
    pub start: f64,
    pub end: f64,
    pub rms_db: f64,
    pub peak_db: f64,
    pub rise_db: f64,
    pub silence_ratio: f64,
}
pub fn analyze_audio(
    tools: &Tools,
    source: &Path,
    root: &Path,
    cancel: &AtomicBool,
    mut notify: impl FnMut(f64),
) -> Result<Vec<AudioCue>, String> {
    let info = tools.probe(source, cancel)?;
    if !info.has_audio {
        return Ok(vec![]);
    }
    let raw = tools.probe_json(source, cancel)?;
    let audio_end = raw["streams"]
        .as_array()
        .and_then(|s| s.iter().find(|s| s["codec_type"] == "audio"))
        .and_then(|a| {
            crate::number(&a["duration"])
                .map(|d| d + crate::number(&a["start_time"]).unwrap_or(0.).max(0.))
        })
        .unwrap_or(info.duration)
        .min(info.duration);
    let temp = tempfile::tempdir_in(root).map_err(|e| e.to_string())?;
    let wav = temp.path().join("intensity.wav");
    let mut cues = Vec::new();
    let mut previous = -90.;
    for (start, length) in chunk_plan(audio_end, 300.)? {
        if cancel.load(Ordering::SeqCst) {
            return Err("Analyse abgebrochen.".into());
        }
        extract_audio(tools, source, &wav, start, length, cancel)?;
        let mut reader = hound::WavReader::open(&wav)
            .map_err(|_| "Audiointensität konnte nicht gelesen werden.")?;
        let rate = reader.spec().sample_rate as usize;
        let mut sum = 0.;
        let mut peak = 0f64;
        let mut quiet = 0usize;
        let mut count = 0usize;
        let mut bucket = 0.;
        for sample in reader.samples::<i16>() {
            if count % 2048 == 0 && cancel.load(Ordering::SeqCst) {
                return Err("Analyse abgebrochen.".into());
            }
            let x = sample.map_err(|_| "Beschädigte Audioanalyse-Datei.")? as f64 / 32768.;
            sum += x * x;
            peak = peak.max(x.abs());
            if x.abs() < 0.008 {
                quiet += 1;
            }
            count += 1;
            if count == rate {
                let rms = 20. * (sum / count as f64).sqrt().max(0.00001).log10();
                cues.push(AudioCue {
                    start: start + bucket,
                    end: (start + bucket + 1.).min(audio_end),
                    rms_db: rms,
                    peak_db: 20. * peak.max(0.00001).log10(),
                    rise_db: (rms - previous).max(0.),
                    silence_ratio: quiet as f64 / count as f64,
                });
                previous = rms;
                sum = 0.;
                peak = 0.;
                quiet = 0;
                count = 0;
                bucket += 1.;
            }
        }
        if count > 0 {
            let rms = 20. * (sum / count as f64).sqrt().max(0.00001).log10();
            cues.push(AudioCue {
                start: start + bucket,
                end: (start + length).min(audio_end),
                rms_db: rms,
                peak_db: 20. * peak.max(0.00001).log10(),
                rise_db: (rms - previous).max(0.),
                silence_ratio: quiet as f64 / count as f64,
            });
            previous = rms;
        }
        std::fs::remove_file(&wav).map_err(|e| e.to_string())?;
        notify((start + length) / audio_end * 100.);
    }
    Ok(cues)
}
pub fn cue_for(unit: &Unit, cues: &[AudioCue]) -> String {
    let selected = cues
        .iter()
        .filter(|c| c.start < unit.end && c.end > unit.start)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return "audio unavailable".into();
    }
    let rms = selected.iter().map(|c| c.rms_db).sum::<f64>() / selected.len() as f64;
    let rise = selected.iter().map(|c| c.rise_db).fold(0., f64::max);
    let silence = selected.iter().map(|c| c.silence_ratio).sum::<f64>() / selected.len() as f64;
    format!("rms={rms:.0}dB rise={rise:.0}dB quiet={silence:.1}")
}
