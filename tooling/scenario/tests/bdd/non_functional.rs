//! The steps of NFR-02 that phase 9 of plan 2609292012 carries: a hook
//! handler of the built core answers within its hook budget, leaving a
//! start marker it clears; one the harness kills leaves its marker
//! behind, counted once, past its hook's harness timeout, by the next
//! hook handler, the MCP server as it starts, or `cairn status`.
//!
//! Each hook's budget and harness timeout are §9.1's and those Cairn
//! installs it with (ASM-10). The development build's seams stand the
//! clock still (`CAIRN_TEST_CLOCK`) and hang a handler until it is
//! killed (`CAIRN_TEST_FAULT=hook-hang`). The row that times a handler
//! against its budget runs the release build, which has no seams, so
//! it needs `cargo build --release -p cairn` first.

use std::cell::RefCell;
use std::fs;
use std::time::{Duration, Instant};

use cucumber::{then, when};
use serde_json::{Value, json};
use testkit::relative;

use crate::World;
use crate::record::{HOOK_BUDGET, home, run_core_killed, run_core_with, run_release_core};
use crate::record_hooks::{SESSION, answered_empty, counted, prompt_submit, session_start};

/// When the killed handlers' clock stands.
const STARTED: u64 = 1_900_000_000;

/// How long after its start a killed handler is killed: well inside
/// its harness timeout, and long past any start.
const KILLED_AFTER: Duration = Duration::from_millis(1500);

/// How much above its hook budget the step's harness timeout lies.
const JUST_ABOVE: Duration = Duration::from_secs(1);

/// How many interleaved pairs of runs the budget row times. Odd, so the
/// last run of the row is the hook handler's, which the steps after it
/// read.
const ROUNDS: usize = 11;

/// The time, in milliseconds, the release core takes to start and exit
/// with `version`: the floor every hook handler pays (the benchmark
/// suite's `core_start_up`, whose median is about 1.8 ms on the
/// development machine that measured it, rounded up). A constant for the
/// stakeholder to confirm or move; docs/testing.md shows the derivation.
const REFERENCE_START_UP_MS: u128 = 2;

/// How many times the release core's start-up a hook handler may take at
/// the median: its hook budget in units of [`REFERENCE_START_UP_MS`],
/// which is 25 for the 50 ms hooks and 75 for `SessionStart`'s 150 ms.
/// The run is confined (ENG-12) and the confinement adds tens of
/// milliseconds to every run, a hook handler's and the reference's alike,
/// so the row compares the two medians, not a hook with the wall clock.
fn limit_of(hook: &str) -> u32 {
    let (budget, _) = budget_of(hook);
    u32::try_from(budget.as_millis() / REFERENCE_START_UP_MS).unwrap_or(u32::MAX)
}

/// What the NFR-02 steps track: how long the hook runs took, against the
/// reference's, and the last hook's.
#[derive(Debug, Default)]
struct BudgetSteps {
    /// The slowest hook run and the harness timeout it must stay below.
    took: Option<(Duration, Duration)>,
    /// The median check of the hook's runs against the reference's.
    relative: Option<Result<(), String>>,
    hook: String,
}

/// A hook's budget (§9.1) and the harness timeout Cairn installs it
/// with (ASM-10).
fn budget_of(hook: &str) -> (Duration, Duration) {
    match hook {
        "SessionStart" => (Duration::from_millis(150), Duration::from_secs(5)),
        "UserPromptSubmit" => (Duration::from_millis(50), Duration::from_secs(5)),
        _ => (Duration::from_millis(50), Duration::from_secs(30)),
    }
}

/// A representative hook input of `hook`.
fn input_of(w: &World, hook: &str) -> Result<Value, String> {
    match hook {
        "SessionStart" => session_start(w, SESSION, "startup"),
        "UserPromptSubmit" => prompt_submit(w, SESSION, "p-budget", "Say ok."),
        _ => Ok(json!({"session_id": SESSION, "hook_event_name": hook, "tool_name": "Bash"})),
    }
}

/// The start markers the home holds, staged ones left out.
fn markers(w: &World) -> Result<Vec<String>, String> {
    let dir = home(w)?.root().join("hooks");
    let Ok(listing) = fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    Ok(listing
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.'))
        .collect())
}

#[when(
    regex = r#"^the hook "([A-Za-z]+)" runs within its hook budget, with a harness timeout just above it$"#
)]
fn runs_within_budget(w: &mut World, hook: String) -> Result<(), String> {
    let (budget, _) = budget_of(&hook);
    let input = input_of(w, &hook)?.to_string();
    let world = RefCell::new(w);
    // The release build, which keeps each hook's own budget: the
    // development build is an order of magnitude slower and flakes on a
    // loaded machine, and its budget seam is not the one users get.
    // Under the confinement a hook handler's 5-10 ms gains 25-30 ms, so
    // the wall clock cannot hold it to 50 ms; the reference, the release
    // core started with `version`, gains the same, and the ratio of the
    // medians of alternating pairs does not.
    let mut reference = || {
        let started = Instant::now();
        run_release_core(&mut world.borrow_mut(), &["version"], None)?;
        Ok(started.elapsed())
    };
    let mut handler = || {
        let mut w = world.borrow_mut();
        let started = Instant::now();
        run_release_core(&mut w, &["hook", &hook], Some(input.as_bytes()))?;
        let took = started.elapsed();
        answered_empty(&mut w).map(|()| took)
    };
    let (hooks, references) = relative::interleaved(ROUNDS, &mut reference, &mut handler)?;
    let relative = relative::within_multiple(&hooks, &references, limit_of(&hook));
    let slowest = hooks.iter().max().copied().unwrap_or_default();
    let w = world.into_inner();
    let state = w.section::<BudgetSteps>();
    state.took = Some((slowest, budget + JUST_ABOVE));
    state.relative = Some(relative);
    state.hook = hook;
    Ok(())
}

