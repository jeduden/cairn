//! Redaction before anything is written (SEC-08). A [`Redactor`] holds
//! the maintained redaction rule set, the harness account identifiers
//! learnt from a transcript, and the principal's redaction patterns
//! (`redaction.extra_patterns`). Every byte range one of them matches
//! is replaced by `[REDACTED:<rule>]`, which names the rule and carries
//! nothing derived from the secret. Ranges that overlap merge into one
//! marker, so no part of a secret two rules both match survives.

use std::fmt;

use serde_json::{Map, Value};

pub mod account;
pub mod rule;

use account::HarnessAccountIdentifiers;
use rule::{RedactionRule, rule_set};

/// The rule name of the principal's `n`th redaction pattern, from 1.
#[must_use]
pub fn extra_pattern(n: usize) -> String {
    format!("extra_pattern_{n}")
}

/// A principal's redaction pattern that does not compile, or compiles
/// past the size limit: refused, while every other rule still applies.
#[derive(Debug, Clone)]
pub struct PatternRefused {
    /// The rule name it would have had.
    pub rule: String,
    pub source: regex::Error,
}

impl fmt::Display for PatternRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "redaction pattern {} refused", self.rule)
    }
}

impl std::error::Error for PatternRefused {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Text with its secrets replaced, and the rule of each replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redacted {
    pub text: String,
    /// One rule name per marker, in the order they appear.
    pub rules: Vec<String>,
}

/// The rules redaction applies.
#[derive(Debug, Clone)]
pub struct Redactor {
    rules: Vec<RedactionRule>,
}

impl Redactor {
    /// The rule set and the principal's redaction patterns, with each
    /// pattern that is refused.
    ///
    /// # Errors
    ///
    /// Fails when the rule set does not compile, which its unit test
    /// rules out.
    pub fn new(extra_patterns: &[String]) -> Result<(Self, Vec<PatternRefused>), regex::Error> {
        let mut rules = rule_set()?;
        let mut refused = Vec::new();
        for (n, pattern) in (1..).zip(extra_patterns) {
            let rule = extra_pattern(n);
            match RedactionRule::new(&rule, pattern) {
                Ok(r) => rules.push(r),
                Err(source) => refused.push(PatternRefused { rule, source }),
            }
        }

        Ok((Self { rules }, refused))
    }

    /// This redactor with the harness account identifiers' rules ahead
    /// of the principal's redaction patterns.
    ///
    /// # Errors
    ///
    /// Fails when an identifier is too long to compile.
    pub fn with_identifiers(&self, ids: &HarnessAccountIdentifiers) -> Result<Self, regex::Error> {
        let set = rule::RULE_SET.len().min(self.rules.len());
        let mut rules = self.rules[..set].to_vec();
        rules.extend(ids.rules()?);
        rules.extend_from_slice(&self.rules[set..]);

        Ok(Self { rules })
    }

    /// `text` with every match replaced.
    #[must_use]
    pub fn redact_text(&self, text: &str) -> Redacted {
        let mut spans: Vec<(usize, usize, usize)> = self
            .rules
            .iter()
            .enumerate()
            .flat_map(|(i, r)| r.spans(text).map(move |(s, e)| (s, e, i)))
            .collect();
        spans.sort_unstable();
        let mut out = Redacted {
            text: String::with_capacity(text.len()),
            rules: Vec::new(),
        };
        let mut at = 0;
        for (start, end, rule) in merged(spans) {
            out.text.push_str(&text[at..start]);
            let name = self.rules[rule].name();
            out.text.push_str(&format!("[REDACTED:{name}]"));
            out.rules.push(name.to_owned());
            at = end;
        }
        out.text.push_str(&text[at..]);

        out
    }

    /// `value` with every string in it, object keys included, redacted.
    #[must_use]
    pub fn redact_value(&self, value: &Value) -> (Value, Vec<String>) {
        let mut rules = Vec::new();
        let redacted = self.walk(value, &mut rules);

        (redacted, rules)
    }

    /// A transcript line or hook input redacted: its JSON value's
    /// strings, written back as JSON when any is redacted, or the raw
    /// text when it is no JSON. A line with nothing to redact stays as
    /// it was, byte for byte.
    #[must_use]
    pub fn redact_line(&self, line: &str) -> Redacted {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            return self.redact_text(line);
        };
        let (redacted, rules) = self.redact_value(&value);
        let text = if rules.is_empty() {
            line.to_owned()
        } else {
            redacted.to_string()
        };

        Redacted { text, rules }
    }

    fn walk(&self, value: &Value, rules: &mut Vec<String>) -> Value {
        match value {
            Value::String(s) => Value::String(self.collect(s, rules)),
            Value::Array(items) => Value::Array(items.iter().map(|v| self.walk(v, rules)).collect()),
            Value::Object(map) => {
                let mut out = Map::new();
                for (k, v) in map {
                    let key = unique(self.collect(k, rules), &out);
                    let v = self.walk(v, rules);
                    out.insert(key, v);
                }
                Value::Object(out)
            }
            other => other.clone(),
        }
    }

    fn collect(&self, text: &str, rules: &mut Vec<String>) -> String {
        let r = self.redact_text(text);
        rules.extend(r.rules);

        r.text
    }
}

/// `key`, or, where a redacted key already stands in `map`, the first
/// of `key (2)`, `key (3)` and on that does not, so no value is lost
/// when two keys redact alike.
fn unique(key: String, map: &Map<String, Value>) -> String {
    if !map.contains_key(&key) {
        return key;
    }

    (2..)
        .map(|n| format!("{key} ({n})"))
        .find(|k| !map.contains_key(k))
        .unwrap_or(key)
}

/// Sorted spans with each run of overlapping ones merged into the
/// first, which keeps its rule.
fn merged(spans: Vec<(usize, usize, usize)>) -> Vec<(usize, usize, usize)> {
    let mut out: Vec<(usize, usize, usize)> = Vec::with_capacity(spans.len());
    for (start, end, rule) in spans {
        match out.last_mut() {
            Some(last) if start < last.1 => last.1 = last.1.max(end),
            _ => out.push((start, end, rule)),
        }
    }

    out
}

#[cfg(test)]
mod tests;
