use super::*;
use std::path::PathBuf;

pub(crate) fn dep_adr(
    id: &str,
    status: &str,
    alternatives: &str,
    modules: Vec<adr::Module>,
) -> Adr {
    Adr {
        path: PathBuf::from(format!("docs/adr/{id}-x.md")),
        id: id.into(),
        title: "t".into(),
        status: status.into(),
        summary: "s".into(),
        sections: [("Alternatives".to_owned(), alternatives.to_owned())].into(),
        modules,
        ..Adr::default()
    }
}

fn module(name: &str, license: &str) -> adr::Module {
    adr::Module {
        name: name.into(),
        purpose: "p".into(),
        license: license.into(),
        maintenance: "m".into(),
    }
}

fn strings(s: &[&str]) -> Vec<String> {
    s.iter().map(|x| (*x).to_owned()).collect()
}

#[test]
fn go_direct_deps_skips_indirect_and_reads_both_forms() {
    let go_mod = "module x

go 1.26.0

require example.com/one v1.0.0

require (
\texample.com/two v1.0.0
\texample.com/three v1.0.0 // indirect
)

require (
\texample.com/four v1.0.0
)
";

    assert_eq!(
        go_direct_deps(go_mod),
        ["example.com/one", "example.com/two", "example.com/four"]
    );
}

#[test]
fn deps_named_once_and_modules_are_deps() {
    let deps = strings(&["a", "b", "c"]);
    let adrs = vec![
        dep_adr(
            "ADR-01",
            adr::ACCEPTED,
            "x",
            vec![module("a", "MIT"), module("stale", "MIT")],
        ),
        dep_adr("ADR-02", adr::ACCEPTED, "x", vec![module("b", "MIT")]),
        dep_adr("ADR-03", adr::ACCEPTED, "x", vec![module("b", "MIT")]),
        dep_adr("ADR-04", adr::SUPERSEDED, "x", vec![module("c", "MIT")]),
    ];

    assert_eq!(
        deps_named_once(&deps, &adrs),
        [
            "b is named by 2 accepted ADRs [ADR-02 ADR-03], want exactly one",
            "c is named by 0 accepted ADRs [], want exactly one",
        ]
    );
    assert_eq!(
        modules_are_deps(&deps, &adrs),
        ["ADR-01 names stale, which is not a direct dependency"]
    );

    assert!(deps_named_once(&strings(&["b"]), &adrs[1..2]).is_empty());
    assert!(modules_are_deps(&strings(&["b"]), &adrs[1..2]).is_empty());
}

#[test]
fn deps_justified_wants_every_column_and_alternatives() {
    let partial = adr::Module {
        name: "a".into(),
        license: "MIT".into(),
        maintenance: "m".into(),
        ..Default::default()
    };
    let adrs = vec![
        dep_adr("ADR-01", adr::ACCEPTED, "", vec![partial]),
        dep_adr("ADR-02", adr::ACCEPTED, "", vec![]),
        dep_adr(
            "ADR-03",
            adr::PROPOSED,
            "",
            vec![adr::Module {
                name: "b".into(),
                ..Default::default()
            }],
        ),
        Adr {
            sections: BTreeMap::new(),
            ..dep_adr("ADR-05", adr::ACCEPTED, "", vec![module("c", "MIT")])
        },
    ];

    assert_eq!(
        deps_justified(&adrs),
        [
            "ADR-01 weighs no alternatives",
            r#"ADR-01 leaves "Purpose" empty for a"#,
            "ADR-05 weighs no alternatives",
        ]
    );
    assert!(
        deps_justified(&[dep_adr(
            "ADR-04",
            adr::ACCEPTED,
            "x",
            vec![module("a", "MIT")]
        )])
        .is_empty()
    );
}

#[test]
fn licenses_and_target_come_from_the_requirement_text() {
    let eng18 =
        "Licenses MUST be on an allow-list (MIT, BSD). The target is ≤ 2 direct dependencies.";
    let allowed = allow_list(eng18).unwrap();
    assert_eq!(allowed, ["MIT", "BSD"]);
    assert_eq!(dependency_target(eng18).unwrap(), 2);

    let ok = [dep_adr(
        "ADR-01",
        adr::ACCEPTED,
        "x",
        vec![module("a", "MIT"), module("b", "BSD-3-Clause")],
    )];
    assert!(licenses_allowed(&ok, &allowed).is_empty());

    let bad = [dep_adr(
        "ADR-01",
        adr::ACCEPTED,
        "x",
        vec![module("a", "GPL-3.0"), module("b", "BSDish")],
    )];
    assert_eq!(
        licenses_allowed(&bad, &allowed),
        [
            r#"a is licensed "GPL-3.0", not on the allow-list [MIT BSD]"#,
            r#"b is licensed "BSDish", not on the allow-list [MIT BSD]"#,
        ]
    );

    assert!(within_target(&strings(&["a", "b"]), 2).is_ok());
    assert_eq!(
        within_target(&strings(&["a", "b", "c"]), 2).unwrap_err(),
        "3 direct dependencies, want at most 2: [a b c]"
    );
}

