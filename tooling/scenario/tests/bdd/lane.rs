//! The steps of SRS §5.2, rooms and lanes (LANE), that phase 11 of plan
//! 2609292012 carries. `@LANE-02`: a repository's identity is a
//! parentless commit of its default branch, bound once by a structural
//! event, independent of the local path; a change in what a directory
//! resolves to is audited and rebinds only on the widening principal act
//! `parinn repository bind`.
//!
//! The bind is recorded when the hook handler's `SessionStart`
//! observation reaches the record, as `parinn ingest` or an MCP server
//! starts: reading a clone's git data cannot stay within the handler's
//! hook budget (NFR-02), and the observation is the first hook event.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::World;
use crate::record::{home, last_run, run_core, run_core_at_terminal, run_core_with};

/// What the repository steps track: the clone, and the identity git
/// names for it.
#[derive(Debug, Default)]
pub struct LaneSteps {
    clone: Option<PathBuf>,
    root: Option<String>,
}

fn lane(w: &mut World) -> &mut LaneSteps {
    w.section::<LaneSteps>()
}

/// Runs `git` in `dir` as a person with no configuration of their own.
fn git(w: &World, dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = testkit::isolate(&mut Command::new("git"), &w.home)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=a",
            "-c",
            "user.email=a@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Makes a repository at `dir`, with a root commit when `commits`.
fn make(w: &World, dir: &Path, commits: bool) -> Result<Option<String>, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    git(w, dir, &["init", "-q"])?;
    if !commits {
        return Ok(None);
    }
    fs::write(dir.join("a.txt"), "a").map_err(|e| e.to_string())?;
    git(w, dir, &["add", "."])?;
    git(w, dir, &["commit", "-q", "-m", "root"])?;
    git(w, dir, &["rev-list", "--max-parents=0", "HEAD"]).map(Some)
}

#[given(
    regex = r"^a repository clone that (contains the bound parentless commit of the default branch|has no commit)$"
)]
fn clone_that(w: &mut World, how: String) -> Result<(), String> {
    let dir = w.home.join("work").join("app");
    let root = make(w, &dir, how.starts_with("contains"))?;
    let steps = lane(w);
    steps.clone = Some(dir);
    steps.root = root;
    Ok(())
}

/// The first hook event for `dir`, and the `parinn ingest` that records
/// what it observed.
fn first_hook_event(w: &mut World, dir: &Path, session: &str) -> Result<(), String> {
    let path = w.home.join(format!("{session}.jsonl"));
    let input = json!({
        "session_id": session,
        "transcript_path": path,
        "cwd": dir,
        "hook_event_name": "SessionStart",
        "source": "startup",
    });
    run_core_with(
        w,
        &["hook", "SessionStart"],
        Some(input.to_string().as_bytes()),
        &[],
    )?;
    run_core(w, &["ingest", "--all"], None)?;
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn ingest: {}",
        String::from_utf8_lossy(&out.stderr)
    ))
}

fn clone_dir(w: &mut World) -> Result<PathBuf, String> {
    lane(w)
        .clone
        .clone()
        .ok_or_else(|| "no step made a clone".to_owned())
}

#[when(regex = r"^the first hook event for the clone arrives$")]
fn first_event(w: &mut World) -> Result<(), String> {
    let dir = clone_dir(w)?;
    first_hook_event(w, &dir, "lane-02-first")
}

/// What `parinn repository show` says of `dir`.
fn shown(w: &mut World, dir: &Path) -> Result<Value, String> {
    run_core(
        w,
        &["repository", "show", &dir.display().to_string(), "--json"],
        None,
    )?;
    let out = last_run(w)?;
    serde_json::from_slice(&out.stdout).map_err(|e| {
        format!(
            "parinn repository show: {e}: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

#[then(
    regex = r"^the repository identity is (the bound identity|a provisional, node-local identity)$"
)]
fn identity_is(w: &mut World, which: String) -> Result<(), String> {
    let dir = clone_dir(w)?;
    let v = shown(w, &dir)?;
    let root = lane(w).root.clone();
    let wanted = if which == "the bound identity" {
        json!(root)
    } else {
        Value::Null
    };
    if v["status"] != "bound" || v["repository"] != wanted {
        return Err(format!("the directory shows {v}, wanting {which}"));
    }
    if root.is_none() {
        let id = provisional_id(w)?;
        if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("the provisional id is not 32 hex digits: {id}"));
        }
    }
    Ok(())
}

/// The provisional id the record's one bind carries.
fn provisional_id(w: &World) -> Result<String, String> {
    let binds = events(w, "repository_bind")?;
    match binds.as_slice() {
        [(_, c)] => c["repository_provisional"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("the bind holds no provisional id: {c}")),
        _ => Err(format!("wanting one bind, the record holds {binds:?}")),
    }
}

