//! The ENG-03 steps: the static analyzers over the product crates. No
//! mutable global state, and the clippy bans that keep a blocking call
//! from waiting with no deadline and derived artifacts from reading the
//! wall clock or randomness. clippy itself runs in CI's lint job; these
//! steps hold each crate's configuration to the bans it must carry.

use cucumber::{given, then, when};
use engineering::product_crates::{self, deadline_bans, determinism_bans};
use engineering::{Checkout, outcome};

use crate::World;

/// What the analyzers found, by the rule each belongs to.
#[derive(Debug, Default)]
pub struct Analysis {
    code: Option<Checkout>,
    ran: Option<Findings>,
}

#[derive(Debug)]
struct Findings {
    globals: Vec<String>,
    deadlines: Vec<String>,
    determinism: Vec<String>,
}

impl Analysis {
    fn findings(&self) -> Result<&Findings, String> {
        self.ran
            .as_ref()
            .ok_or_else(|| "no step ran the static analyzers".to_owned())
    }
}

#[given(regex = r"^the repository's code$")]
fn the_repository_s_code(w: &mut World) {
    w.section::<Analysis>().code = Some(Checkout::new(&testkit::repo_root()));
}

#[when(regex = r"^the static analyzers run$")]
fn the_static_analyzers_run(w: &mut World) -> Result<(), String> {
    let a = w.section::<Analysis>();
    let code = a
        .code
        .as_ref()
        .ok_or("the scenario must open on the repository's code")?;
    a.ran = Some(Findings {
        globals: product_crates::no_mutable_globals(code),
        deadlines: product_crates::crates_keep_bans(code, &deadline_bans()),
        determinism: product_crates::crates_keep_bans(code, &determinism_bans()),
    });
    Ok(())
}

#[then(regex = r"^no crate declares mutable global state$")]
fn no_mutable_global_state(w: &mut World) -> Result<(), String> {
    outcome(w.section::<Analysis>().findings()?.globals.clone())
}

#[then(
    regex = r"^every public blocking operation takes a deadline, directly or through a cancellation signal a deadline fires$"
)]
fn every_blocking_operation_takes_a_deadline(w: &mut World) -> Result<(), String> {
    outcome(w.section::<Analysis>().findings()?.deadlines.clone())
}

#[then(regex = r"^no crate that computes derived artifacts reads the wall clock or randomness$")]
fn no_clock_or_randomness(w: &mut World) -> Result<(), String> {
    outcome(w.section::<Analysis>().findings()?.determinism.clone())
}
