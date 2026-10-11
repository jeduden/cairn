use super::*;
use crate::fixture;
use testkit::TempDir;

/// Runs the command, returning its code, stdout and stderr.
fn invoke(args: &[&str]) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(args.iter().map(|a| (*a).to_owned()), &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// Two copies of the fixture tree, a diff and an output directory.
struct World {
    base: TempDir,
    head: TempDir,
    files: TempDir,
}

impl World {
    fn new(diff: &str) -> Self {
        let w = Self {
            base: TempDir::new().unwrap(),
            head: TempDir::new().unwrap(),
            files: TempDir::new().unwrap(),
        };
        fixture::tree(w.base.path());
        fixture::tree(w.head.path());
        fs::write(w.files.path().join("pr.diff"), diff).unwrap();
        w
    }

    fn path(dir: &TempDir) -> String {
        dir.path().display().to_string()
    }

    fn args(&self) -> Vec<String> {
        vec![
            "--base".into(),
            Self::path(&self.base),
            format!("--head={}", Self::path(&self.head)),
            "--diff".into(),
            self.files.path().join("pr.diff").display().to_string(),
            "--out".into(),
            Self::path(&self.files),
        ]
    }

    fn run(&self) -> (u8, String, String) {
        let args = self.args();
        invoke(&args.iter().map(String::as_str).collect::<Vec<_>>())
    }

    fn output(&self, name: &str) -> String {
        fs::read_to_string(self.files.path().join(name)).unwrap()
    }
}

#[test]
fn run_writes_the_bundle_and_findings_and_prints_the_route() {
    let w = World::new("diff --git a/docs/srs/05.md b/docs/srs/05.md\n+REC-01\n");

    let (code, out, err) = w.run();

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(out, "class=deep\ntouched=1\nfindings=0\n");
    assert!(w.output(BUNDLE).contains("### REC-01"));
    assert_eq!(w.output(CHECKED), "[]\n");
}

#[test]
fn run_writes_the_ratchet_finding() {
    let w = World::new("diff --git a/features/record.feature b/features/record.feature\n");
    let feature = w.head.path().join("features/record.feature");
    let body = fs::read_to_string(&feature).unwrap();
    fs::write(
        &feature,
        body.replace("@REC-01 @P0 @I1", "@REC-01 @P0 @I1 @pending"),
    )
    .unwrap();

    let (code, out, err) = w.run();

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.contains("findings=1"), "{out}");
    let checked: serde_json::Value = serde_json::from_str(&w.output(CHECKED)).unwrap();
    assert_eq!(checked[0]["severity"], "blocking");
    assert_eq!(checked[0]["path"], "features/record.feature");
}

/// A change that breaks one input.
type Breaks<'a> = &'a dyn Fn(&World);

#[test]
fn run_fails_on_a_tree_a_diff_or_an_output_it_cannot_use() {
    let cases: [(&str, Breaks, &str); 4] = [
        (
            "main's tree",
            &|w| fs::remove_dir_all(w.base.path().join("docs/srs")).unwrap(),
            "(main's tree)",
        ),
        (
            "the pull request's tree",
            &|w| fs::remove_dir_all(w.head.path().join("docs/srs")).unwrap(),
            "(the pull request's tree)",
        ),
        (
            "the diff",
            &|w| fs::remove_file(w.files.path().join("pr.diff")).unwrap(),
            "pr.diff",
        ),
        (
            "the output",
            &|w| fs::create_dir(w.files.path().join(BUNDLE)).unwrap(),
            "write",
        ),
    ];
    for (name, change, want) in cases {
        let w = World::new("");
        change(&w);
        let (code, out, err) = w.run();
        assert_eq!(code, EXIT_FAILURE, "{name}");
        assert!(out.is_empty(), "{name}");
        assert!(err.contains(want), "{name}: {err}");
    }
}

#[test]
fn run_refuses_a_wrong_command_line() {
    for (args, want) in [
        (vec!["--base"], "--base needs a value"),
        (vec!["--nope", "x"], r#"unknown flag "--nope""#),
        (vec!["stray"], "unexpected argument"),
        (
            vec!["--base", "a", "--head", "b", "--diff", "c"],
            "--out is required",
        ),
        (vec!["--base="], "--base needs a value"),
    ] {
        let (code, out, err) = invoke(&args);
        assert_eq!(code, EXIT_USAGE, "{args:?}");
        assert!(out.is_empty());
        assert!(
            err.contains(want) && err.contains("usage:"),
            "{args:?}: {err}"
        );
    }
}
