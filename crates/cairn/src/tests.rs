use super::*;
use std::io;
use std::sync::mpsc;
use testkit::TempDir;

const DEV: Build<'static> = Build {
    version: "dev",
    commit: None,
};

/// A process's surroundings: a fresh home, and input that closes at
/// once unless a test hands lines.
struct Process {
    dir: TempDir,
    lines: Vec<String>,
    /// The input fails after its lines rather than closing.
    input_error: bool,
    /// The input stays open after its lines, as a harness that never
    /// closes it would.
    held_open: bool,
    home_id: Option<String>,
    hook_panics: bool,
}

/// The time every test's clock reads.
const NOW: u64 = 1_760_000_000;

impl Process {
    fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
            lines: Vec::new(),
            input_error: false,
            held_open: false,
            home_id: None,
            hook_panics: false,
        }
    }

    fn cairn_home(&self) -> std::path::PathBuf {
        self.dir.path().join(".cairn")
    }

    fn invoke(&self, args: &[&str], build: &Build) -> (u8, String, String) {
        let (lines, input_error, held_open) =
            (self.lines.clone(), self.input_error, self.held_open);
        let input = move || {
            let (tx, rx) = mpsc::channel();
            for line in lines.clone() {
                tx.send(Ok(line)).unwrap();
            }
            if input_error {
                tx.send(Err(io::Error::new(io::ErrorKind::InvalidData, "not UTF-8")))
                    .unwrap();
            }
            if held_open {
                std::mem::forget(tx);
            }
            rx
        };
        let ctx = Context {
            build,
            cairn_home: None,
            home: Some(self.dir.path().as_os_str().to_owned()),
            home_id: self.home_id.as_ref().map(Into::into),
            hook_panics: self.hook_panics,
            input: &input,
            clock: &|| NOW,
        };
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(
            args.iter().map(|a| (*a).to_owned()),
            &ctx,
            &mut out,
            &mut err,
        );

        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn transcript(&self, text: &str) -> String {
        let path = self.dir.path().join("t.jsonl");
        std::fs::write(&path, text).unwrap();
        path.display().to_string()
    }
}

fn invoke(args: &[&str], build: &Build) -> (u8, String, String) {
    Process::new().invoke(args, build)
}

#[test]
fn no_command_is_a_usage_error() {
    let (code, out, err) = invoke(&[], &DEV);

    assert_eq!(code, EXIT_USAGE);
    assert_eq!(out, "");
    assert!(err.starts_with("usage: cairn"), "{err}");
}

#[test]
fn help_prints_the_usage_to_stdout() {
    for arg in ["help", "-h", "--help"] {
        let (code, out, err) = invoke(&[arg], &DEV);

        assert_eq!(code, EXIT_OK, "{arg}");
        assert!(out.starts_with("usage: cairn"), "{arg}: {out}");
        assert_eq!(err, "", "{arg}");
    }
}

#[test]
fn an_unknown_command_is_a_usage_error() {
    let (code, _, err) = invoke(&["rebuild"], &DEV);

    assert_eq!(code, EXIT_USAGE);
    assert!(
        err.starts_with("cairn: unknown command \"rebuild\"\n\nusage: cairn"),
        "{err}"
    );
}

#[test]
fn version_prints_the_stamped_version() {
    let build = Build {
        version: "v1.2.3",
        commit: Some("abc"),
    };

    assert_eq!(
        invoke(&["version"], &build),
        (EXIT_OK, "v1.2.3\n".into(), String::new())
    );
}

#[test]
fn version_json_carries_the_version_and_the_commit() {
    let build = Build {
        version: "v1.2.3",
        commit: Some("abc"),
    };

    let (code, out, _) = invoke(&["version", "--json"], &build);

    assert_eq!(code, EXIT_OK);
    assert_eq!(out, "{\"version\":\"v1.2.3\",\"commit\":\"abc\"}\n");
}

