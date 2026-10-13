//! The ENG-15 steps: the benchmark suite covers NFR-01..05, and the
//! benchmark CI job fails naming a benchmark that regressed. The
//! scenario stays pending until every NFR id ENG-15 names has a
//! benchmark of real code; until then its first step fails naming each
//! one and what it awaits, and the gate tests in the `engineering`
//! crate's `benchmark_gate` target hold the registry and the CI job.

use cucumber::{given, then, when};
use engineering::benchmarks::suite::{AWAITING, SUITE};
use engineering::{Checkout, benchmarks, outcome, review_workflow};

use crate::World;

/// What the ENG-15 steps carry from one to the next.
#[derive(Debug, Default)]
struct Benchmarks {
    /// The benchmark the change slowed, and the base's and the head's
    /// samples files.
    slowed: Option<(String, String, String)>,
}

fn state(w: &mut World) -> &mut Benchmarks {
    w.section::<Benchmarks>()
}

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[given(regex = r"^the benchmark suite covering (NFR-\d\d) to (NFR-\d\d)$")]
fn the_suite_covering(_w: &mut World, first: String, last: String) -> Result<(), String> {
    let named = benchmarks::named_ids(&checkout().requirement("ENG-15")?.text, "NFR");
    let stepped = benchmarks::named_ids(&format!("{first}..{}", &last[4..]), "NFR");
    if named != stepped {
        return Err(format!(
            "the step covers {stepped:?}, but ENG-15 names {named:?}"
        ));
    }

    outcome(
        [
            benchmarks::registered(&named, &SUITE, &AWAITING),
            benchmarks::uncovered(&named, &SUITE, &AWAITING),
        ]
        .concat(),
    )
}

#[when(regex = r"^a change slows a benchmark by more than 10% with statistical significance$")]
fn a_change_slows_a_benchmark(w: &mut World) -> Result<(), String> {
    let slowed = SUITE.first().ok_or("the suite has no benchmark")?.name;
    let (base, head) = benchmarks::slowed(slowed, 11);
    state(w).slowed = Some((slowed.to_owned(), base, head));
    Ok(())
}

#[then(regex = r"^the benchmark CI job fails naming the benchmark$")]
fn the_job_fails_naming_it(w: &mut World) -> Result<(), String> {
    let ci = checkout().read(".github/workflows/ci.yml")?;
    outcome(benchmarks::gated(&review_workflow::parse(&ci)))?;
    let (name, base, head) = state(w)
        .slowed
        .as_ref()
        .ok_or("no step slowed a benchmark")?;

    benchmarks::gate_fails_naming(base, head, name)
}
