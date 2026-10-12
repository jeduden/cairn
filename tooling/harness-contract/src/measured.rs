//! What the spikes measured rather than recorded line by line: the
//! harness timeouts of hook handlers (`timeouts.json`, ASM-10) and the
//! harness's transcript cleanup (`cleanup.json`, ASM-05).

use serde_json::Value;

use crate::text;

/// How far a measured handler may fall from the limit it is compared to,
/// as a share of that limit.
pub const TOLERANCE: f64 = 0.15;

/// One handler that slept past its harness timeout.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub mode: String,
    pub trial: String,
    pub event: String,
    pub timeout_s: Option<f64>,
    pub ran_ms: u64,
    pub finished: bool,
}

/// Reads the measurements of a `timeouts.json` document.
///
/// # Errors
///
/// Fails when the document holds no measurement or one lacks its run
/// time.
pub fn measurements(doc: &Value) -> Result<Vec<Measurement>, String> {
    let all = doc
        .get("measurements")
        .and_then(Value::as_array)
        .filter(|a| !a.is_empty())
        .ok_or("timeouts: no measurement")?;
    all.iter().map(measurement).collect()
}

fn measurement(m: &Value) -> Result<Measurement, String> {
    Ok(Measurement {
        mode: text(m, "mode").to_owned(),
        trial: text(m, "trial").to_owned(),
        event: text(m, "event").to_owned(),
        timeout_s: m.get("configured_timeout_s").and_then(Value::as_f64),
        ran_ms: m
            .get("ran_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("timeouts: a measurement has no ran_ms: {m}"))?,
        finished: m
            .get("finished")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("timeouts: a measurement has no finished: {m}"))?,
    })
}

/// The measurements of `event` in `mode` with `timeout_s` configured.
#[must_use]
pub fn matching<'a>(
    all: &'a [Measurement],
    event: &str,
    mode: &str,
    timeout_s: Option<f64>,
) -> Vec<&'a Measurement> {
    all.iter()
        .filter(|m| m.event == event && m.mode == mode && m.timeout_s == timeout_s)
        .collect()
}

/// Whether `ran_ms` lies within [`TOLERANCE`] of `limit_s`.
#[must_use]
pub fn about(ran_ms: u64, limit_s: f64) -> bool {
    #[allow(clippy::cast_precision_loss)]
    let ran_s = ran_ms as f64 / 1000.0;
    (ran_s - limit_s).abs() <= limit_s * TOLERANCE
}

/// Checks the print-mode trials that configured both `Stop` and
/// `SessionEnd` and whose `Stop` handler reached its harness timeout:
/// none may have started its `SessionEnd` handler. Returns how many
/// such trials there were and the problems.
///
/// # Errors
///
/// Fails when the document's trials or measurements cannot be read.
pub fn session_end_after_stop_timeout(doc: &Value) -> Result<(usize, Vec<String>), String> {
    let all = measurements(doc)?;
    let trials = doc
        .get("trials")
        .and_then(Value::as_object)
        .ok_or("timeouts: no trials")?;
    let mut checked = 0;
    let mut problems = Vec::new();
    for (name, trial) in trials {
        let configured = |hook: &str| {
            trial
                .get("configured")
                .and_then(Value::as_array)
                .is_some_and(|c| c.iter().any(|h| h.as_str() == Some(hook)))
        };
        let stop_timed_out = in_trial(&all, name, "Stop").any(|m| !m.finished);
        if text(trial, "mode") != "print" || !configured("SessionEnd") || !stop_timed_out {
            continue;
        }
        checked += 1;
        if in_trial(&all, name, "SessionEnd").next().is_some() {
            problems.push(format!(
                "trial {name}: SessionEnd started after Stop reached its timeout"
            ));
        }
    }
    Ok((checked, problems))
}

fn in_trial<'a>(
    all: &'a [Measurement],
    trial: &'a str,
    hook: &'a str,
) -> impl Iterator<Item = &'a Measurement> {
    all.iter()
        .filter(move |m| m.trial == trial && m.event == hook)
}

/// A transcript a cleanup observation names, with its age.
#[derive(Debug, Clone, PartialEq)]
pub struct Aged {
    pub path: String,
    pub age_days: f64,
}

/// Splits the transcripts of a `cleanup.json` document into those the
/// harness deleted and those it kept.
///
/// # Errors
///
/// Fails when the document names no transcript before cleanup, no list
/// after it, or a transcript without its age.
pub fn cleanup(doc: &Value) -> Result<(Vec<Aged>, Vec<Aged>), String> {
    let before = doc
        .get("before")
        .and_then(Value::as_array)
        .filter(|b| !b.is_empty())
        .ok_or("cleanup: no transcript before")?;
    let after: Vec<&str> = doc
        .get("after")
        .and_then(Value::as_array)
        .ok_or("cleanup: no list after")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let aged = before
        .iter()
        .map(|b| {
            let age_days = b
                .get("age_days")
                .and_then(Value::as_f64)
                .ok_or_else(|| format!("cleanup: a transcript has no age_days: {b}"))?;
            Ok(Aged {
                path: text(b, "path").to_owned(),
                age_days,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(aged
        .into_iter()
        .partition(|a| !after.contains(&a.path.as_str())))
}

#[cfg(test)]
mod tests;