#[test]
fn version_json_leaves_out_a_commit_the_build_did_not_stamp() {
    assert_eq!(
        invoke(&["version", "--json"], &DEV).1,
        "{\"version\":\"dev\"}\n"
    );
}

#[test]
fn version_refuses_an_unknown_flag() {
    let (code, out, err) = invoke(&["version", "--yaml"], &DEV);

    assert_eq!(code, EXIT_USAGE);
    assert_eq!(out, "");
    assert_eq!(err, "cairn version: unknown flag \"--yaml\"\n");
}

#[test]
fn json_string_escapes_quotes_backslashes_and_control_characters() {
    assert_eq!(
        json_string("a\"b\\c\nd\u{1}é"),
        r#""a\"b\\c\u000ad\u0001é""#
    );
}

/// Refuses every write, standing in for a closed stdout.
struct Closed;

impl io::Write for Closed {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_failed_write_is_a_failure() {
    let p = Process::new();
    let path = p.transcript("x\n");
    for args in [
        &["version"][..],
        &["version", "--json"],
        &["help"],
        &["ingest", "--path", &path],
        &["verify"],
        &["counter", "ack"],
        &["doctor"],
        &["status"],
    ] {
        let mut err = Vec::new();
        let input = || mpsc::channel().1;
        let ctx = Context {
            build: &DEV,
            cairn_home: Some(p.cairn_home().into_os_string()),
            home: None,
            home_id: None,
            hook_panics: false,
            input: &input,
            clock: &|| NOW,
        };

        let code = run(
            args.iter().map(|a| (*a).to_owned()),
            &ctx,
            &mut Closed,
            &mut err,
        );

        assert_eq!(code, EXIT_FAILURE, "{args:?}");
        assert!(
            String::from_utf8(err).unwrap().contains("closed"),
            "{args:?}"
        );
    }
}

#[test]
fn the_build_is_stamped_from_the_environment_or_reports_dev() {
    assert_eq!(BUILD.version, option_env!("CAIRN_VERSION").unwrap_or("dev"));
    assert_eq!(BUILD.commit, option_env!("CAIRN_COMMIT"));
}

#[test]
fn ingest_records_each_line_and_names_the_writer() {
    let p = Process::new();
    let path = p.transcript("a\nb\n");

    let (code, out, err) = p.invoke(&["ingest", "--path", &path], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.starts_with("ingested 2 lines into writer "), "{out}");
    assert!(!out.contains(&path), "the output repeats no path: {out}");
}

#[test]
fn a_release_build_records_no_transcript_until_redaction_lands() {
    let p = Process::new();
    let path = p.transcript("a\n");
    let release = Build {
        version: "1.0.0",
        commit: Some("abc"),
    };

    let (code, out, err) = p.invoke(&["ingest", "--path", &path], &release);

    assert_eq!(
        (code, out.as_str(), err.as_str()),
        (
            EXIT_FAILURE,
            "",
            "cairn ingest: a release build records no transcript until redaction before \
             storage (SEC-08) lands; use a development build\n"
        )
    );
    assert!(!p.cairn_home().exists());
}

#[test]
fn ingest_json_names_the_writer_seat_room_and_count() {
    let p = Process::new();
    let path = p.transcript("a\n");

    let (code, out, _) = p.invoke(&["ingest", &format!("--path={path}"), "--json"], &DEV);

    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["events"], 1);
    for field in ["writer", "seat", "room"] {
        assert_eq!(v[field].as_str().unwrap().len(), 64, "{field}");
    }
}

#[test]
fn ingest_refuses_a_wrong_command_line() {
    for (args, want) in [
        (&["ingest"][..], "cairn ingest: want --path P\n"),
        (
            &["ingest", "--path"],
            "cairn ingest: --path needs a transcript\n",
        ),
        (
            &["ingest", "--all"],
            "cairn ingest: --all is not supported yet; pass --path\n",
        ),
        (
            &["ingest", "--yaml"],
            "cairn ingest: unknown flag \"--yaml\"\n",
        ),
    ] {
        let (code, _, err) = invoke(args, &DEV);
        assert_eq!((code, err.as_str()), (EXIT_USAGE, want), "{args:?}");
    }
}

