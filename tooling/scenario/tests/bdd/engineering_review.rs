//! The ENG-28 steps: the review workflow keeps the reviewing agent away
//! from the reviewer app's key, and the review gate decides as the
//! requirement says.

use cucumber::gherkin::Step;
use cucumber::{then, when};
use engineering::{outcome, review_workflow};

use crate::World;
use crate::engineering::eng;

#[when(regex = r#"^the review workflow is read from "([^"]+)"$"#)]
fn read_review_workflow(w: &mut World, file: String) -> Result<(), String> {
    let body = eng(w).checkout()?.read(&file)?;
    eng(w).flow = review_workflow::parse(&body);
    Ok(())
}

#[then(
    regex = r#"^it runs only when the "([^"]+)" workflow completes, as the default branch defines it$"#
)]
fn runs_after(w: &mut World, name: String) -> Result<(), String> {
    eng(w).flow.runs_after(&name)
}

#[then(
    regex = r#"^the job that runs the reviewing agent holds no write permission and not the "([^"]+)" key$"#
)]
fn agents_unprivileged(w: &mut World, key: String) -> Result<(), String> {
    outcome(eng(w).flow.agents_unprivileged(&key))
}

#[then(regex = r#"^only one job holds the "([^"]+)" key, and it runs no agent$"#)]
fn key_held_apart(w: &mut World, key: String) -> Result<(), String> {
    eng(w).flow.key_held_apart(&key)
}

#[then(regex = r"^that job runs only after the agent's job succeeded$")]
fn key_job_after_agents(w: &mut World) -> Result<(), String> {
    outcome(eng(w).flow.key_job_after_agents(KEY)?)
}

#[then(regex = r#"^that job decides the review with "([^"]+)"$"#)]
fn key_job_runs(w: &mut World, cmd: String) -> Result<(), String> {
    eng(w).flow.key_job_runs(KEY, &cmd)
}

#[then(regex = r"^the review gate decides:$")]
fn review_gate_decides(_w: &mut World, step: &Step) -> Result<(), String> {
    let table = step.table.as_ref().ok_or("the step has no table")?;
    outcome(review_workflow::gate_decides(&review_workflow::table_rows(
        &table.rows,
    )?))
}

/// The reviewer app's key, as the scenario names it.
const KEY: &str = "JEDUDEN_REVIEW_AGENT_KEY";
