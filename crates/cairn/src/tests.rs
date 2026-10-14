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
}

impl Process {
    fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
            lines: Vec::new(),
        }
    }

    fn cairn_home(&self) -> std::path::PathBuf {
        self.dir.path().join(".cairn")
    }

    fn invoke(&self, args: &[&str], build: &Build) -> (u8, String, String) {
        let lines = self.lines.clone();
        let input = move || {
            let (tx, rx) = mpsc::channel();
            for line in lines.clone() {
                tx.send(Ok(line)).unwrap();
            }
            rx
        };
        let ctx = Context {
            build,
            cairn_home: None,
            home: Some(self.dir.path().as_os_str().to_owned()),
            input: &input,
        };
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(args.iter().map(|a| (*a).to_owned()), &ctx, &mut out, &mut err);

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
    ] {
        let mut err = Vec::new();
        let input = || mpsc::channel().1;
        let ctx = Context {
            build: &DEV,
            cairn_home: Some(p.cairn_home().into_os_string()),
            home: None,
            input: &input,
        };

        let code = run(args.iter().map(|a| (*a).to_owned()), &ctx, &mut Closed, &mut err);

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
    assert!(out.starts_with(&format!("ingested 2 lines of {path} into writer ")), "{out}");
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
        (&["ingest", "--path"], "cairn ingest: --path needs a transcript\n"),
        (&["ingest", "--all"], "cairn ingest: --all is not supported yet; pass --path\n"),
        (&["ingest", "--yaml"], "cairn ingest: unknown flag \"--yaml\"\n"),
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
    assert!(err.starts_with("cairn ingest: read the transcript /nonexistent/t.jsonl: "), "{err}");
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
        (EXIT_OK, "verified 2 writers and 6 events: every chain and seat certificate checks\n".into(), String::new())
    );
    assert_eq!(json.0, EXIT_OK);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json.1).unwrap(),
        serde_json::json!({"writers": 2, "events": 6, "broken": null})
    );
}

#[test]
fn verify_exits_3_naming_where_the_chain_breaks() {
    let p = Process::new();
    let path = p.transcript("a\nb\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    let conn = rusqlite::Connection::open(p.cairn_home().join("store.db")).unwrap();
    conn.execute("UPDATE events SET provenance = 'user' WHERE kind = 'unparsed_line' AND seq = 2", []).unwrap();

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    let (json_code, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_INTEGRITY, ""));
    assert!(err.starts_with("cairn verify: chain broken at cairn:room/"), "{err}");
    assert!(err.ends_with("/2: the stored hash does not match the event's header\n"), "{err}");
    assert_eq!(json_code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("cairn verify: {}\n", v["broken"].as_str().unwrap()), err);
}

#[test]
fn verify_without_a_store_or_with_a_failing_one_fails() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(err, format!("cairn verify: no store at {}\n", p.cairn_home().join("store.db").display()));

    std::fs::create_dir_all(p.cairn_home()).unwrap();
    std::fs::write(p.cairn_home().join("store.db"), "not a database, but long enough to read a header").unwrap();
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

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("cairn verify: the store is corrupt"), "{err}");
}

#[test]
fn verify_refuses_an_unknown_flag() {
    assert_eq!(
        invoke(&["verify", "--yaml"], &DEV),
        (EXIT_USAGE, String::new(), "cairn verify: unknown flag \"--yaml\"\n".into())
    );
}

#[test]
fn mcp_answers_each_line_of_input() {
    let mut p = Process::new();
    p.lines = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.into(),
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#.into(),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let lines: Vec<serde_json::Value> = out.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines[0]["result"]["serverInfo"]["version"], "dev");
    assert_eq!(lines[1]["id"], 2);
}

#[test]
fn mcp_refuses_flags_and_reports_a_store_it_cannot_open() {
    assert_eq!(
        invoke(&["mcp", "--x"], &DEV),
        (EXIT_USAGE, String::new(), "cairn mcp: unknown flag \"--x\"\n".into())
    );
    let p = Process::new();
    std::fs::create_dir_all(p.cairn_home()).unwrap();
    std::fs::write(p.cairn_home().join("store.db"), "not a database, but long enough to read a header").unwrap();
    let (code, _, err) = p.invoke(&["mcp"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("cairn mcp: the store: "), "{err}");
}

#[test]
fn mcp_reports_a_reply_it_cannot_write() {
    let p = Process::new();
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
        input: &input,
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
        input: &input,
    };
    for command in ["ingest", "mcp", "verify"] {
        let mut args = vec![command.to_owned()];
        if command == "ingest" {
            args.extend(["--path".to_owned(), "t".to_owned()]);
        }
        let (mut out, mut err) = (Vec::new(), Vec::new());

        let code = run(args, &ctx, &mut out, &mut err);

        assert_eq!(code, EXIT_FAILURE, "{command}");
        assert_eq!(String::from_utf8(err).unwrap(), format!("cairn {command}: no home: set CAIRN_HOME or HOME\n"));
    }
}