/// The events of `kind` the record holds, with their provenance and
/// content.
fn events(w: &World, kind: &str) -> Result<Vec<(String, Value)>, String> {
    let conn = rusqlite::Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    let mut statement = conn
        .prepare("SELECT provenance, content FROM events WHERE kind = ?1 ORDER BY writer, seq")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([kind], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    rows.map(|row| {
        let (provenance, content) = row.map_err(|e| e.to_string())?;
        Ok((
            provenance,
            serde_json::from_str(&content).map_err(|e| e.to_string())?,
        ))
    })
    .collect()
}

#[then(
    regex = r#"^a repository's first bind is a structural event ParleyInn records at the first hook event, never an "operator" event$"#
)]
fn first_bind_is_structural(w: &mut World) -> Result<(), String> {
    let binds = events(w, "repository_bind")?;
    if binds.len() != 1 || binds[0].0 != "structural" {
        return Err(format!(
            "the record holds {binds:?}, wanting one structural bind"
        ));
    }
    let acts = events(w, "principal_act")?;
    match acts.iter().find(|(_, c)| c["act"] == "repository_bind") {
        Some(act) => Err(format!("an operator act bound it: {act:?}")),
        None => Ok(()),
    }
}

#[then(regex = r"^no identity is minted from the local path$")]
fn no_identity_from_the_path(w: &mut World) -> Result<(), String> {
    // A second clone at another path names the same identity and binds
    // nothing more; a provisional one's id holds no path.
    let first = clone_dir(w)?;
    let root = lane(w).root.clone();
    if root.is_some() {
        let second = w.home.join("elsewhere").join("copy");
        fs::create_dir_all(second.parent().ok_or("no parent")?).map_err(|e| e.to_string())?;
        git(
            w,
            &w.home.clone(),
            &[
                "clone",
                "-q",
                &first.display().to_string(),
                &second.display().to_string(),
            ],
        )?;
        first_hook_event(w, &second, "lane-02-second")?;
        let v = shown(w, &second)?;
        // The copy is another worktree of the same identity: one more
        // bind, naming that identity and a worktree of its own.
        let binds = events(w, "repository_bind")?;
        let digests: Vec<_> = binds
            .iter()
            .map(|(_, c)| c["worktree_digest"].clone())
            .collect();
        let same = binds
            .iter()
            .all(|(_, c)| json!(c["repository"]) == json!(root));
        if v["repository"] != json!(root) || binds.len() != 2 || !same || digests[0] == digests[1] {
            return Err(format!("the copy at another path shows {v}: {binds:?}"));
        }
    } else {
        let first_id = provisional_id(w)?;
        let second = w.home.join("elsewhere").join("empty");
        fs::create_dir_all(&second).map_err(|e| e.to_string())?;
        git(w, &second, &["init", "-q"])?;
        first_hook_event(w, &second, "lane-02-empty")?;
        let ids: Vec<String> = events(w, "repository_bind")?
            .iter()
            .filter_map(|(_, c)| c["repository_provisional"].as_str().map(str::to_owned))
            .collect();
        if ids.len() != 2 || ids[0] == ids[1] || !ids.contains(&first_id) {
            return Err(format!("two empty clones share a provisional id: {ids:?}"));
        }
    }
    for (_, content) in events(w, "repository_bind")? {
        if content.to_string().contains(&first.display().to_string()) {
            return Err(format!("a bind holds the path: {content}"));
        }
    }
    Ok(())
}

/// Every file under `dir`, with its bytes.
fn files(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            files(&path, out)?;
        } else {
            out.push((path.clone(), fs::read(&path).map_err(|e| e.to_string())?));
        }
    }
    Ok(())
}

#[then(
    regex = r"^no name ParleyInn derives from the identity for its local state reveals anything about the identity off this node$"
)]
fn no_local_name_reveals_it(w: &mut World) -> Result<(), String> {
    let dir = clone_dir(w)?;
    let secrets: Vec<String> = [lane(w).root.clone(), Some(dir.display().to_string())]
        .into_iter()
        .flatten()
        .collect();
    let conn = rusqlite::Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    let mut statement = conn
        .prepare("SELECT worktree, identity FROM repository_worktrees")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String)> = statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Err("the node keeps no name for the directory".to_owned());
    }
    for (directory, identity) in &rows {
        if secrets
            .iter()
            .any(|s| directory.contains(s) || identity.contains(s))
        {
            return Err(format!("a local name reveals it: {directory} {identity}"));
        }
    }
    // Nor does any file of the home but the record (the audit log, the
    // settings, the hook state).
    let mut all = Vec::new();
    files(home(w)?.root(), &mut all)?;
    let root = lane(w).root.clone().unwrap_or_default();
    for (path, bytes) in all {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.starts_with("store.db") || path.to_string_lossy().contains("/payloads/") {
            continue;
        }
        if !root.is_empty() && bytes.windows(root.len()).any(|b| b == root.as_bytes()) {
            return Err(format!("{} holds the identity", path.display()));
        }
    }
    Ok(())
}