#[test]
fn ingest_reports_a_transcript_it_cannot_read() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["ingest", "--path", "/nonexistent/t.jsonl"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("cairn ingest: read the transcript /nonexistent/t.jsonl: "),
        "{err}"
    );
}

#[test]
fn verify_checks_the_record_and_counts_it() {
    let p = Process::new();
    let path = p.transcript("a\nb\n");
    p.invoke(&["ingest", "--path", &path], &DEV);

    let text = p.invoke(&["verify"], &DEV);
    let json = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!(
        text,
        (
            EXIT_OK,
            "checked 2 writers and 6 events: every chain, commitment and seat certificate holds; \
             integrity unsigned, as no log is signed yet; the audit log's chain holds over 0 \
             entries\n"
                .into(),
            String::new()
        )
    );
    assert_eq!(json.0, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&json.1).unwrap();
    assert_eq!(
        (&v["integrity"], &v["writers"], &v["events"], &v["broken"]),
        (
            &serde_json::json!("unsigned"),
            &serde_json::json!(2),
            &serde_json::json!(6),
            &serde_json::Value::Null
        )
    );
    assert_eq!(v["audit"], serde_json::json!({"files": 0, "entries": 0}));
    let order: Vec<&str> = v["order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    assert_eq!(order.len(), 6);
    assert!(
        order.iter().all(|a| a.starts_with("cairn:room/")),
        "{order:?}"
    );
}

#[test]
fn verify_exits_3_naming_where_the_chain_breaks() {
    let p = Process::new();
    let path = p.transcript("a\nb\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    let conn = rusqlite::Connection::open(p.cairn_home().join("store.db")).unwrap();
    conn.execute(
        "UPDATE events SET provenance = 'user' WHERE kind = 'unparsed_line' AND seq = 2",
        [],
    )
    .unwrap();

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    let (json_code, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_INTEGRITY, ""));
    assert!(
        err.starts_with("cairn verify: chain broken at cairn:room/"),
        "{err}"
    );
    assert!(
        err.ends_with("/2: the stored hash does not match the event's header\n"),
        "{err}"
    );
    assert_eq!(json_code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        format!(
            "cairn verify: {}\n",
            v["broken"]["message"].as_str().unwrap()
        ),
        err
    );
    assert_eq!(
        (&v["integrity"], &v["broken"]["class"], &v["order"]),
        (
            &serde_json::json!("broken"),
            &serde_json::json!("chain"),
            &serde_json::Value::Null
        )
    );
    assert!(
        err.contains(v["broken"]["address"].as_str().unwrap()),
        "{v}"
    );
}

#[test]
fn verify_without_a_store_or_with_a_failing_one_fails() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        err,
        format!(
            "cairn verify: no store at {}\n",
            p.cairn_home().join("store.db").display()
        )
    );

    std::fs::create_dir_all(p.cairn_home()).unwrap();
    junk_store(&p);
    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("cairn verify: the store: "), "{err}");
}

#[test]
fn verify_reports_a_store_error_met_while_checking() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.cairn_home().join("store.db")).unwrap();
    conn.execute("UPDATE seats SET room = 'x'", []).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    let (_, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!(code, EXIT_INTEGRITY);
    assert!(
        err.starts_with("cairn verify: the store is corrupt"),
        "{err}"
    );
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        (&v["broken"]["class"], &v["broken"]["address"]),
        (&serde_json::json!("corrupt"), &serde_json::Value::Null)
    );
}

#[test]
fn verify_reports_a_store_that_fails_while_checking() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.cairn_home().join("store.db")).unwrap();
    conn.execute("DROP TABLE event_heads", []).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("cairn verify: the store: no such table"),
        "{err}"
    );
}

