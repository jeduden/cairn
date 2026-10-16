//! One settings layer's text, parsed as TOML and validated strictly
//! (ADM-04): every key must be a settings key, every value of its kind
//! and in its range. What fails is named by key and problem, never by
//! quoting the layer's text, which a repository configuration's author
//! controls (T5).

use std::collections::BTreeMap;
use std::fmt;

use crate::key::{Kind, Lookup, SettingsKey, lookup, shown};
use crate::value::Value;

/// The settings one layer sets, by key, each validated.
pub type LayerValues = BTreeMap<SettingsKey, Value>;

/// What is wrong with one key of a layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// No settings key has its name.
    UnknownKey,
    /// Its value is not of the kind the key takes; says what it takes.
    TypeError(&'static str),
    /// Its value is of the right kind but outside the key's range; says
    /// what the range is.
    OutOfRange(String),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKey => f.write_str("unknown key"),
            Self::TypeError(want) => write!(f, "type error: {want}"),
            Self::OutOfRange(want) => write!(f, "out of range: {want}"),
        }
    }
}

/// One invalid key: its name as an error may show it, and its problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invalid {
    pub key: String,
    pub problem: Problem,
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "key {}: {}", self.key, self.problem)
    }
}

/// Why a layer's text gives no settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerError {
    /// The text is not TOML, from the given line and column on.
    Syntax { line: usize, column: usize },
    /// The text is TOML, but these keys are invalid, in key order.
    Invalid(Vec<Invalid>),
}

impl fmt::Display for LayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { line, column } => {
                write!(f, "not TOML at line {line}, column {column}")
            }
            Self::Invalid(invalid) => {
                let each: Vec<String> = invalid.iter().map(ToString::to_string).collect();
                f.write_str(&each.join("; "))
            }
        }
    }
}

/// The settings `text` sets.
///
/// # Errors
///
/// Fails when `text` is not TOML, or when any key in it is unknown or
/// holds a value of the wrong kind or out of range, naming each.
pub fn parse(text: &str) -> Result<LayerValues, LayerError> {
    let table: toml::Table = text.parse().map_err(|e: toml::de::Error| {
        let at = e.span().map_or(0, |s| s.start);
        let (line, column) = position(text, at);
        LayerError::Syntax { line, column }
    })?;
    let (mut values, mut invalid) = (LayerValues::new(), Vec::new());
    walk(&mut Vec::new(), &table, &mut values, &mut invalid);
    if invalid.is_empty() {
        Ok(values)
    } else {
        Err(LayerError::Invalid(invalid))
    }
}

/// The line and column, from 1, of byte `at` of `text`.
fn position(text: &str, at: usize) -> (usize, usize) {
    let before = text.get(..at).unwrap_or(text);
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;

    (line, column)
}

/// Validates every key under `path` in `table`.
fn walk(
    path: &mut Vec<String>,
    table: &toml::Table,
    values: &mut LayerValues,
    invalid: &mut Vec<Invalid>,
) {
    for (name, value) in table {
        path.push(name.clone());
        let segments: Vec<&str> = path.iter().map(String::as_str).collect();
        match (lookup(&segments), value) {
            (Lookup::Key(key), value) => match validate(key.spec().kind, value) {
                Ok(v) => {
                    values.insert(key, v);
                }
                Err(problem) => invalid.push(Invalid {
                    key: shown(key.name()),
                    problem,
                }),
            },
            // An unknown table's keys are named in full, each unknown.
            (Lookup::Table | Lookup::Unknown, toml::Value::Table(inner)) if !inner.is_empty() => {
                walk(path, inner, values, invalid);
            }
            (Lookup::Table, toml::Value::Table(_)) => {}
            (Lookup::Table | Lookup::Unknown, _) => invalid.push(Invalid {
                key: shown(&segments.join(".")),
                problem: Problem::UnknownKey,
            }),
        }
        path.pop();
    }
}

/// `value` as a value of `kind`.
fn validate(kind: Kind, value: &toml::Value) -> Result<Value, Problem> {
    let wrong = Problem::TypeError(kind.wants());
    match (kind, value) {
        (Kind::Bool, toml::Value::Boolean(b)) => Ok(Value::Bool(*b)),
        (Kind::Integer { min, max }, toml::Value::Integer(i)) => {
            if (min..=max).contains(i) {
                Ok(Value::Integer(*i))
            } else {
                Err(Problem::OutOfRange(format!("want {min} to {max}")))
            }
        }
        (Kind::Choice(choices), toml::Value::String(s)) => {
            if choices.contains(&s.as_str()) {
                Ok(Value::Text(s.clone()))
            } else {
                Err(Problem::OutOfRange(format!(
                    "want one of {}",
                    choices.join(", ")
                )))
            }
        }
        (Kind::Text { max }, toml::Value::String(s)) => text(s, max).map(Value::Text),
        (Kind::Texts { items, max }, toml::Value::Array(array)) => {
            let texts = array
                .iter()
                .map(|item| item.as_str().ok_or_else(|| wrong.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            if texts.len() > items {
                return Err(Problem::OutOfRange(format!("want at most {items} entries")));
            }
            texts
                .into_iter()
                .map(|t| text(t, max))
                .collect::<Result<_, _>>()
                .map(Value::Texts)
        }
        _ => Err(wrong),
    }
}

/// `s` as a text of 1 to `max` characters.
fn text(s: &str, max: usize) -> Result<String, Problem> {
    let n = s.chars().count();
    if n == 0 || n > max {
        return Err(Problem::OutOfRange(format!("want 1 to {max} characters")));
    }

    Ok(s.to_owned())
}

#[cfg(test)]
mod tests;
