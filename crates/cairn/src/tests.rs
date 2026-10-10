use super::*;
use std::io;

const DEV: Build<'static> = Build {
    version: "dev",
    commit: None,
};

fn invoke(args: &[&str], build: &Build) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(
        args.iter().map(|a| (*a).to_owned()),
        build,
        &mut out,
        &mut err,
    );

    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
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
    let (code, _, err) = invoke(&["ingest"], &DEV);

    assert_eq!(code, EXIT_USAGE);
    assert!(
        err.starts_with("cairn: unknown command \"ingest\"\n\nusage: cairn"),
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
    for args in [&["version"][..], &["version", "--json"], &["help"]] {
        let mut err = Vec::new();

        let code = run(
            args.iter().map(|a| (*a).to_owned()),
            &DEV,
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