#[test]
fn verify_of_a_home_without_its_device_key_fails() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    std::fs::remove_file(p.cairn_home().join("keys/device.key")).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("device.key"), "{err}");
}

#[test]
fn verify_refuses_an_unknown_flag() {
    assert_eq!(
        invoke(&["verify", "--yaml"], &DEV),
        (
            EXIT_USAGE,
            String::new(),
            "cairn verify: unknown flag \"--yaml\"\n".into()
        )
    );
}

#[test]
fn mcp_answers_each_line_of_input() {
    let mut p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    p.lines = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.into(),
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#.into(),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let lines: Vec<serde_json::Value> = out
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines[0]["result"]["serverInfo"]["version"], "dev");
    assert_eq!(lines[1]["id"], 2);
}

#[test]
fn mcp_refuses_flags_and_reports_a_store_it_cannot_open() {
    for flag in ["--x", "--json"] {
        assert_eq!(
            invoke(&["mcp", flag], &DEV),
            (
                EXIT_USAGE,
                String::new(),
                format!("cairn mcp: unknown flag {flag:?}\n")
            )
        );
    }
    let p = Process::new();
    std::fs::create_dir_all(p.cairn_home()).unwrap();
    junk_store(&p);
    let (code, _, err) = p.invoke(&["mcp"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("cairn mcp: the store: "), "{err}");
}

#[test]
fn mcp_without_a_store_or_a_node_fails_and_makes_nothing() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        err,
        format!(
            "cairn mcp: no store at {}: run cairn ingest first\n",
            p.cairn_home().join("store.db").display()
        )
    );
    assert!(!p.cairn_home().exists());
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    std::fs::remove_file(p.cairn_home().join("keys/device.key")).unwrap();
    let (code, _, err) = p.invoke(&["mcp"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("device.key"), "{err}");
}

#[test]
fn mcp_reports_a_reply_it_cannot_write() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let lines = vec![r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#.to_owned()];
    let input = move || {
        let (tx, rx) = mpsc::channel();
        for line in lines.clone() {
            tx.send(Ok(line)).unwrap();
        }
        rx
    };
    let ctx = Context {
        build: &DEV,
        cairn_home: Some(p.cairn_home().into_os_string()),
        home: None,
        home_id: None,
        hook_panics: false,
        input: &input,
        clock: &|| NOW,
    };
    let mut err = Vec::new();

    let code = run(["mcp".to_owned()], &ctx, &mut Closed, &mut err);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(String::from_utf8(err).unwrap(), "cairn mcp: closed\n");
}

#[test]
fn a_command_without_a_home_fails() {
    let input = || mpsc::channel().1;
    let ctx = Context {
        build: &DEV,
        cairn_home: None,
        home: None,
        home_id: None,
        hook_panics: false,
        input: &input,
        clock: &|| NOW,
    };
    for command in ["ingest", "mcp", "verify", "status", "doctor", "counter"] {
        let mut args = vec![command.to_owned()];
        if command == "ingest" {
            args.extend(["--path".to_owned(), "t".to_owned()]);
        }
        if command == "counter" {
            args.push("ack".to_owned());
        }
        let (mut out, mut err) = (Vec::new(), Vec::new());

        let code = run(args, &ctx, &mut out, &mut err);

        assert_eq!(code, EXIT_FAILURE, "{command}");
        let name = if command == "counter" {
            "counter ack"
        } else {
            command
        };
        assert_eq!(
            String::from_utf8(err).unwrap(),
            format!("cairn {name}: no home: set CAIRN_HOME or HOME\n")
        );
    }
}

