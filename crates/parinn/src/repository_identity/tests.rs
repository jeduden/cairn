use std::fs;
use std::path::{Path, PathBuf};

use crate::tests::{DEV, Process};
use crate::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";

/// A zlib stream of `data` in stored blocks.
fn zlib(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01, 1];
    out.extend_from_slice(&(data.len() as u16).to_le_bytes());
    out.extend_from_slice(&(!(data.len() as u16)).to_le_bytes());
    out.extend_from_slice(data);
    let (mut a, mut b) = (1_u32, 0_u32);
    for byte in data {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    out.extend_from_slice(&((b << 16) | a).to_be_bytes());
    out
}

/// A repository at `dir` with one root commit `id`, on `main`.
fn repository(dir: &Path, id: Option<&str>) {
    let git = dir.join(".git");
    fs::create_dir_all(git.join("refs/heads")).unwrap();
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    if let Some(id) = id {
        let body = format!("tree {}\n\nmessage\n", "0".repeat(40));
        let object = format!("commit {}\0{body}", body.len());
        let path = git.join("objects").join(&id[..2]).join(&id[2..]);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, zlib(object.as_bytes())).unwrap();
        fs::write(git.join("refs/heads/main"), format!("{id}\n")).unwrap();
    }
}

fn clone(p: &Process, name: &str, id: Option<&str>) -> PathBuf {
    let dir = p.dir.path().join(name);
    fs::create_dir_all(&dir).unwrap();
    repository(&dir, id);
    dir
}

fn run(p: &Process, args: &[&str]) -> (u8, String, String) {
    p.invoke(args, &DEV)
}

fn shown(p: &Process, dir: &Path) -> serde_json::Value {
    let (code, out, err) = run(
        p,
        &["repository", "show", &dir.display().to_string(), "--json"],
    );
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    serde_json::from_str(&out).unwrap()
}

#[test]
fn show_says_a_directory_is_unbound_with_the_identity_its_clone_names() {
    let p = Process::new();
    let app = clone(&p, "app", Some(A));

    let v = shown(&p, &app);

    assert_eq!(v["status"], "unbound");
    assert_eq!(v["repository"], A);
    let (code, text, _) = run(&p, &["repository", "show", &app.display().to_string()]);
    assert_eq!(code, EXIT_OK);
    assert_eq!(text.trim(), format!("unbound: {A}"));
}

#[test]
fn show_defaults_to_the_working_directory_and_names_a_provisional_identity() {
    let p = Process::new();
    repository(p.dir.path(), None);

    let (code, out, _) = run(&p, &["repository", "show"]);

    assert_eq!(code, EXIT_OK);
    assert_eq!(out.trim(), "unbound: provisional");
}

#[test]
fn show_names_a_directory_in_no_repository_and_one_it_cannot_read() {
    let p = Process::new();
    let plain = p.dir.path().join("plain");
    fs::create_dir_all(&plain).unwrap();
    let bad = clone(&p, "bad", Some(A));
    fs::write(
        bad.join(".git/config"),
        "[extensions]\n\tobjectformat = sha256\n",
    )
    .unwrap();

    assert_eq!(shown(&p, &plain)["status"], "no repository");
    assert_eq!(shown(&p, &bad)["status"], "unreadable");
}

#[test]
fn a_widening_bind_needs_a_terminal() {
    let mut p = Process::new();
    p.terminal = false;
    let app = clone(&p, "app", Some(A));

    let (code, _, err) = run(&p, &["repository", "bind", &app.display().to_string()]);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("terminal"), "{err}");
}

#[test]
fn a_widening_bind_records_the_act_and_binds_the_directory() {
    let p = Process::new();
    let app = clone(&p, "app", Some(A));

    let (code, out, err) = run(
        &p,
        &["repository", "bind", &app.display().to_string(), "--json"],
    );

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["repository"], A);
    assert_eq!(shown(&p, &app)["status"], "bound");
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    let acts: i64 = conn
        .query_row(
            "SELECT count(*) FROM events WHERE kind = 'principal_act' AND content LIKE '%repository_bind%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(acts, 1);
}

#[test]
fn a_widening_bind_rebinds_after_a_change_and_may_name_a_commit_the_clone_lacks() {
    let p = Process::new();
    let app = clone(&p, "app", Some(A));
    let at = app.display().to_string();
    assert_eq!(run(&p, &["repository", "bind", &at]).0, EXIT_OK);
    fs::remove_dir_all(app.join(".git")).unwrap();
    repository(&app, Some(B));
    assert_eq!(shown(&p, &app)["status"], "changed");

    assert_eq!(run(&p, &["repository", "bind", &at]).0, EXIT_OK);
    assert_eq!(shown(&p, &app)["status"], "bound");
    let lacking = clone(&p, "lacking", Some(B));
    let (code, out, _) = run(
        &p,
        &[
            "repository",
            "bind",
            &lacking.display().to_string(),
            "--commit",
            A,
            "--json",
        ],
    );
    assert_eq!(code, EXIT_OK);
    assert!(out.contains(A), "{out}");
}

