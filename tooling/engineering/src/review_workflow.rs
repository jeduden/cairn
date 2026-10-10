//! ENG-28: the review workflow keeps the reviewing agent away from the
//! reviewer app's key, and the review gate that posts the review
//! decides as the requirement says.

use std::collections::BTreeMap;

use crate::list;

/// The action that runs the reviewing agent.
const AGENT_ACTION: &str = "anthropics/claude-code-action@";

/// A GitHub Actions workflow split the way the ENG-28 steps read it:
/// its triggers and its jobs, comments left out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Workflow {
    pub triggers: Vec<String>,
    /// The lines of the `on:` block.
    pub on: String,
    pub jobs: Vec<Job>,
}

/// One job's name and its lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Job {
    pub name: String,
    pub text: String,
}

/// Splits `body` into its `on:` block and its jobs. It reads only the
/// block layout the repository's workflows use: two-space indents, and
/// triggers and jobs as keys one level down.
#[must_use]
pub fn parse(body: &str) -> Workflow {
    let mut w = Workflow::default();
    let mut top = "";
    for line in body.split_inclusive('\n').map(uncomment) {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(key) = top_key(&line) {
            top = key;
            continue;
        }
        match (top, sub_key(&line)) {
            ("on", Some(key)) => {
                w.triggers.push(key.to_owned());
                w.on.push_str(&line);
            }
            ("on", None) => w.on.push_str(&line),
            ("jobs", Some(key)) => w.jobs.push(Job {
                name: key.to_owned(),
                text: String::new(),
            }),
            ("jobs", None) => {
                if let Some(job) = w.jobs.last_mut() {
                    job.text.push_str(&line);
                }
            }
            _ => {}
        }
    }

    w
}

/// The key a line names at the top level, such as `on` in `on:`.
fn top_key(line: &str) -> Option<&'static str> {
    let (key, _) = line.split_once(':')?;
    let valid = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '_' || c == '-');
    // The two keys the steps read; any other top-level key ends them.
    valid.then_some(match key {
        "on" => "on",
        "jobs" => "jobs",
        _ => "other",
    })
}

/// The key a line names one level down, at a two-space indent.
fn sub_key(line: &str) -> Option<&str> {
    let (key, _) = line.strip_prefix("  ")?.split_once(':')?;
    let valid = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

    valid.then_some(key)
}

/// Drops a YAML comment: a whole comment line, or one that follows a
/// space.
fn uncomment(line: &str) -> String {
    if line.trim_start().starts_with('#') {
        return String::new();
    }
    match line.find(" #") {
        Some(i) => format!("{}\n", &line[..i]),
        None => line.to_owned(),
    }
}

/// Whether `text` grants a write permission: a `:` followed by
/// `write`, quoted or not, as a whole word, or `write-all`.
fn grants_write(text: &str) -> bool {
    text.match_indices(':').any(|(i, _)| {
        let rest = text[i + 1..].trim_start().trim_start_matches(['"', '\'']);
        rest.strip_prefix("write")
            .is_some_and(|after| !after.starts_with(is_word))
    })
}

/// Whether `text` holds `word` with no word character on either side.
fn has_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        !before.is_some_and(is_word) && !after.is_some_and(is_word)
    })
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl Workflow {
    /// Requires `workflow_run` as the only trigger, on the named
    /// workflow's completion. GitHub runs a `workflow_run` workflow as
    /// the default branch defines it, so a pull request cannot rewrite
    /// its own reviewer; any other trigger would let a branch's copy
    /// run.
    ///
    /// # Errors
    ///
    /// Fails on another trigger, or one that names another workflow.
    pub fn runs_after(&self, name: &str) -> Result<(), String> {
        if self.triggers != ["workflow_run"] {
            return Err(format!(
                "the review workflow triggers on {}, want only workflow_run",
                list(&self.triggers)
            ));
        }
        for want in [
            format!("workflows: [{name}]"),
            "types: [completed]".to_owned(),
        ] {
            if !self.on.contains(&want) {
                return Err(format!("the workflow_run trigger does not say {want:?}"));
            }
        }

        Ok(())
    }

    /// The jobs that run the reviewing agent.
    fn agent_jobs(&self) -> impl Iterator<Item = &Job> {
        self.jobs.iter().filter(|j| j.text.contains(AGENT_ACTION))
    }

    /// Requires an agent job, and that none of them is granted a write
    /// permission or reads `key`.
    #[must_use]
    pub fn agents_unprivileged(&self, key: &str) -> Vec<String> {
        let mut problems = Vec::new();
        for j in self.agent_jobs() {
            if grants_write(&j.text) {
                problems.push(format!(
                    "agent job {} is granted a write permission",
                    j.name
                ));
            }
            if j.text.contains(&format!("secrets.{key}")) {
                problems.push(format!("agent job {} reads {key}", j.name));
            }
        }
        if self.agent_jobs().next().is_none() {
            problems.push("no job runs the reviewing agent".to_owned());
        }

        problems
    }

    /// The one job that reads `key`.
    fn key_job(&self, key: &str) -> Result<&Job, String> {
        let holders: Vec<&Job> = self
            .jobs
            .iter()
            .filter(|j| j.text.contains(&format!("secrets.{key}")))
            .collect();
        match holders.as_slice() {
            [one] => Ok(one),
            _ => Err(format!(
                "{} jobs read {key:?}, want exactly one",
                holders.len()
            )),
        }
    }

    /// Requires one job to hold `key`, and that job to run no agent.
    ///
    /// # Errors
    ///
    /// Fails when no job or several hold the key, or its job runs the
    /// agent.
    pub fn key_held_apart(&self, key: &str) -> Result<(), String> {
        let j = self.key_job(key)?;
        if j.text.contains(AGENT_ACTION) {
            return Err(format!("job {} holds {key} and runs the agent", j.name));
        }

        Ok(())
    }

    /// Requires the key's job to need every agent job, and not to run
    /// when one of them failed.
    ///
    /// # Errors
    ///
    /// Fails when the key's job cannot be found.
    pub fn key_job_after_agents(&self, key: &str) -> Result<Vec<String>, String> {
        let j = self.key_job(key)?;
        let needs = j
            .text
            .lines()
            .rfind(|l| l.trim_start().starts_with("needs:"))
            .unwrap_or_default();
        let mut problems: Vec<String> = self
            .agent_jobs()
            .filter(|a| !has_word(needs, &a.name))
            .map(|a| format!("job {} does not need agent job {}", j.name, a.name))
            .collect();
        for over in ["always()", "failure()", "cancelled()"] {
            if j.text.contains(over) {
                problems.push(format!(
                    "job {} runs on {over}, even after a failed review",
                    j.name
                ));
            }
        }

        Ok(problems)
    }

    /// Requires the key's job to run `cmd`.
    ///
    /// # Errors
    ///
    /// Fails when the key's job cannot be found or does not run `cmd`.
    pub fn key_job_runs(&self, key: &str, cmd: &str) -> Result<(), String> {
        let j = self.key_job(key)?;
        if !j.text.contains(cmd) {
            return Err(format!("job {} does not run {cmd:?}", j.name));
        }

        Ok(())
    }
}