/// Writes a store file that is no database, readable by its owner alone
/// so the store reads it rather than refuse its permissions (SEC-02).
fn junk_store(p: &Process) {
    let path = p.cairn_home().join("store.db");
    std::fs::write(&path, "not a database, but long enough to read a header").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

/// The audit log's entries in `p`'s home.
fn audited(p: &Process) -> Vec<audit::entry::Entry> {
    audit::log::Log::new(p.cairn_home().join("audit"))
        .entries()
        .unwrap()
}

const SESSION_START: &str =
    r#"{"session_id":"s","hook_event_name":"SessionStart","source":"startup"}"#;

#[test]
fn a_hook_with_a_valid_hook_input_injects_nothing_and_audits_nothing() {
    let mut p = Process::new();
    p.lines = vec![SESSION_START.to_owned()];

    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert_eq!(audited(&p), Vec::new());
}

#[test]
fn a_hook_input_that_is_no_json_object_is_audited_and_counted() {
    let mut p = Process::new();
    p.lines = vec!["{\"session_id\": ".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(
        (
            e.kind.as_str(),
            e.counter.as_deref(),
            e.operation.as_str(),
            e.detail.as_str(),
            e.time
        ),
        (
            "rejected",
            Some("hook_rejected"),
            "hook UserPromptSubmit",
            "not a JSON object",
            NOW
        )
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode =
            |path: &std::path::Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&p.cairn_home()), 0o700);
        assert_eq!(
            mode(&p.cairn_home().join("audit/audit-000001.jsonl")),
            0o600
        );
    }
}

#[test]
fn a_hook_input_that_cannot_be_read_or_never_ends_is_rejected() {
    let mut p = Process::new();
    p.input_error = true;
    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    p.input_error = false;
    p.held_open = true;
    p.lines = vec![SESSION_START.to_owned()];
    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let details: Vec<String> = audited(&p).into_iter().map(|e| e.detail).collect();
    assert_eq!(details, ["not a JSON object", "not a JSON object"]);
}

#[test]
fn a_hook_cairn_does_not_install_is_rejected_and_exits_0() {
    let mut p = Process::new();
    p.lines = vec![SESSION_START.to_owned()];
    for args in [&["hook", "Stop"][..], &["hook"]] {
        assert_eq!(
            p.invoke(args, &DEV),
            (EXIT_OK, String::new(), String::new()),
            "{args:?}"
        );
    }
    let entries = audited(&p);
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .all(|e| e.operation == "hook" && e.detail == "no such hook"),
        "{entries:?}"
    );
}

#[test]
fn a_hook_handler_that_panics_fails_open_and_is_audited() {
    let mut p = Process::new();
    p.lines = vec![SESSION_START.to_owned()];
    p.hook_panics = true;

    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        (
            entries[0].kind.as_str(),
            entries[0].counter.as_deref(),
            entries[0].detail.as_str()
        ),
        ("failed", Some("hook_panicked"), "panic recovered")
    );
}

