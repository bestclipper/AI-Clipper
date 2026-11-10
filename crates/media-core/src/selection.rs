//! Local, reproducible fallback: source text/audio signals, never invented story links.
use crate::{audio::AudioCue, scenes::*};
use std::collections::HashSet;
fn hits(text: &str, words: &[&str]) -> f64 {
    words.iter().filter(|w| text.contains(**w)).count() as f64
}
fn signals(text: &str) -> (f64, f64, f64) {
    let text = text.to_lowercase();
    (
        hits(
            &text,
            &[
                "laugh",
                "funny",
                "joke",
                "hilarious",
                "ridiculous",
                "embarrass",
                "punchline",
            ],
        ),
        hits(
            &text,
            &[
                "no way",
                "unexpected",
                "surprise",
                "can't believe",
                "cannot believe",
                "did not expect",
                "backfire",
                "what just",
            ],
        ),
        hits(
            &text,
            &[
                "excited",
                "angry",
                "scream",
                "cry",
                "insane",
                "unbelievable",
                "lost",
                "won",
            ],
        ),
    )
}
pub fn window_plans(units: &[Unit], windows: &[Vec<usize>], duration: f64) -> Vec<Plan> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for ids in windows {
        let mut best: Option<(f64, Plan)> = None;
        for &first in ids {
            for last in first..units.len() {
                let length = units[last].end - units[first].start;
                if length > 90.001 {
                    break;
                }
                if length < 61. {
                    continue;
                }
                let p = Plan {
                    id: format!("window_{first}_{last}"),
                    parts: vec![Span { first, last }],
                    event_ids: vec![],
                };
                if plan_segments(&p, units, duration).is_err() {
                    continue;
                }
                let text = span_text(units, &p.parts[0]).unwrap();
                let (humor, surprise, emotion) = signals(&text);
                let quality = humor * 2. + surprise * 2. + emotion - (length - 75.).abs() * 0.06;
                if best.as_ref().is_none_or(|(score, _)| quality > *score) {
                    best = Some((quality, p));
                }
            }
        }
        if let Some((_, p)) = best {
            if seen.insert((p.parts[0].first, p.parts[0].last)) {
                out.push(p);
            }
        }
    }
    // Every window contributes before a temporal round-robin bounds expensive comparisons.
    let mut buckets: Vec<Vec<Plan>> = (0..16).map(|_| vec![]).collect();
    for p in out {
        let bucket = ((units[p.parts[0].first].start / duration * 16.).floor() as usize).min(15);
        buckets[bucket].push(p);
    }
    let mut out = Vec::new();
    for index in 0..buckets.iter().map(Vec::len).max().unwrap_or(0) {
        for b in &buckets {
            if let Some(p) = b.get(index) {
                out.push(p.clone());
            }
        }
        if out.len() >= 160 {
            break;
        }
    }
    out.truncate(160);
    out
}
pub fn fallback(
    plan: &Plan,
    units: &[Unit],
    audio: &[AudioCue],
    duration: f64,
) -> Option<Candidate> {
    // No unverified semantic joins. Multi-part failures can only yield a continuous source scene.
    if plan.parts.len() != 1 {
        return None;
    }
    let span = &plan.parts[0];
    plan_segments(plan, units, duration).ok()?;
    let text = span_text(units, span)?;
    let (humor, surprise, emotion) = signals(&text);
    let first = units[span.first].text.trim();
    let last = units[span.last].text.trim();
    let hook = hits(
        &first.to_lowercase(),
        &[
            "watch",
            "bet",
            "promise",
            "guess",
            "challenge",
            "what",
            "why",
            "no way",
            "can't believe",
            "cannot believe",
        ],
    );
    let ending = hits(
        &last.to_lowercase(),
        &[
            "finally",
            "lost",
            "won",
            "result",
            "punchline",
            "turns out",
            "that is why",
            "that's why",
            "because",
            "never again",
        ],
    );
    // Loudness alone and fluent routine speech are not entertainment evidence.
    if humor + surprise + emotion < 2.
        || hook == 0.
        || ending == 0.
        || !last.ends_with(['.', '!', '?'])
    {
        return None;
    }
    let rise = audio
        .iter()
        .filter(|c| c.start < units[span.last].end && c.end > units[span.first].start)
        .map(|c| c.rise_db)
        .fold(0., f64::max);
    let boost = if rise >= 8. { 0.5 } else { 0. };
    let ratings = Ratings {
        entertainment: (6. + humor * 0.35 + surprise * 0.3 + boost).min(8.),
        surprise: (5.5 + surprise * 0.5 + emotion * 0.25).min(8.),
        hook: (6. + hook * 0.5).min(8.),
        clarity: 6.,
        tension: (5.5 + emotion * 0.3).min(7.5),
        ending: (6. + ending * 0.5).min(8.),
    };
    let quote = first.to_string();
    if quote.len() < 8 {
        return None;
    }
    let j=Judgment{accept:true,title:if humor>0.{"Humorous streamer moment"}else if surprise>0.{"Unexpected streamer moment"}else{"Emotional streamer moment"}.into(),category:"Local signal assessment".into(),reason:"Deterministic replacement assessment: source text contains humor, surprise or emotion, an opening hook and an ending. Semantic quality has not been verified by Qwen; preview the original scene.".into(),relation:"single".into(),connection_score:10.,context_safe:true,evidence:vec![quote],ratings};
    let mut candidate = validate_candidate(plan, j, units, duration).ok()?;
    candidate.evaluation_method = "deterministic".into();
    Some(candidate)
}
pub fn qualifies(c: &Candidate) -> bool {
    c.ratings.valid()
        && c.ratings.clarity >= 6.
        && c.ratings.hook >= 5.
        && c.ratings.ending >= 5.
        && (c.ratings.entertainment >= 6. || c.ratings.surprise >= 7.)
        && c.score >= 6.
}
pub fn strongly_overlap(a: &Candidate, b: &Candidate) -> bool {
    let intersection: f64 = a
        .segments
        .iter()
        .flat_map(|x| {
            b.segments
                .iter()
                .map(move |y| (x.end.min(y.end) - x.start.max(y.start)).max(0.))
        })
        .sum();
    intersection / a.total_duration.min(b.total_duration).max(0.001) > 0.2
}
/// Different boundaries can have different compatibility with a fifth scene.
/// Only exact source intervals are redundant before diversity selection.
pub fn rank_pool(candidates: &[Candidate]) -> Vec<Candidate> {
    let mut ranked = candidates
        .iter()
        .filter(|c| qualifies(c))
        .cloned()
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then(a.segments.len().cmp(&b.segments.len()))
            .then(a.id.cmp(&b.id))
    });
    let mut seen = HashSet::new();
    ranked.retain(|c| {
        seen.insert(
            c.segments
                .iter()
                .map(|s| (s.start.to_bits(), s.end.to_bits()))
                .collect::<Vec<_>>(),
        )
    });
    ranked
}
pub fn best_five(candidates: &[Candidate]) -> Vec<String> {
    let ranked = rank_pool(candidates);
    let mut beam: Vec<Vec<usize>> = vec![vec![]];
    for (index, c) in ranked.iter().enumerate() {
        let mut next = beam.clone();
        for chosen in &beam {
            if chosen.len() < 5 && chosen.iter().all(|id| !strongly_overlap(c, &ranked[*id])) {
                let mut group = chosen.clone();
                group.push(index);
                next.push(group);
            }
        }
        next.sort_by(|a, b| {
            b.len()
                .cmp(&a.len())
                .then_with(|| {
                    let score = |ids: &Vec<usize>| {
                        ids.iter()
                            .map(|i| {
                                ranked[*i].score
                                    - if (65. ..=80.).contains(&ranked[*i].total_duration) {
                                        0.
                                    } else {
                                        0.1
                                    }
                            })
                            .sum::<f64>()
                    };
                    score(b).total_cmp(&score(a))
                })
                .then(a.cmp(b))
        });
        next.dedup();
        // Preserve partial solutions even after some five-scene sets exist.
        let mut counts = [0usize; 6];
        next.retain(|group| {
            let count = &mut counts[group.len()];
            *count += 1;
            *count <= 128
        });
        beam = next;
    }
    beam.first()
        .into_iter()
        .flatten()
        .map(|i| ranked[*i].id.clone())
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn near_duplicate_boundaries_can_enable_the_fifth_scene() {
        let u=vec![Unit{id:0,start:0.,end:75.,text:"Watch this ridiculous bet! No way, I cannot believe the hilarious surprise. Finally I lost, that is the punchline!".into(),approximate:true}];
        let p = Plan {
            id: "base".into(),
            parts: vec![Span { first: 0, last: 0 }],
            event_ids: vec![],
        };
        let base = fallback(&p, &u, &[], 900.).unwrap();
        let mut candidates = vec![];
        for (i, start) in [100., 108., 48., 300., 450., 600.].into_iter().enumerate() {
            let mut c = base.clone();
            c.id = format!("variant{i}");
            c.segments[0].start = start;
            c.segments[0].end = start + 75.;
            c.score = 9.9 - i as f64 * 0.2;
            candidates.push(c);
        }
        assert_eq!(rank_pool(&candidates).len(), 6);
        let ids = best_five(&candidates);
        assert_eq!(ids.len(), 5);
        assert!(ids.contains(&"variant1".into()));
        assert!(ids.contains(&"variant2".into()));
        assert!(!ids.contains(&"variant0".into()));
    }
    #[test]
    fn five_distinct_scenes_win_over_a_high_scoring_overlapping_scene() {
        let u=vec![Unit{id:0,start:0.,end:75.,text:"Watch this ridiculous bet! No way, I cannot believe the hilarious surprise. Finally I lost, that is the punchline!".into(),approximate:true}];
        let p = Plan {
            id: "base".into(),
            parts: vec![Span { first: 0, last: 0 }],
            event_ids: vec![],
        };
        let base = fallback(&p, &u, &[], 900.).unwrap();
        let mut candidates = vec![];
        for (i, start) in [37.5, 0., 75., 300., 450., 600.].into_iter().enumerate() {
            let mut c = base.clone();
            c.id = format!("choice{i}");
            c.segments[0].start = start;
            c.segments[0].end = start + 75.;
            c.score = if i == 0 { 9.9 } else { 8. };
            candidates.push(c);
        }
        let ids = best_five(&candidates);
        assert_eq!(ids.len(), 5);
        assert!(!ids.contains(&"choice0".into()));
    }
}
