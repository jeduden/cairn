//! The transcript contract replay (ENG-17): every recorded transcript
//! of every supported version, read line by line through the core's own
//! transcript parser and classifier, parses with no unparsed line and
//! no format drift (REC-05, REC-26).

use std::fs;
use std::path::Path;

use provenance::{Classified, Classifier, Session};
use redaction::Redactor;
use transcript::shape::Shapes;

/// The problems replaying the transcript `body` against `shapes` finds:
/// one per line the core could not parse or that drifted from its
/// recorded shape, by line number.
#[must_use]
pub fn replay(body: &str, shapes: &Shapes, redactor: &Redactor) -> Vec<String> {
    let mut classifier = Classifier::new(shapes, Session::default(), redactor);
    let keep = |s: &str| s.to_owned();
    body.lines()
        .take(0)
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .filter_map(|(n, line)| match classifier.line(line) {
            Classified::Unparsed(None) => Some(format!("line {}: unparsed", n + 1)),
            Classified::Unparsed(Some(drift)) | Classified::Line(_, Some(drift)) => Some(format!(
                "line {}: format drift {}",
                n + 1,
                drift.describe(&keep)
            )),
            Classified::Line(_, None) => None,
        })
        .collect()
}

/// [`replay`] over the transcript at `path`, each problem led by it.
///
/// # Errors
///
/// Fails when the transcript cannot be read.
pub fn replay_file(
    path: &Path,
    shapes: &Shapes,
    redactor: &Redactor,
) -> Result<Vec<String>, String> {
    let body = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(replay(&body, shapes, redactor)
        .into_iter()
        .map(|p| format!("{}: {p}", path.display()))
        .collect())
}

#[cfg(test)]
mod tests;