#[cfg(unix)]
#[test]
fn a_hook_in_a_home_it_must_refuse_writes_nothing_and_exits_0() {
    use std::os::unix::fs::PermissionsExt;
    let mut p = Process::new();
    std::fs::create_dir(p.cairn_home()).unwrap();
    std::fs::set_permissions(p.cairn_home(), std::fs::Permissions::from_mode(0o755)).unwrap();
    p.lines = vec!["not json".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert!(!p.cairn_home().join("audit").exists());
    p.home_id = Some("other".into());
    std::fs::set_permissions(p.cairn_home(), std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(p.cairn_home().join("home.json"), r#"{"id":"mine"}"#).unwrap();
    std::fs::set_permissions(
        p.cairn_home().join("home.json"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert!(!p.cairn_home().join("audit").exists());
}

#[test]
fn a_hook_whose_audit_log_cannot_take_an_entry_still_exits_0() {
    let mut p = Process::new();
    std::fs::create_dir_all(p.cairn_home().join("audit/audit-000001.jsonl")).unwrap();
    p.lines = vec!["not json".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
}

#[test]
fn counter_ack_appends_its_entry_and_says_so() {
    let p = Process::new();

    assert_eq!(
        p.invoke(&["counter", "ack"], &DEV),
        (EXIT_OK, "counters acknowledged\n".into(), String::new())
    );
    let (code, out, _) = p.invoke(&["counter", "ack", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    assert_eq!(out, "{\"acknowledged\":2}\n");
    let kinds: Vec<String> = audited(&p).into_iter().map(|e| e.kind).collect();
    assert_eq!(kinds, ["counter_ack", "counter_ack"]);
}

#[test]
fn counter_needs_ack_and_refuses_other_flags() {
    for (args, why) in [
        (&["counter"][..], "cairn counter: want ack\n"),
        (&["counter", "reset"], "cairn counter: want ack\n"),
        (
            &["counter", "ack", "--all"],
            "cairn counter ack: unknown flag \"--all\"\n",
        ),
    ] {
        assert_eq!(
            invoke(args, &DEV),
            (EXIT_USAGE, String::new(), why.into()),
            "{args:?}"
        );
    }
}

#[test]
fn counter_ack_reports_an_audit_log_it_cannot_append_to() {
    let p = Process::new();
    std::fs::create_dir_all(p.cairn_home().join("audit/audit-000001.jsonl")).unwrap();

    let (code, _, err) = p.invoke(&["counter", "ack"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("cairn counter ack: "), "{err}");
}

#[test]
fn doctor_fails_while_a_counter_rose_since_the_last_acknowledgement() {
    let mut p = Process::new();
    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_OK,
            format!(
                "no home at {} yet: nothing to diagnose\n",
                p.cairn_home().display()
            ),
            String::new()
        )
    );
    p.invoke(&["counter", "ack"], &DEV);
    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_OK,
            "no counter rose since the counters were last acknowledged\n".into(),
            String::new()
        )
    );
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);

    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_FAILURE,
            "counter hook_rejected rose by 2 since the counters were last acknowledged\n".into(),
            String::new()
        )
    );
    let (code, out, _) = p.invoke(&["doctor", "--json"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(out, "{\"risen\":{\"hook_rejected\":2}}\n");

    p.invoke(&["counter", "ack"], &DEV);
    let (code, out, _) = p.invoke(&["doctor", "--json"], &DEV);
    assert_eq!((code, out.as_str()), (EXIT_OK, "{\"risen\":{}}\n"));
}

#[test]
fn doctor_and_status_exit_3_on_a_broken_audit_log() {
    let p = Process::new();
    p.invoke(&["counter", "ack"], &DEV);
    let file = p.cairn_home().join("audit/audit-000001.jsonl");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, text.replace("acknowledged", "acknowledgeD")).unwrap();

    for command in ["doctor", "status"] {
        let (code, _, err) = p.invoke(&[command], &DEV);
        assert_eq!(code, EXIT_INTEGRITY, "{command}");
        assert_eq!(
            err,
            format!("cairn {command}: audit log audit-000001.jsonl line 1: hash does not match\n")
        );
    }
}

#[test]
fn doctor_and_status_refuse_unknown_flags() {
    for command in ["doctor", "status"] {
        assert_eq!(
            invoke(&[command, "--fix"], &DEV),
            (
                EXIT_USAGE,
                String::new(),
                format!("cairn {command}: unknown flag \"--fix\"\n")
            )
        );
    }
}

#[test]
fn status_shows_the_home_the_store_the_audit_log_and_the_counters() {
    let mut p = Process::new();
    let home = p.cairn_home().display().to_string();
    assert_eq!(
        p.invoke(&["status"], &DEV),
        (EXIT_OK, format!("home: {home} (none yet)\n"), String::new())
    );
    p.invoke(&["counter", "ack"], &DEV);
    assert_eq!(
        p.invoke(&["status"], &DEV).1,
        format!(
            "home: {home}\nstore: {home}/store.db (none yet)\naudit log: {home}/audit (1 entries)\n\
             counters: none\n"
        )
    );
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);

    let (code, out, err) = p.invoke(&["status"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let bytes = std::fs::metadata(p.cairn_home().join("store.db"))
        .unwrap()
        .len();
    assert_eq!(
        out,
        format!(
            "home: {home}\nstore: {home}/store.db ({bytes} bytes)\naudit log: {home}/audit (2 entries)\n\
             counters: hook_rejected 1 (1 since acknowledged)\n"
        )
    );
    let (code, out, _) = p.invoke(&["status", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v,
        serde_json::json!({
            "home": home,
            "store": {"path": format!("{home}/store.db"), "bytes": bytes},
            "audit": {"path": format!("{home}/audit"), "entries": 2},
            "counters": {"hook_rejected": {"total": 1, "since_ack": 1}},
        })
    );
    let fresh = Process::new();
    let (_, out, _) = fresh.invoke(&["status", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v,
        serde_json::json!({"home": fresh.cairn_home().display().to_string(), "store": null, "audit": null, "counters": {}})
    );
}

#[cfg(unix)]
#[test]
fn every_command_refuses_a_home_others_can_reach_naming_it() {
    use std::os::unix::fs::PermissionsExt;
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    std::fs::set_permissions(p.cairn_home(), std::fs::Permissions::from_mode(0o750)).unwrap();
    let path = p.transcript("b\n");
    for args in [
        &["ingest", "--path", path.as_str()][..],
        &["mcp"],
        &["verify"],
        &["status"],
        &["doctor"],
        &["counter", "ack"],
    ] {
        let (code, out, err) = p.invoke(args, &DEV);
        let name = if args[0] == "counter" {
            "counter ack"
        } else {
            args[0]
        };
        assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""), "{args:?}");
        assert_eq!(
            err,
            format!(
                "cairn {name}: {}: mode 0750 lets other OS users in; the home wants owner-only \
                 permissions\n",
                p.cairn_home().display()
            ),
            "{args:?}"
        );
    }
}

#[test]
fn every_command_refuses_a_home_bound_to_another_home_id() {
    let mut p = Process::new();
    p.invoke(&["counter", "ack"], &DEV);
    let binding = p.cairn_home().join("home.json");
    std::fs::write(&binding, r#"{"id":"home-a"}"#).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binding, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    p.home_id = Some("home-b".into());
    for command in ["verify", "status", "doctor", "mcp"] {
        let (code, _, err) = p.invoke(&[command], &DEV);
        assert_eq!(code, EXIT_FAILURE, "{command}");
        assert_eq!(
            err,
            format!(
                "cairn {command}: the home id CAIRN_HOME_ID supplies does not match the home's\n"
            )
        );
    }
    p.home_id = Some("home-a".into());
    assert_eq!(p.invoke(&["doctor"], &DEV).0, EXIT_OK);
}

#[test]
fn verify_checks_the_audit_chain_too() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    p.invoke(&["counter", "ack"], &DEV);
    let (code, out, _) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_OK);
    assert!(
        out.ends_with("the audit log's chain holds over 1 entries\n"),
        "{out}"
    );
    let file = p.cairn_home().join("audit/audit-000001.jsonl");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, text.replace("acknowledged", "acknowledgeD")).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert_eq!(
        err,
        "cairn verify: audit log audit-000001.jsonl line 1: hash does not match\n"
    );
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["broken"],
        serde_json::json!({
            "class": "audit",
            "address": null,
            "message": "audit log audit-000001.jsonl line 1: hash does not match",
        })
    );
}

#[test]
fn a_panic_in_the_mcp_server_is_audited() {
    let p = Process::new();
    let home = Home::locate(Some(p.cairn_home().as_os_str()), None).unwrap();

    mcp_panicked(&home, NOW);

    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        (
            entries[0].counter.as_deref(),
            entries[0].operation.as_str(),
            entries[0].detail.as_str()
        ),
        (Some("mcp_panicked"), "mcp", "panic recovered")
    );
}
