//! The steps of SRS §5.1a, administration (ADM). `@ADM-04` stays
//! `@pending`: its flags row needs PRV-07, outside M1. Its other rows
//! are bound here: a settings layer that fails validation fails `cairn
//! status`, naming the key and the problem, while the core records
//! every event of an agent's run.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use cucumber::{given, then, when};
use serde_json::Value;

use crate::World;
use crate::record::{last_run, run_core};

/// The transcript an agent's run leaves: one recorded harness session.
const TRANSCRIPT: &str = "testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/686aa14f-b535-45bc-9ed0-5400ea84bec7.jsonl";

/// What the administration steps track: the lines the agent's run left
/// and the events `cairn ingest` recorded of them.
#[derive(Debug, Default)]
pub struct AdministrationSteps {
    lines: usize,
    recorded: Option<u64>,
}

fn adm(w: &mut World) -> &mut AdministrationSteps {
    w.section::<AdministrationSteps>()
}

/// Writes the principal's configuration in the scenario's home,
/// owner-only as the home wants it.
pub fn configure(w: &World, text: &str) -> Result<(), String> {
    let home = crate::record::home(w)?;
    home.create().map_err(|e| e.to_string())?;
    let path = w.cairn_home.join("config.toml");
    fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())
}

/// An agent's run: the core ingests its transcript, as `cairn ingest`
/// does, and the step keeps how many lines it left and how many events
/// were recorded.
pub fn agent_runs(w: &mut World) -> Result<(), String> {
    let path = testkit::repo_root().join(TRANSCRIPT);
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let shown = path.display().to_string();
    run_core(w, &["ingest", "--path", &shown, "--json"], None)?;
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "cairn ingest exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let report: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let recorded = report["events"].as_u64();
    let state = adm(w);
    state.lines = text.lines().count();
    state.recorded = recorded;
    Ok(())
}

#[given(regex = r#"^the person's config\.toml containing "([^"]*)"$"#)]
fn config_containing(w: &mut World, setting: String) -> Result<(), String> {
    configure(w, &format!("{setting}\n"))
}

#[when(regex = r#"^an agent runs and the person starts "cairn ([^"]+)"$"#)]
fn agent_runs_and_person_starts(w: &mut World, args: String) -> Result<(), String> {
    agent_runs(w)?;
    let args: Vec<&str> = args.split_whitespace().collect();
    run_core(w, &args, None)
}

#[then(
    regex = r#"^"cairn [^"]+" exits (\d+), and the error names the key "([^"]+)" and the problem "([^"]+)"$"#
)]
fn exits_naming(w: &mut World, code: i32, key: String, problem: String) -> Result<(), String> {
    let out = last_run(w)?;
    let err = String::from_utf8_lossy(&out.stderr);
    let named = format!("key \"{key}\": {problem}");
    if out.status.code() == Some(code) && err.contains(&named) {
        return Ok(());
    }
    Err(format!(
        "the command exited {}, wanting {code} with {named:?}: {}",
        out.status,
        err.trim()
    ))
}

#[then(regex = r"^the core records every event of the run$")]
fn records_every_event(w: &mut World) -> Result<(), String> {
    let state = adm(w);
    if state.recorded == Some(state.lines as u64) && state.lines > 0 {
        return Ok(());
    }
    Err(format!(
        "the run left {} lines and the core recorded {:?} events",
        state.lines, state.recorded
    ))
}