#[test]
fn requirement_text_that_states_nothing_is_reported() {
    assert_eq!(
        allow_list("Nothing said here").unwrap_err(),
        "ENG-18 does not state its license allow-list"
    );
    assert_eq!(
        allow_list("an allow-list (MIT").unwrap_err(),
        "ENG-18 does not state its license allow-list"
    );
    assert_eq!(
        dependency_target("Nothing said here").unwrap_err(),
        "ENG-18 does not state its dependency target"
    );
    assert_eq!(
        dependency_target("≤ 5 crates, ≤ x direct dependencies").unwrap_err(),
        "ENG-18 does not state its dependency target"
    );
    assert_eq!(
        dependency_target("≤ 5 crates and ≤ 7 direct dependencies").unwrap(),
        7
    );
}

#[test]
fn license_family_admits_only_its_permissive_variants() {
    let allowed = strings(&["Apache-2.0", "MIT", "BSD", "ISC"]);

    for license in ["BSD-2-Clause", "BSD-3-Clause", "MIT"] {
        assert!(license_allowed(license, &allowed), "{license}");
    }
    for license in [
        "BSD",
        "BSD-4-Clause",
        "BSD-Protection",
        "MIT-advertising",
        "Apache-2.0-foo",
    ] {
        assert!(!license_allowed(license, &allowed), "{license}");
    }
}

#[test]
fn lists_dependency_adrs_wants_a_link_for_each_record_naming_a_module() {
    let body = "- [ADR-01](docs/adr/ADR-01-x.md)\n";
    let mut adrs = vec![
        dep_adr("ADR-01", adr::ACCEPTED, "x", vec![module("a", "MIT")]),
        dep_adr("ADR-02", adr::ACCEPTED, "x", vec![]),
    ];
    assert!(lists_dependency_adrs(body, "DEPENDENCIES.md", "docs/adr", &adrs).is_empty());

    adrs.push(dep_adr(
        "ADR-03",
        adr::SUPERSEDED,
        "x",
        vec![module("b", "MIT")],
    ));
    assert_eq!(
        lists_dependency_adrs(body, "DEPENDENCIES.md", "docs/adr", &adrs),
        ["DEPENDENCIES.md does not link docs/adr/ADR-03-x.md"]
    );
}

#[test]
fn cargo_direct_deps_reads_every_registry_dependency_of_the_members() {
    let metadata = r#"{"packages": [
        {"name": "a", "dependencies": [
            {"name": "serde", "source": "registry+https://github.com/rust-lang/crates.io-index", "kind": null},
            {"name": "b", "source": null, "path": "/w/b", "kind": null},
            {"name": "cucumber", "source": "registry+https://github.com/rust-lang/crates.io-index", "kind": "dev"}
        ]},
        {"name": "b", "dependencies": [
            {"name": "serde", "source": "registry+https://github.com/rust-lang/crates.io-index", "kind": null}
        ]}
    ]}"#;

    assert_eq!(cargo_direct_deps(metadata).unwrap(), ["cucumber", "serde"]);
    assert!(
        cargo_direct_deps("not json")
            .unwrap_err()
            .starts_with("read cargo metadata: ")
    );
    assert_eq!(cargo_direct_deps("{}").unwrap(), Vec::<String>::new());
}

#[test]
fn an_spdx_or_expression_is_allowed_when_one_choice_is() {
    let allowed = strings(&["Apache-2.0", "MIT", "BSD", "ISC"]);

    assert!(license_allowed("MIT OR Apache-2.0", &allowed));
    assert!(license_allowed("GPL-3.0 OR BSD-3-Clause", &allowed));
    assert!(!license_allowed("GPL-3.0 OR LGPL-2.1", &allowed));
    assert!(!license_allowed("MIT AND GPL-3.0", &allowed));
}

fn cargo() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

#[test]
fn cargo_metadata_reads_a_workspace_and_names_a_failure() {
    let (dir, _) = crate::tests::checkout(&[
        (
            "Cargo.toml",
            "[package]\nname = \"t\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        ),
        ("src/lib.rs", ""),
    ]);
    let home = dir.path().join(".cairn");

    let json = cargo_metadata(&cargo(), &dir.path().join("Cargo.toml"), &home).unwrap();
    assert!(json.contains("\"name\":\"t\""), "{json}");

    let err = cargo_metadata(&cargo(), &dir.path().join("missing/Cargo.toml"), &home).unwrap_err();
    assert!(err.starts_with("cargo metadata failed: "), "{err}");
    let err = cargo_metadata(
        std::ffi::OsStr::new("/nonexistent/cargo"),
        &dir.path().join("Cargo.toml"),
        &home,
    );
    assert!(err.unwrap_err().starts_with("run cargo metadata: "));
}

#[test]
fn direct_deps_reads_each_manifest_with_its_reader() {
    let (_dir, checkout) = crate::tests::checkout(&[("go.mod", "require example.com/a v1.0.0\n")]);
    let metadata = |manifest: &std::path::Path| {
        assert!(manifest.ends_with("Cargo.toml"));
        Ok(
            r#"{"packages": [{"dependencies": [{"name": "serde", "source": "registry+x"}]}]}"#
                .to_owned(),
        )
    };

    assert_eq!(
        direct_deps(&checkout, "go.mod", metadata).unwrap(),
        ["example.com/a"]
    );
    assert_eq!(
        direct_deps(&checkout, "Cargo.toml", metadata).unwrap(),
        ["serde"]
    );
    assert_eq!(
        direct_deps(&checkout, "package.json", metadata).unwrap_err(),
        "no reader for the dependencies in package.json"
    );
    assert!(
        direct_deps(&checkout, "Cargo.toml", |_| Err("boom".to_owned())).unwrap_err() == "boom"
    );
}