/// How many times the change of a repository identity is counted.
fn changed(w: &mut World) -> Result<u64, String> {
    run_core(w, &["status", "--json"], None)?;
    let status: Value = serde_json::from_slice(&last_run(w)?.stdout).map_err(|e| e.to_string())?;
    Ok(status["counters"]["repository_identity_changed"]["total"]
        .as_u64()
        .unwrap_or(0))
}

/// Replaces the repository in `dir` with a new one whose root commit
/// is `message`.
fn swap_repository(w: &World, dir: &Path, message: &str) -> Result<(), String> {
    fs::remove_dir_all(dir.join(".git")).map_err(|e| e.to_string())?;
    fs::write(dir.join("b.txt"), message).map_err(|e| e.to_string())?;
    git(w, dir, &["init", "-q"])?;
    git(w, dir, &["add", "."])?;
    git(w, dir, &["commit", "-q", "-m", message])?;
    Ok(())
}

#[then(
    regex = r#"^a later change in the identity the directory resolves to is audited, rewrites no room's branch links silently, rebinds only on the widening principal act "parinn repository bind", the same act that binds by hand, and scopes no ParleyInn state$"#
)]
fn a_later_change(w: &mut World) -> Result<(), String> {
    let dir = clone_dir(w)?;
    if lane(w).root.is_none() {
        // A provisional identity binds on its own once its clone names a
        // commit: no change, and the record is not rewritten.
        swap_repository(w, &dir, "first root")?;
        first_hook_event(w, &dir, "lane-02-binds-later")?;
        if shown(w, &dir)?["status"] != "bound" || changed(w)? != 0 {
            return Err("a provisional identity did not bind on its own".to_owned());
        }
        // The commit's bind names the provisional identity it replaces,
        // for the same worktree.
        let binds = events(w, "repository_bind")?;
        let linked = binds.iter().any(|(_, later)| {
            later["repository"].is_string()
                && binds.iter().any(|(_, first)| {
                    first["worktree_digest"] == later["worktree_digest"]
                        && first["repository_provisional"].is_string()
                        && later["replaces_provisional"] == first["repository_provisional"]
                })
        });
        if !linked {
            return Err(format!(
                "the later bind does not name what it replaces: {binds:?}"
            ));
        }
    }
    let before = events(w, "repository_bind")?;
    // The directory now holds another repository.
    swap_repository(w, &dir, "another root")?;
    first_hook_event(w, &dir, "lane-02-later")?;
    first_hook_event(w, &dir, "lane-02-later-again")?;
    if changed(w)? != 1 {
        return Err("the change is not counted once".to_owned());
    }
    let audit = audit::log::Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    if !audit
        .iter()
        .any(|e| e.detail.contains("repository identity changed"))
    {
        return Err("no audit entry records the change".to_owned());
    }
    // M1 holds no room, so no branch link exists to be rewritten: the
    // clauses on branch links and on state hold vacuously.
    if events(w, "repository_bind")? != before || shown(w, &dir)?["status"] != "changed" {
        return Err("the change rebound the directory or relinked it".to_owned());
    }
    run_core_at_terminal(w, &["repository", "bind", &dir.display().to_string()])?;
    if !last_run(w)?.status.success() || shown(w, &dir)?["status"] != "bound" {
        return Err("the widening bind did not rebind the directory".to_owned());
    }
    // Only that act rebinds: away from a terminal the same bind is refused.
    let before_acts = events(w, "principal_act")?.len();
    run_core(w, &["repository", "bind", &dir.display().to_string()], None)?;
    if last_run(w)?.status.success() || events(w, "principal_act")?.len() != before_acts {
        return Err("a bind away from a terminal was not refused".to_owned());
    }
    // The same act binds by hand a directory no hook event has shown.
    let by_hand = w.home.join("work").join("by-hand");
    make(w, &by_hand, true)?;
    if shown(w, &by_hand)?["status"] != "unbound" {
        return Err("a directory no hook showed is bound".to_owned());
    }
    run_core_at_terminal(w, &["repository", "bind", &by_hand.display().to_string()])?;
    if !last_run(w)?.status.success() || shown(w, &by_hand)?["status"] != "bound" {
        return Err("the widening bind did not bind a directory by hand".to_owned());
    }
    let acts = events(w, "principal_act")?;
    if !acts
        .iter()
        .any(|(p, c)| p == "operator" && c["act"] == "repository_bind")
    {
        return Err("the rebind is no operator act".to_owned());
    }
    Ok(())
}
