//! The benchmark suite: each benchmark registered against the NFR id it
//! measures toward, and each NFR id ENG-15 names that no benchmark of
//! real code covers yet, with what it waits on.
//!
//! A benchmark measures code that exists. The hook handlers, the MCP
//! tools, ingestion and the store that NFR-01..05 set targets for
//! arrive with M1's other plans, so they are declared [`AWAITING`]
//! here rather than stood in for by a benchmark of something else. The
//! one benchmark today, [`SUITE`]'s core start-up, is a
//! [`Coverage::Floor`]: a cost every hook handler pays, not a hook
//! handler. The ENG-15 scenario stays pending until every NFR id it
//! names has a [`Coverage::Target`] benchmark.
//!
//! The CI job runs the base commit's suite and the head's. A benchmark
//! renamed or removed on the head goes into [`RETIRED`] in the same
//! change, so the gate reports it rather than failing on it as missing.

use crate::samples::valid_name;

/// How much of its requirement's target a benchmark measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// What the requirement's target names: the requirement is covered.
    Target,
    /// A cost the target's subject pays, a lower bound on it; the
    /// requirement is not covered.
    Floor,
}

/// One benchmark of the suite. Every benchmark today starts the core
/// executable with `args` and times it until it exits; a benchmark that
/// calls into a crate in-process joins with the crate it measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Benchmark {
    /// Its name in the samples file and the gate's report.
    pub name: &'static str,
    /// The NFR id it measures toward.
    pub requirement: &'static str,
    pub coverage: Coverage,
    /// What it measures, in words a reader of the report checks it by.
    pub measures: &'static str,
    /// The core executable's arguments.
    pub args: &'static [&'static str],
}

/// An NFR id ENG-15 names that no benchmark of real code covers yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Awaiting {
    pub requirement: &'static str,
    /// What the benchmark waits on.
    pub on: &'static str,
}

/// The suite.
pub const SUITE: [Benchmark; 1] = [Benchmark {
    name: "core_start_up",
    requirement: "NFR-01",
    coverage: Coverage::Floor,
    measures: "the core executable started with `version` until it exits: \
               the process start every hook handler pays before its own work, \
               not a hook handler",
    args: &["version"],
}];

/// The NFR ids ENG-15 names that have no benchmark of real code yet.
pub const AWAITING: [Awaiting; 5] = [
    Awaiting {
        requirement: "NFR-01",
        on: "the hook handlers, timed on a store of 1M events (plan 2609292012)",
    },
    Awaiting {
        requirement: "NFR-02",
        on: "the hook handlers and their hook budgets (plan 2609292012)",
    },
    Awaiting {
        requirement: "NFR-03",
        on: "the MCP server's event_search and event_expand on a store of 10M events (plan 2609292012)",
    },
    Awaiting {
        requirement: "NFR-04",
        on: "ingestion (plan 2609292012)",
    },
    Awaiting {
        requirement: "NFR-05",
        on: "the store at 10M events and 50 concurrent runs writing (plan 2609292012)",
    },
];

/// The benchmarks the suite no longer runs, by the name the base's
/// suite gave them.
pub const RETIRED: [&str; 0] = [];

/// The problems with `suite`: a name the samples file cannot carry, or
/// one used twice, or one also retired.
#[must_use]
pub fn problems(suite: &[Benchmark], retired: &[&str]) -> Vec<String> {
    let mut problems = Vec::new();
    for (i, b) in suite.iter().enumerate() {
        if !valid_name(b.name) {
            problems.push(format!(
                "benchmark {:?} is not lowercase letters, digits and underscores",
                b.name
            ));
        }
        if suite[..i].iter().any(|earlier| earlier.name == b.name) {
            problems.push(format!("benchmark {} is registered twice", b.name));
        }
        if retired.contains(&b.name) {
            problems.push(format!("benchmark {} is registered and retired", b.name));
        }
    }

    problems
}

#[cfg(test)]
mod tests;