#[test]
fn a_bind_of_a_directory_in_no_repository_or_with_a_bad_commit_fails() {
    let p = Process::new();
    let plain = p.dir.path().join("plain");
    fs::create_dir_all(&plain).unwrap();
    let app = clone(&p, "app", Some(A));

    let (code, _, err) = run(&p, &["repository", "bind", &plain.display().to_string()]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("lies in no repository"), "{err}");
    let (code, _, err) = run(
        &p,
        &[
            "repository",
            "bind",
            &app.display().to_string(),
            "--commit",
            "nope",
        ],
    );
    assert_eq!(code, EXIT_USAGE);
    assert!(err.contains("40 lowercase hexadecimal"), "{err}");
}

#[test]
fn the_repository_verbs_refuse_what_they_do_not_know() {
    let p = Process::new();

    for args in [
        &["repository"][..],
        &["repository", "forget"][..],
        &["repository", "show", "--now"][..],
        &["repository", "show", "a", "b"][..],
        &["repository", "show", "--commit", A][..],
        &["repository", "bind", "--commit"][..],
    ] {
        assert_eq!(run(&p, args).0, EXIT_USAGE, "{args:?}");
    }
}

/// Runs the `SessionStart` hook in `cwd`.
fn start_in(p: &mut Process, session: &str, cwd: &Path) {
    let path = p.dir.path().join(format!("{session}.jsonl"));
    p.lines = vec![
        serde_json::json!({
            "session_id": session,
            "transcript_path": path,
            "cwd": cwd,
            "hook_event_name": "SessionStart",
            "source": "startup",
        })
        .to_string(),
    ];
    assert_eq!(p.invoke(&["hook", "SessionStart"], &DEV).0, EXIT_OK);
    p.lines = Vec::new();
}

/// The repository bind events the record holds.
fn binds(p: &Process) -> Vec<serde_json::Value> {
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    let mut statement = conn
        .prepare("SELECT content FROM events WHERE kind = 'repository_bind' ORDER BY seq")
        .unwrap();
    statement
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(|c| serde_json::from_str(&c.unwrap()).unwrap())
        .collect()
}

fn counter(p: &Process, name: &str) -> u64 {
    let (_, out, _) = run(p, &["status", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    v["counters"][name]["total"].as_u64().unwrap_or(0)
}

#[test]
fn the_first_hook_event_in_a_clone_binds_its_identity_when_ingest_next_runs_once() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some(A));
    start_in(&mut p, "s1", &app);

    assert_eq!(run(&p, &["ingest", "--all"]).0, EXIT_OK);
    start_in(&mut p, "s2", &app);
    assert_eq!(run(&p, &["ingest", "--all"]).0, EXIT_OK);

    assert_eq!(binds(&p), [serde_json::json!({"repository": A})]);
    assert_eq!(shown(&p, &app)["status"], "bound");
}

#[test]
fn an_mcp_server_binds_the_identity_of_the_clones_its_hooks_showed_too() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some(A));
    start_in(&mut p, "s1", &app);

    assert_eq!(run(&p, &["mcp"]).0, EXIT_OK);

    assert_eq!(binds(&p).len(), 1);
}

#[test]
fn a_change_of_identity_is_audited_and_counted_once_and_relinks_nothing() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some(A));
    start_in(&mut p, "s1", &app);
    run(&p, &["ingest", "--all"]);
    fs::remove_dir_all(app.join(".git")).unwrap();
    repository(&app, Some(B));
    start_in(&mut p, "s2", &app);

    run(&p, &["ingest", "--all"]);
    run(&p, &["ingest", "--all"]);

    assert_eq!(counter(&p, "repository_identity_changed"), 1);
    assert_eq!(binds(&p).len(), 1);
    let audit = crate::tests::audited(&p);
    assert!(
        audit
            .iter()
            .any(|e| e.detail.contains("repository identity changed")),
        "{audit:?}"
    );
    assert!(
        audit
            .iter()
            .all(|e| !e.detail.contains(A) && !e.detail.contains(B))
    );
}

#[test]
fn a_clone_that_cannot_be_read_is_counted_once() {
    let mut p = Process::new();
    let bad = clone(&p, "bad", Some(A));
    fs::write(
        bad.join(".git/config"),
        "[extensions]\n\tobjectformat = sha256\n",
    )
    .unwrap();
    start_in(&mut p, "s1", &bad);

    run(&p, &["ingest", "--all"]);
    run(&p, &["ingest", "--all"]);

    assert_eq!(counter(&p, "repository_read_failed"), 1);
    assert!(binds(&p).is_empty());
}
