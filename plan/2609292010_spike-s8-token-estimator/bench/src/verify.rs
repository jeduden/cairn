//! Checks an estimator against the committed sample: for each stratum
//! and model, the share of samples whose estimate is at or above
//! Claude's count (SRS §8.4 asks for at least 99%), and how far above
//! it the estimate runs.

use std::collections::BTreeMap;
use std::path::Path;

use crate::estimate::{self, BASE, SWITCH, WEIGHTS};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Sample {
    pub id: String,
    pub stratum: String,
    pub text: String,
    /// Claude's count per model.
    pub tokens: BTreeMap<String, u64>,
}

pub fn read(path: &Path) -> Result<Vec<Sample>> {
    let mut out = Vec::new();
    for line in std::fs::read_to_string(path)?.lines() {
        let v: serde_json::Value = serde_json::from_str(line)?;
        let s = |k: &str| v[k].as_str().map(str::to_owned).ok_or(format!("sample lacks {k}"));
        let mut tokens = BTreeMap::new();
        if let Some(t) = v["tokens"].as_object() {
            for (m, n) in t {
                tokens.insert(m.clone(), n.as_u64().ok_or("token count is not a count")?);
            }
        }
        out.push(Sample { id: s("id")?, stratum: s("stratum")?, text: s("text")?, tokens });
    }
    Ok(out)
}

/// One estimator to compare.
pub struct Rule {
    pub name: &'static str,
    pub f: fn(&str) -> u64,
}

pub const RULES: &[Rule] = &[
    Rule { name: "S8 estimator", f: estimate::estimate },
    Rule { name: "bytes / 4", f: |t| (t.len() as u64).div_ceil(4) },
    Rule { name: "bytes / 3", f: |t| (t.len() as u64).div_ceil(3) },
    Rule { name: "bytes / 2", f: |t| (t.len() as u64).div_ceil(2) },
    Rule { name: "bytes", f: |t| t.len() as u64 },
];

#[derive(Default)]
struct Tally {
    n: u64,
    covered: u64,
    est: u64,
    truth: u64,
    worst: f64,
    worst_id: String,
}

impl Tally {
    fn add(&mut self, id: &str, est: u64, truth: u64) {
        self.n += 1;
        self.est += est;
        self.truth += truth;
        if est >= truth {
            self.covered += 1;
        }
        let r = est as f64 / truth.max(1) as f64;
        if self.n == 1 || r < self.worst {
            self.worst = r;
            self.worst_id = id.to_owned();
        }
    }

    fn row(&self, label: &str) -> String {
        format!(
            "| {label} | {} | {:.1}% | {:.2} | {:.2} ({}) |",
            self.n,
            100.0 * self.covered as f64 / self.n.max(1) as f64,
            self.est as f64 / self.truth.max(1) as f64,
            self.worst,
            self.worst_id
        )
    }
}

/// The model every sample was counted on. The others share its
/// tokenizer; the report explains the two counts that differ.
pub const REFERENCE: &str = "claude-haiku-5-5";

/// Prints one table per rule and model; returns whether the S8
/// estimator covers at least 99% of the samples on the reference model.
pub fn verify(path: &Path) -> Result<bool> {
    let samples = read(path)?;
    let models: Vec<String> = {
        let mut m: Vec<String> = samples.iter().flat_map(|s| s.tokens.keys().cloned()).collect();
        m.sort();
        m.dedup();
        m
    };
    println!("weights {WEIGHTS:?} / 1024, switch {SWITCH} / 1024, base {BASE}");
    let mut ok = true;
    for rule in RULES {
        for model in &models {
            let mut all = Tally::default();
            let mut by: BTreeMap<&str, Tally> = BTreeMap::new();
            for s in &samples {
                let Some(&truth) = s.tokens.get(model) else { continue };
                let est = (rule.f)(&s.text);
                all.add(&s.id, est, truth);
                by.entry(&s.stratum).or_default().add(&s.id, est, truth);
            }
            println!("\n{} on {model}\n", rule.name);
            println!("| Stratum | Samples | Estimate ≥ count | Estimate / count | Lowest ratio |");
            println!("| --- | --- | --- | --- | --- |");
            for (k, t) in &by {
                println!("{}", t.row(k));
            }
            println!("{}", all.row("all"));
            if rule.name == "S8 estimator" && model == REFERENCE && all.covered * 100 < all.n * 99 {
                ok = false;
            }
        }
    }
    Ok(ok)
}
