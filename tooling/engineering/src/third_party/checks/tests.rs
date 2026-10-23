use super::*;
use crate::third_party::{Entry, non_crate, render};

fn package(name: &str, version: &str) -> Package {
    Package {
        name: name.into(),
        version: version.into(),
        license: Some("MIT".into()),
        dir: Path::new("/r").join(name),
    }
}

fn files(holder: &str) -> Vec<(String, String)> {
    vec![(
        "LICENSE".into(),
        format!(
            "Copyright (c) 2024 {holder}\n\n{}",
            license_texts::canonical("MIT").unwrap()
        ),
    )]
}

fn rendered(packages: &[Package]) -> String {
    let entries: Vec<Entry> = packages
        .iter()
        .map(|p| crate_entry(p, &files(&p.name)))
        .collect();

    render(&entries)
}

#[test]
fn host_target_is_the_release_target_on_the_hosts_os_and_architecture() {
    assert_eq!(
        host_target("linux", "x86_64"),
        Some("x86_64-unknown-linux-musl")
    );
    assert_eq!(
        host_target("linux", "aarch64"),
        Some("aarch64-unknown-linux-musl")
    );
    assert_eq!(
        host_target("windows", "x86_64"),
        Some("x86_64-pc-windows-gnu")
    );
    assert_eq!(
        host_target("macos", "aarch64"),
        Some("aarch64-apple-darwin")
    );
    assert_eq!(host_target("macos", "x86_64"), Some("aarch64-apple-darwin"));
    assert_eq!(host_target("freebsd", "x86_64"), None);
}

#[test]
fn a_file_holding_every_crate_passes_and_one_missing_a_part_is_named() {
    let linked = [package("a", "1.0.0"), package("b", "2.0.0")];
    let read = |dir: &Path| Ok(files(dir.file_name().unwrap().to_str().unwrap()));
    let whole = rendered(&linked);

    assert!(holds_every_crate(&whole, &linked, read).is_empty());

    let no_b = whole.replace("b 2.0.0 (", "x 2.0.0 (");
    assert_eq!(
        holds_every_crate(&no_b, &linked, read),
        ["b 2.0.0 is missing"]
    );
    let no_line = whole.replace("Copyright (c) 2024 a", "Copyright (c) 2024 z");
    assert_eq!(
        holds_every_crate(&no_line, &linked, read),
        ["a 1.0.0: the copyright line \"Copyright (c) 2024 a\" is missing"]
    );
    let no_text = whole.replace("Permission is hereby granted", "Permission denied");
    assert_eq!(
        holds_every_crate(&no_text, &linked, read),
        [
            "a 1.0.0: the text of LICENSE is missing",
            "b 2.0.0: the text of LICENSE is missing"
        ]
    );
    let unreadable = |_: &Path| Err("read a: gone".to_owned());
    assert_eq!(holds_every_crate(&whole, &linked, unreadable).len(), 2);
}

#[test]
fn a_crate_the_target_does_not_link_must_not_appear() {
    let metadata = r#"{"packages":[
      {"name":"a","version":"1.0.0","source":"registry+x","manifest_path":"/r/a/Cargo.toml"},
      {"name":"dev","version":"1.0.0","source":"registry+x","manifest_path":"/r/dev/Cargo.toml"}]}"#;
    let linked = [package("a", "1.0.0")];

    assert!(
        unlinked_present(&rendered(&linked), metadata, &linked)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        unlinked_present(
            &rendered(&[package("a", "1.0.0"), package("dev", "1.0.0")]),
            metadata,
            &linked
        )
        .unwrap(),
        ["dev 1.0.0 is not linked into the executable"]
    );
    assert!(unlinked_present("", "nope", &linked).is_err());
}

#[test]
fn a_file_must_name_what_its_target_compiles_in() {
    let musl = render(&non_crate("x86_64-unknown-linux-musl"));

    assert!(names_compiled_in("x86_64-unknown-linux-musl", &musl).is_empty());
    assert_eq!(
        names_compiled_in("x86_64-unknown-linux-musl", "SQLite, Rust standard library"),
        ["x86_64-unknown-linux-musl: musl C library is not named"]
    );
}

#[test]
fn files_differ_exactly_when_the_linked_crates_do() {
    let linked: BTreeMap<String, Vec<Package>> = [
        ("a".to_owned(), vec![package("x", "1.0.0")]),
        ("b".to_owned(), vec![package("x", "1.0.0")]),
        (
            "c".to_owned(),
            vec![package("x", "1.0.0"), package("y", "1.0.0")],
        ),
    ]
    .into();
    let files = |a: &str, b: &str, c: &str| -> BTreeMap<String, String> {
        [("a", a), ("b", b), ("c", c)]
            .map(|(t, f)| (t.to_owned(), f.to_owned()))
            .into()
    };

    assert!(files_follow_crates(&linked, &files("one", "one", "two")).is_empty());
    assert_eq!(
        files_follow_crates(&linked, &files("one", "two", "three")),
        ["a and b link the same crates but their files differ in bytes"]
    );
    assert_eq!(
        files_follow_crates(&linked, &files("one", "one", "one")),
        [
            "a and c link different crates but their files do not differ in bytes",
            "b and c link different crates but their files do not differ in bytes"
        ]
    );
}

#[test]
fn the_core_must_include_and_name_each_targets_file() {
    let targets = vec!["a".to_owned(), "b".to_owned()];
    let line = |t: &str| {
        format!(
            "include_str!(\"{0}\") const FILE: &str = \"{0}\";\n",
            committed_path(t)
        )
    };
    let both = format!("{}{}", line("a"), line("b"));

    assert!(embedding_selects(&both, &targets).is_empty());
    assert_eq!(
        embedding_selects(&line("a"), &targets),
        ["the core does not select and name licenses/b/THIRD_PARTY_LICENSES"]
    );
}

#[test]
fn only_the_core_executable_may_ship() {
    let json = |bin: &str| {
        format!(
            r#"{{"workspace_root":"/w","packages":[
              {{"manifest_path":"/w/crates/parinn/Cargo.toml","targets":[{{"name":"parinn","kind":["bin"]}},{{"name":"parinn","kind":["lib"]}}]}},
              {{"manifest_path":"/w/crates/other/Cargo.toml","targets":[{{"name":"{bin}","kind":["bin"]}}]}},
              {{"manifest_path":"/w/tooling/t/Cargo.toml","targets":[{{"name":"tool","kind":["bin"]}}]}}]}}"#
        )
    };

    assert_eq!(
        other_executables(&json("extra")).unwrap(),
        ["the executable extra ships without the texts of its crates"]
    );
    assert!(other_executables(&json("parinn")).unwrap().is_empty());
    assert!(other_executables("nope").is_err());
}

#[test]
fn the_release_build_regenerates_and_compares_on_every_platform() {
    let ok = "run: cargo run --locked -p engineering --bin license-check -- --out regenerated\n\
              run: scripts/check-licenses.sh a\nrun: scripts/check-licenses.sh b\nrun: scripts/check-licenses.sh c\n";

    assert!(release_compares_output(ok).is_empty());
    assert_eq!(
        release_compares_output(&ok.replace("--out regenerated", "--write")),
        ["the build workflow does not regenerate the license files with license-check --out"]
    );
    assert_eq!(
        release_compares_output(&ok.replace("scripts/check-licenses.sh c", "true")),
        ["the build workflow does not compare every platform's licenses output"]
    );
}