/// Keys each body row of `rows` by the header row.
///
/// # Errors
///
/// Fails when the table has no rows under its header.
pub fn table_rows(rows: &[Vec<String>]) -> Result<Vec<BTreeMap<String, String>>, String> {
    let [header, body @ ..] = rows else {
        return Err("the table has no rows under its header".to_owned());
    };
    if body.is_empty() {
        return Err("the table has no rows under its header".to_owned());
    }

    Ok(body
        .iter()
        .map(|row| header.iter().cloned().zip(row.iter().cloned()).collect())
        .collect())
}

/// Runs every row through the review gate and compares the outcome
/// with the row's review column.
#[must_use]
pub fn gate_decides(rows: &[BTreeMap<String, String>]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let got = gate_outcome(row);
            (row.get("review").map(String::as_str) != Some(got))
                .then(|| format!("row {} {row:?}: the review gate gives {got}", i + 1))
        })
        .collect()
}

/// Runs the review gate's command line, as the review workflow does,
/// on the files one table row describes, and names what it does: the
/// review event it prints, "nothing" for a head that moved, or "an
/// error" when it exits unsuccessfully. A `missing` verdict leaves the
/// outcome file out.
#[must_use]
pub fn gate_outcome(row: &BTreeMap<String, String>) -> &'static str {
    const REVIEWED: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";
    const MOVED: &str = "490efa734617788d1dc1f831b26d0e23888e90fa";
    let cell = |name: &str| row.get(name).map(String::as_str).unwrap_or_default();
    let findings = match cell("finding") {
        "none" => "[]".to_owned(),
        severity => format!(r#"[{{"path":"a.rs","line":1,"severity":{severity:?},"body":"b"}}]"#),
    };
    let conclusion = |cell: &str| {
        if cell == "passed" {
            "success"
        } else {
            "failure"
        }
    };
    let mut files = BTreeMap::from([(
        "check-runs.jsonl",
        format!(
            "{{\"name\":\"CI\",\"status\":\"completed\",\"conclusion\":{:?}}}\n\
             {{\"name\":\"lint\",\"status\":\"completed\",\"conclusion\":{:?}}}\n",
            conclusion(cell("CI")),
            conclusion(cell("other check"))
        ),
    )]);
    if cell("verdict") != "missing" {
        files.insert(
            "outcome.json",
            format!(
                r#"{{"verdict":{:?},"summary":"s","findings":{findings}}}"#,
                cell("verdict")
            ),
        );
    }
    let head = if cell("head") == "moved" {
        MOVED
    } else {
        REVIEWED
    };
    let args = [
        "--outcome",
        "outcome.json",
        "--check-runs",
        "check-runs.jsonl",
        "--reviewed",
        REVIEWED,
        "--head",
        head,
    ];
    let read = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = review_gate::cli::run(args.map(str::to_owned), &mut out, &mut err, &read);
    match serde_json::from_slice::<review_gate::Review>(&out) {
        _ if code != review_gate::cli::EXIT_OK => "an error",
        _ if out.is_empty() => "nothing",
        Ok(review) if review.event == review_gate::APPROVE => review_gate::APPROVE,
        _ => review_gate::REQUEST_CHANGES,
    }
}

#[cfg(test)]
mod tests;