#[then(regex = r"^the hook handler exits 0 within its hook budget, below the harness timeout$")]
fn exits_within_budget(w: &mut World) -> Result<(), String> {
    answered_empty(w)?;
    // A handler stopped at its budget also exits 0 with an empty answer.
    // The confined run cannot count an overrun as a failure: the core
    // stops a handler at its own wall-clock budget, which the
    // confinement's overhead alone can reach, so the budget is held
    // here relative to the reference, and in absolute terms by the
    // benchmark job, which times the unconfined release core and fails
    // on a counted `hook_budget_exceeded` (ENG-15).
    let state = w.section::<BudgetSteps>();
    state
        .relative
        .take()
        .ok_or("no step ran a hook within its budget")?
        .map_err(|e| format!("the {} hook handler is too slow for its budget: {e}", state.hook))?;
    let (took, timeout) = w
        .section::<BudgetSteps>()
        .took
        .ok_or("no step ran a hook within its budget")?;
    if took < timeout {
        return Ok(());
    }
    Err(format!(
        "the hook took {took:?}, past its harness timeout {timeout:?}"
    ))
}

#[then(regex = r"^it left a start marker and cleared it before it exited$")]
fn marker_cleared(w: &mut World) -> Result<(), String> {
    let dir = home(w)?.root().join("hooks");
    let left = markers(w)?;
    if dir.is_dir() && left.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the hooks directory exists: {}; markers left: {left:?}",
        dir.is_dir()
    ))
}

/// Kills a hanging `hook` handler whose clock stands at `at`.
fn kill_hook(w: &mut World, hook: &str, at: u64) -> Result<(), String> {
    let at = at.to_string();
    let running = run_core_killed(
        w,
        &["hook", hook],
        &[
            ("CAIRN_TEST_FAULT", "hook-hang"),
            ("CAIRN_TEST_CLOCK", &at),
            (HOOK_BUDGET, "60000"),
        ],
        KILLED_AFTER,
    )?;
    if running {
        return Ok(());
    }
    Err(format!(
        "the {hook} hook handler finished before the harness killed it"
    ))
}

#[when(regex = r#"^the hook "([A-Za-z]+)" runs and the harness kills it before it finishes$"#)]
fn killed(w: &mut World, hook: String) -> Result<(), String> {
    kill_hook(w, &hook, STARTED)?;
    w.section::<BudgetSteps>().hook = hook;
    Ok(())
}

#[then(regex = r"^its start marker is left behind$")]
fn marker_left(w: &mut World) -> Result<(), String> {
    let left = markers(w)?;
    let hook = w.section::<BudgetSteps>().hook.clone();
    if left.len() == 1 && left[0].starts_with(&format!("{hook}.")) {
        return Ok(());
    }
    Err(format!("the markers left are {left:?}"))
}

/// Runs the reader `next` names with the clock at `at`.
fn read_markers(w: &mut World, next: &str, at: u64) -> Result<(), String> {
    let at = at.to_string();
    let clock = [("CAIRN_TEST_CLOCK", at.as_str())];
    match next {
        "the next hook handler" => {
            let input = input_of(w, "SessionStart")?.to_string();
            run_core_with(w, &["hook", "SessionStart"], Some(input.as_bytes()), &clock)
        }
        "the MCP server of the next harness session as it starts" => {
            run_core_with(w, &["mcp"], None, &clock)
        }
        _ => run_core_with(w, &["status"], None, &clock),
    }
}

/// When a reader finds the killed handler's marker past its harness
/// timeout.
fn past_timeout(w: &mut World) -> u64 {
    let hook = w.section::<BudgetSteps>().hook.clone();
    STARTED + budget_of(&hook).1.as_secs() + 1
}

#[then(
    regex = r#"^(the next hook handler|the MCP server of the next harness session as it starts|"cairn status") counts and audits it under "hook_unfinished"$"#
)]
fn next_counts(w: &mut World, next: String) -> Result<(), String> {
    let at = past_timeout(w);
    read_markers(w, &next, at)?;
    let (count, left) = (counted(w, "hook_unfinished")?, markers(w)?);
    if count == 1 && left.is_empty() {
        return Ok(());
    }
    let out = crate::record::last_run(w)?;
    Err(format!(
        "hook_unfinished was counted {count} times; markers left: {left:?}; {next} exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

#[then(regex = r"^a second reader does not count the same start marker again$")]
fn second_reader(w: &mut World) -> Result<(), String> {
    let at = past_timeout(w) + 60;
    read_markers(w, "\"cairn status\"", at)?;
    match counted(w, "hook_unfinished")? {
        1 => Ok(()),
        n => Err(format!("hook_unfinished was counted {n} times")),
    }
}

#[then(
    regex = r"^a start marker of a hook handler still within its harness timeout is not counted$"
)]
fn within_timeout(w: &mut World) -> Result<(), String> {
    let hook = w.section::<BudgetSteps>().hook.clone();
    let again = STARTED + 1_000;
    kill_hook(w, &hook, again)?;
    read_markers(w, "\"cairn status\"", again + 1)?;
    let (count, left) = (counted(w, "hook_unfinished")?, markers(w)?);
    if count == 1 && left.len() == 1 {
        return Ok(());
    }
    Err(format!(
        "hook_unfinished was counted {count} times; markers left: {left:?}"
    ))
}
