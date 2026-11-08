use crate::transcription::Transcript;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub id: usize,
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub approximate: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Span {
    pub first: usize,
    pub last: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Event {
    pub first: usize,
    pub last: usize,
    pub summary: String,
    pub topic: String,
    pub people: Vec<String>,
    pub role: String,
    pub interest: f64,
    pub evidence: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub id: String,
    pub parts: Vec<Span>,
    pub event_ids: Vec<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Ratings {
    pub entertainment: f64,
    pub surprise: f64,
    pub hook: f64,
    pub clarity: f64,
    pub tension: f64,
    pub ending: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Judgment {
    pub accept: bool,
    pub title: String,
    pub category: String,
    pub reason: String,
    pub relation: String,
    pub connection_score: f64,
    pub context_safe: bool,
    pub evidence: Vec<String>,
    pub ratings: Ratings,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipSegment {
    pub first_unit: usize,
    pub last_unit: usize,
    pub start: f64,
    pub end: f64,
    pub duration: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: String,
    pub title: String,
    pub mode: String,
    pub segments: Vec<ClipSegment>,
    pub total_duration: f64,
    pub score: f64,
    pub category: String,
    pub reason: String,
    pub relation: String,
    pub connection_score: f64,
    pub ratings: Ratings,
    pub evidence: Vec<String>,
    pub timestamps_approximate: bool,
}
fn normalized(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
pub fn make_units(t: &Transcript) -> Vec<Unit> {
    let mut units = Vec::new();
    for s in &t.segments {
        if s.text.trim().is_empty() {
            continue;
        }
        let complete = !s.words.is_empty()
            && normalized(
                &s.words
                    .iter()
                    .map(|w| w.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            ) == normalized(&s.text);
        if complete {
            let mut first = 0;
            for index in 0..s.words.len() {
                let text = s.words[index].text.trim();
                if text.ends_with(['.', '?', '!']) || index == s.words.len() - 1 {
                    let words = &s.words[first..=index];
                    let start = if first == 0 { s.start } else { words[0].start };
                    let end = if index == s.words.len() - 1 {
                        s.end
                    } else {
                        words.last().unwrap().end
                    };
                    if end > start {
                        units.push(Unit {
                            id: units.len(),
                            start,
                            end,
                            text: words
                                .iter()
                                .map(|w| w.text.as_str())
                                .collect::<Vec<_>>()
                                .join(" "),
                            approximate: true,
                        });
                    }
                    first = index + 1;
                }
            }
        } else {
            units.push(Unit {
                id: units.len(),
                start: s.start,
                end: s.end,
                text: s.text.clone(),
                approximate: true,
            });
        }
    }
    units.retain(|u| {
        u.start.is_finite()
            && u.end.is_finite()
            && u.start >= 0.
            && u.end > u.start
            && u.end <= t.duration + 0.01
    });
    for (id, u) in units.iter_mut().enumerate() {
        u.id = id;
    }
    units
}
pub fn windows(units: &[Unit]) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    let mut at = 0;
    while at < units.len() {
        let mut bytes = 0;
        let mut end = at;
        while end < units.len() && end - at < 16 {
            let size = units[end].text.len().min(1000) + 70;
            if end > at && bytes + size > 1500 {
                break;
            }
            bytes += size;
            end += 1;
        }
        result.push((at..end).collect());
        if end == units.len() {
            break;
        }
        at = if end - at > 2 { end - 2 } else { end };
    }
    result
}
pub fn span_text(units: &[Unit], span: &Span) -> Option<String> {
    if span.first > span.last || span.last >= units.len() {
        return None;
    }
    Some(
        units[span.first..=span.last]
            .iter()
            .map(|u| u.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    )
}
pub fn event_valid(e: &Event, units: &[Unit], allowed: &[usize]) -> bool {
    let Some(text) = span_text(
        units,
        &Span {
            first: e.first,
            last: e.last,
        },
    ) else {
        return false;
    };
    let set: HashSet<_> = allowed.iter().copied().collect();
    (e.first..=e.last).all(|id| set.contains(&id))
        && e.interest.is_finite()
        && (1.0..=10.0).contains(&e.interest)
        && e.summary.len() <= 240
        && e.topic.len() <= 100
        && e.people.len() <= 5
        && e.evidence.trim().len() >= 8
        && text
            .to_lowercase()
            .contains(&e.evidence.trim().to_lowercase())
        && units[e.last].end - units[e.first].start <= 150.
}
impl Ratings {
    pub fn valid(&self) -> bool {
        [
            self.entertainment,
            self.surprise,
            self.hook,
            self.clarity,
            self.tension,
            self.ending,
        ]
        .into_iter()
        .all(|x| x.is_finite() && (1.0..=10.0).contains(&x))
    }
    pub fn weighted(&self) -> f64 {
        self.entertainment * 0.25
            + self.surprise * 0.20
            + self.hook * 0.20
            + self.clarity * 0.15
            + self.tension * 0.10
            + self.ending * 0.10
    }
}
pub fn plan_segments(
    plan: &Plan,
    units: &[Unit],
    duration: f64,
) -> Result<Vec<ClipSegment>, String> {
    if plan.parts.is_empty() || plan.parts.len() > 5 {
        return Err("Ein Clip muss 1–5 Segmente enthalten.".into());
    }
    let mut out: Vec<ClipSegment> = Vec::new();
    for p in &plan.parts {
        if p.first > p.last || p.last >= units.len() {
            return Err("Unbekannte Transkript-Zeitstempel.".into());
        }
        let start = units[p.first].start;
        let end = units[p.last].end;
        if !start.is_finite()
            || !end.is_finite()
            || start < 0.
            || end <= start
            || end > duration + 0.001
        {
            return Err("Zeitbereich liegt außerhalb des Videos.".into());
        }
        if let Some(previous) = out.last() {
            if start < previous.end - 0.0001 || p.first <= previous.last_unit {
                return Err("Segmente überlappen oder sind nicht chronologisch.".into());
            }
        }
        out.push(ClipSegment {
            first_unit: p.first,
            last_unit: p.last,
            start,
            end,
            duration: end - start,
        });
    }
    let total: f64 = out.iter().map(|s| s.duration).sum();
    if !(61.0 - 0.001..=90.0 + 0.001).contains(&total) {
        return Err("Die Summe der Segmente muss 61–90 Sekunden betragen.".into());
    }
    Ok(out)
}
pub fn validate_candidate(
    plan: &Plan,
    j: Judgment,
    units: &[Unit],
    duration: f64,
) -> Result<Candidate, String> {
    let parts = plan_segments(plan, units, duration)?;
    if !j.accept
        || !j.context_safe
        || !j.ratings.valid()
        || j.ratings.clarity < 6.
        || j.title.trim().is_empty()
        || j.title.len() > 160
        || j.reason.trim().len() < 12
        || j.reason.len() > 800
        || j.category.len() > 100
        || j.evidence.len() != parts.len()
    {
        return Err("Die KI-Auswahl ist nicht ausreichend begründet oder verständlich.".into());
    }
    for (part, quote) in plan.parts.iter().zip(&j.evidence) {
        if quote.trim().len() < 8
            || !span_text(units, part)
                .unwrap()
                .to_lowercase()
                .contains(&quote.trim().to_lowercase())
        {
            return Err(
                "Die Begründung enthält keine belegbare Originalaussage pro Segment.".into(),
            );
        }
    }
    let smart = parts.len() > 1;
    if smart
        && (!j.connection_score.is_finite()
            || !(7.0..=10.).contains(&j.connection_score)
            || !matches!(
                j.relation.as_str(),
                "setup_payoff" | "claim_reaction" | "question_answer" | "conflict_resolution"
            ))
    {
        return Err("Smart Story hat keinen verlässlichen inhaltlichen Zusammenhang.".into());
    }
    let base = j.ratings.weighted();
    let score = if smart {
        base.min(j.connection_score) - 0.08 * (parts.len() - 1) as f64
    } else {
        base
    };
    let total = parts.iter().map(|s| s.duration).sum();
    Ok(Candidate {
        id: plan.id.clone(),
        title: j.title,
        mode: if smart { "smart_story" } else { "single_scene" }.into(),
        segments: parts,
        total_duration: total,
        score: (score.clamp(1., 10.) * 10.).round() / 10.,
        category: j.category,
        reason: j.reason,
        relation: j.relation,
        connection_score: if smart { j.connection_score } else { 10. },
        ratings: j.ratings,
        evidence: j.evidence,
        timestamps_approximate: true,
    })
}
fn overlap(a: &Candidate, b: &Candidate) -> f64 {
    let intersection: f64 = a
        .segments
        .iter()
        .flat_map(|x| {
            b.segments
                .iter()
                .map(move |y| (x.end.min(y.end) - x.start.max(y.start)).max(0.))
        })
        .sum();
    intersection / (a.total_duration + b.total_duration - intersection).max(0.001)
}
pub fn rank_deduplicate(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    candidates.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.segments.len().cmp(&b.segments.len()))
            .then_with(|| {
                (a.total_duration - 75.)
                    .abs()
                    .total_cmp(&(b.total_duration - 75.).abs())
            })
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut result = Vec::new();
    for candidate in candidates {
        if !result.iter().any(|other| overlap(&candidate, other) >= 0.8) {
            result.push(candidate);
        }
        if result.len() >= 50 {
            break;
        }
    }
    result
}
fn terms(s: &str) -> BTreeSet<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|s| {
            s.len() > 3
                && !matches!(
                    s.as_str(),
                    "this"
                        | "that"
                        | "they"
                        | "their"
                        | "with"
                        | "about"
                        | "speaker"
                        | "reaction"
                        | "says"
                )
        })
        .collect()
}
pub fn related_events(anchor: usize, events: &[Event]) -> Vec<usize> {
    let a = terms(&format!(
        "{} {} {}",
        events[anchor].topic,
        events[anchor].summary,
        events[anchor].people.join(" ")
    ));
    let mut scored: Vec<_> = events
        .iter()
        .enumerate()
        .filter(|(i, e)| {
            *i != anchor && (e.last < events[anchor].first || e.first > events[anchor].last)
        })
        .map(|(i, e)| {
            let b = terms(&format!("{} {} {}", e.topic, e.summary, e.people.join(" ")));
            let shared = a.intersection(&b).count() as f64;
            let union = a.union(&b).count().max(1) as f64;
            (i, shared / union)
        })
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut ids = scored.iter().take(4).map(|s| s.0).collect::<Vec<_>>();
    if let Some((far, _)) = scored
        .iter()
        .max_by_key(|(i, _)| events[*i].first.abs_diff(events[anchor].first))
    {
        if !ids.contains(far) {
            ids.push(*far);
        }
    }
    ids
}
fn options(event: &Event, units: &[Unit]) -> Vec<Span> {
    let mut spans = Vec::new();
    for first in event.first..=event.last {
        for last in first..=event.last {
            let span = Span { first, last };
            let duration = units[last].end - units[first].start;
            if (6.0..=80.).contains(&duration)
                && span_text(units, &span)
                    .unwrap()
                    .to_lowercase()
                    .contains(&event.evidence.to_lowercase())
            {
                spans.push(span);
            }
        }
    }
    spans.sort_by(|a, b| {
        let x = (units[a.last].end - units[a.first].start - 35.).abs();
        let y = (units[b.last].end - units[b.first].start - 35.).abs();
        x.total_cmp(&y).then(a.first.cmp(&b.first))
    });
    spans.truncate(12);
    spans
}
pub fn build_plans(events: &[Event], units: &[Unit], duration: f64) -> Vec<Plan> {
    let mut plans = Vec::new();
    let mut seen = HashSet::new();
    let mut anchors: Vec<_> = (0..events.len()).collect();
    anchors.sort_by(|a, b| {
        events[*b]
            .interest
            .total_cmp(&events[*a].interest)
            .then(a.cmp(b))
    }); // Global events, never a fixed distance limit.
        // Temporal strata protect late-stream events from being crowded out by the beginning.
    let mut diverse = Vec::new();
    for bucket in 0..8 {
        for id in anchors
            .iter()
            .filter(|id| {
                ((units[events[**id].first].start / duration * 8.).floor() as usize).min(7)
                    == bucket
            })
            .take(4)
        {
            diverse.push(*id);
        }
    }
    for id in anchors {
        if !diverse.contains(&id) {
            diverse.push(id);
        }
    }
    diverse.truncate(32);
    for anchor in diverse {
        let e = &events[anchor];
        let mut singles = Vec::new();
        for first in e.first.saturating_sub(2)..=e.first {
            for last in e.last..=(e.last + 2).min(units.len().saturating_sub(1)) {
                let p = Plan {
                    id: String::new(),
                    parts: vec![Span { first, last }],
                    event_ids: vec![anchor],
                };
                if plan_segments(&p, units, duration).is_ok() {
                    singles.push(p);
                }
            }
        }
        singles.sort_by(|a, b| {
            let a = (units[a.parts[0].last].end - units[a.parts[0].first].start - 75.).abs();
            let b = (units[b.parts[0].last].end - units[b.parts[0].first].start - 75.).abs();
            a.total_cmp(&b)
        });
        for plan in singles.into_iter().take(1) {
            push_plan(plan, &mut plans, &mut seen);
        }
        let neighbors = related_events(anchor, events);
        let mut groups: Vec<Vec<usize>> = neighbors.iter().map(|id| vec![anchor, *id]).collect();
        for count in 2..=4 {
            if neighbors.len() >= count {
                let mut group = vec![anchor];
                group.extend_from_slice(&neighbors[..count]);
                groups.push(group);
            }
        }
        for mut group in groups {
            group.sort_by_key(|id| events[*id].first);
            let mut beam = vec![Vec::<Span>::new()];
            for id in &group {
                let opts = options(&events[*id], units);
                let mut next = Vec::new();
                for parts in &beam {
                    for opt in &opts {
                        if parts.last().is_some_and(|p| {
                            p.last >= opt.first || units[p.last].end > units[opt.first].start
                        }) {
                            continue;
                        }
                        let mut combined = parts.clone();
                        combined.push(opt.clone());
                        let total: f64 = combined
                            .iter()
                            .map(|s| units[s.last].end - units[s.first].start)
                            .sum();
                        if total <= 90.001 {
                            next.push(combined);
                        }
                    }
                }
                next.sort_by(|a, b| {
                    let x = (a
                        .iter()
                        .map(|s| units[s.last].end - units[s.first].start)
                        .sum::<f64>()
                        - 75.)
                        .abs();
                    let y = (b
                        .iter()
                        .map(|s| units[s.last].end - units[s.first].start)
                        .sum::<f64>()
                        - 75.)
                        .abs();
                    x.total_cmp(&y)
                });
                next.truncate(64);
                beam = next;
            }
            for parts in beam
                .into_iter()
                .filter(|parts| {
                    plan_segments(
                        &Plan {
                            id: String::new(),
                            parts: parts.clone(),
                            event_ids: group.clone(),
                        },
                        units,
                        duration,
                    )
                    .is_ok()
                })
                .take(1)
            {
                push_plan(
                    Plan {
                        id: String::new(),
                        parts,
                        event_ids: group.clone(),
                    },
                    &mut plans,
                    &mut seen,
                );
            }
            if plans.len() >= 96 {
                break;
            }
        }
        if plans.len() >= 96 {
            break;
        }
    }
    plans.truncate(96);
    plans
}
fn push_plan(mut plan: Plan, plans: &mut Vec<Plan>, seen: &mut HashSet<String>) {
    let key = plan
        .parts
        .iter()
        .map(|s| format!("{}:{}", s.first, s.last))
        .collect::<Vec<_>>()
        .join(";");
    if seen.insert(key) {
        plan.id = format!("clip_{:03}", plans.len() + 1);
        plans.push(plan);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn units() -> Vec<Unit> {
        [
            (18., 45., "I promise to win the moon race."),
            (45., 55., "Let us start the challenge."),
            (170., 220., "You lost the moon race, that was unexpected."),
            (220., 240., "We can try again tomorrow."),
        ]
        .into_iter()
        .enumerate()
        .map(|(id, (start, end, text))| Unit {
            id,
            start,
            end,
            text: text.into(),
            approximate: true,
        })
        .collect()
    }
    fn event(first: usize, last: usize, quote: &str) -> Event {
        Event {
            first,
            last,
            summary: "Moon race promise and result".into(),
            topic: "moon race".into(),
            people: vec![],
            role: "setup_payoff".into(),
            interest: 9.,
            evidence: quote.into(),
        }
    }
    fn judgment(n: usize) -> Judgment {
        Judgment {
            accept: true,
            title: "The moon race backfires".into(),
            category: "Unexpected payoff".into(),
            reason: "The promise is followed by its actual unexpected result.".into(),
            relation: "setup_payoff".into(),
            connection_score: 9.,
            context_safe: true,
            evidence: if n == 2 {
                vec!["promise to win".into(), "You lost the moon race".into()]
            } else {
                vec!["promise to win".into()]
            },
            ratings: Ratings {
                entertainment: 8.,
                surprise: 8.,
                hook: 8.,
                clarity: 8.,
                tension: 8.,
                ending: 8.,
            },
        }
    }
    #[test]
    fn validates_smart_77_seconds_without_invented_times() {
        let u = units();
        let p = Plan {
            id: "test".into(),
            parts: vec![Span { first: 0, last: 0 }, Span { first: 2, last: 2 }],
            event_ids: vec![0, 1],
        };
        let c = validate_candidate(&p, judgment(2), &u, 240.).unwrap();
        assert_eq!(c.total_duration, 77.);
        assert_eq!(c.mode, "smart_story");
        assert!(c.score < 8.);
        let e = vec![
            event(0, 1, "promise to win"),
            event(2, 3, "You lost the moon race"),
        ];
        assert!(build_plans(&e, &u, 240.)
            .iter()
            .any(|p| p.parts.len() == 2 && plan_segments(p, &u, 240.).is_ok()));
    }
    #[test]
    fn rejects_overlap_bounds_order_and_wrong_totals() {
        let u = units();
        for parts in [
            vec![Span { first: 0, last: 1 }, Span { first: 1, last: 2 }],
            vec![Span { first: 2, last: 2 }, Span { first: 0, last: 0 }],
            vec![Span { first: 0, last: 99 }],
            vec![Span { first: 0, last: 0 }],
        ] {
            assert!(plan_segments(
                &Plan {
                    id: "bad".into(),
                    parts,
                    event_ids: vec![]
                },
                &u,
                240.
            )
            .is_err());
        }
    }
    #[test]
    fn scores_weights_and_rejects_unrelated_or_unquoted() {
        assert_eq!(judgment(2).ratings.weighted(), 8.);
        let u = units();
        let p = Plan {
            id: "x".into(),
            parts: vec![Span { first: 0, last: 0 }, Span { first: 2, last: 2 }],
            event_ids: vec![],
        };
        let mut j = judgment(2);
        j.relation = "unrelated".into();
        assert!(validate_candidate(&p, j, &u, 240.).is_err());
        let mut j = judgment(2);
        j.evidence[1] = "This quote is invented".into();
        assert!(validate_candidate(&p, j, &u, 240.).is_err());
        let c = validate_candidate(&p, judgment(2), &u, 240.).unwrap();
        assert_eq!(rank_deduplicate(vec![c.clone(), c]).len(), 1);
    }
    #[test]
    fn supports_three_to_five_segments() {
        let u = (0..5)
            .map(|id| Unit {
                id,
                start: id as f64 * 100.,
                end: id as f64 * 100. + 15.,
                text: "Actual source quote".into(),
                approximate: true,
            })
            .collect::<Vec<_>>();
        let p = Plan {
            id: "three".into(),
            parts: (0..5)
                .map(|id| Span {
                    first: id,
                    last: id,
                })
                .collect(),
            event_ids: vec![],
        };
        assert_eq!(
            plan_segments(&p, &u, 500.)
                .unwrap()
                .iter()
                .map(|s| s.duration)
                .sum::<f64>(),
            75.
        );
        let mut invalid = p.clone();
        invalid.parts.push(Span { first: 0, last: 0 });
        assert!(plan_segments(&invalid, &u, 500.).is_err());
    }
    #[test]
    fn windows_cover_long_transcripts_with_overlap() {
        let u = (0..1000)
            .map(|id| Unit {
                id,
                start: id as f64,
                end: id as f64 + 1.,
                text: "A complete source sentence about a challenge and reaction.".repeat(3),
                approximate: true,
            })
            .collect::<Vec<_>>();
        let w = windows(&u);
        let ids = w.iter().flatten().copied().collect::<HashSet<_>>();
        assert_eq!(ids.len(), u.len());
        assert!(w
            .windows(2)
            .any(|p| p[0].last().unwrap() >= p[1].first().unwrap()));
    }
}
