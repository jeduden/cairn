//! The ENG-14 steps: the isolation guard aborts a test that resolves
//! real state, and no agent instruction file directs a destructive
//! command at non-isolated state.

use std::env;

use cucumber::{given, then, when};
use engineering::{Checkout, agent_instructions, isolation_guard, outcome};
use testkit::guard::Place;

use crate::World;

/// What the ENG-14 scenario tracks: the places the probed test
/// resolves, and what running it found wrong.
#[derive(Debug, Default)]
struct Isolation {
    places: Vec<&'static str>,
    problems: Vec<String>,
}

fn isolation(w: &mut World) -> &mut Isolation {
    w.section::<Isolation>()
}

#[given(regex = r"^a test that resolves a path under the real `HOME`$")]
fn a_test_resolves_the_real_user_home(w: &mut World) {
    isolation(w).places = Place::ALL.iter().map(|p| p.name()).collect();
}

#[when(regex = r"^the suite runs$")]
fn the_suite_runs(w: &mut World) -> Result<(), String> {
    let exe = env::current_exe().map_err(|e| format!("find the test executable: {e}"))?;
    let scratch = w.home.join("probe");
    let state = isolation(w);
    state.problems = isolation_guard::guard_holds(&exe, &scratch, &state.places);

    Ok(())
}

#[then(regex = r"^the isolation guard aborts that test$")]
fn the_guard_aborts_it(w: &mut World) -> Result<(), String> {
    let mut problems = isolation(w).problems.clone();
    problems.extend(isolation_guard::no_unguarded_reads(&Checkout::new(
        &testkit::repo_root(),
    )));

    outcome(problems)
}

#[then(
    regex = r"^no agent instruction file in the repository directs a destructive command at non-isolated state$"
)]
fn no_destructive_instruction(_w: &mut World) -> Result<(), String> {
    outcome(agent_instructions::no_destructive_instructions(
        &Checkout::new(&testkit::repo_root()),
    ))
}
