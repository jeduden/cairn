use super::*;
use crate::tests::checkout;

fn package(name: &str, license: Option<&str>) -> Package {
    Package {
        name: name.into(),
        version: "1.0.0".into(),
        license: license.map(str::to_owned),
        dir: PathBuf::from(format!("/registry/{name}-1.0.0")),
    }
}

fn file(name: &str, text: &str) -> (String, String) {
    (name.to_owned(), text.to_owned())
}

fn mit_file() -> (String, String) {
    file(
        "LICENSE",
        &format!(
            "MIT License\n\nCopyright (c) 2024 Someone\n\n{}",
            CANON[5].1
        ),
    )
}

#[test]
fn every_canonical_text_fits_itself_and_keeps_its_id() {
    for (id, text) in CANON {
        assert!(fits(text, text), "{id}");
        assert_eq!(canonical(id), Some(text));
    }
    assert_eq!(canonical("GPL-3.0-only"), None);
}

#[test]
fn normalize_ignores_case_whitespace_and_copyright_lines() {
    assert_eq!(
        normalize(
            "Copyright (c) 2020 A\n© 2021 B\n(C) 2022 C\nCopyright 2023 D\nHello\n  World \n"
        ),
        "hello world"
    );
    assert_eq!(
        normalize("the\ncopyright\nholders"),
        "the copyright holders"
    );
}

#[test]
fn fits_lets_a_template_fill_in_what_it_leaves_free() {
    let bsd3 = canonical("BSD-3-Clause").unwrap();
    let text = bsd3.replace(FREE, "the University of Somewhere");

    assert!(fits(&text, bsd3));
    assert!(fits(
        &format!("Copyright (c) 2020 X\n\n{text}\nAn extra notice."),
        bsd3
    ));
    assert!(!fits(
        &text.replace("nor the names of its", "or anyone"),
        bsd3
    ));
    assert!(!fits("Permission granted.", canonical("MIT").unwrap()));
}

#[test]
fn the_apache_appendix_is_optional() {
    let apache = canonical("Apache-2.0").unwrap();
    let with = format!("{apache}\nAPPENDIX: How to apply the Apache License to your work.\n");

    assert!(fits(apache, apache));
    assert!(fits(&with, apache));
}

#[test]
fn choices_group_and_split_an_expression() {
    let want = |g: &[&[&str]]| -> Vec<Vec<String>> {
        g.iter()
            .map(|c| c.iter().map(|s| (*s).to_owned()).collect())
            .collect()
    };

    assert_eq!(
        choices("MIT OR Apache-2.0"),
        want(&[&["MIT", "Apache-2.0"]])
    );
    assert_eq!(choices("MIT/Apache-2.0"), want(&[&["MIT", "Apache-2.0"]]));
    assert_eq!(
        choices("(MIT OR Apache-2.0) AND Unicode-3.0"),
        want(&[&["MIT", "Apache-2.0"], &["Unicode-3.0"]])
    );
    assert_eq!(
        choices("Apache-2.0 WITH LLVM-exception OR MIT"),
        want(&[&["Apache-2.0", "MIT"]])
    );
}

#[test]
fn license_file_names_cover_the_usual_spellings() {
    for name in [
        "LICENSE",
        "LICENSE-MIT",
        "license-apache-2.0",
        "COPYING",
        "UNLICENSE",
        "LICENCE.md",
    ] {
        assert!(license_file_name(name), "{name}");
    }
    assert!(!license_file_name("README.md"));
}

#[test]
fn a_crate_with_a_fitting_license_file_passes() {
    assert!(check_crate(&package("a", Some("MIT")), &[mit_file()]).is_empty());
    assert!(
        check_crate(
            &package("a", Some("Unlicense OR MIT")),
            &[file("UNLICENSE", "free"), mit_file()]
        )
        .is_empty()
    );
}

#[test]
fn a_crate_with_no_license_file_fails_naming_the_crate_and_version() {
    assert_eq!(
        check_crate(&package("a", Some("MIT")), &[]),
        ["a 1.0.0 ships no license file"]
    );
}

#[test]
fn a_crate_declaring_no_license_fails() {
    assert_eq!(
        check_crate(&package("a", None), &[mit_file()]),
        ["a 1.0.0 declares no license"]
    );
}

#[test]
fn a_license_file_that_does_not_match_the_declared_license_fails() {
    let problems = check_crate(&package("a", Some("Apache-2.0")), &[mit_file()]);

    assert_eq!(
        problems,
        ["a 1.0.0 declares Apache-2.0, but no license file fits Apache-2.0"]
    );
}

#[test]
fn a_part_joined_by_and_needs_its_own_file() {
    let both = package("a", Some("MIT AND Unicode-3.0"));

    assert_eq!(
        check_crate(&both, &[mit_file()]),
        ["a 1.0.0 declares MIT AND Unicode-3.0, but no license file fits Unicode-3.0"]
    );
    assert!(
        check_crate(
            &both,
            &[
                mit_file(),
                file("LICENSE-UNICODE", canonical("Unicode-3.0").unwrap())
            ]
        )
        .is_empty()
    );
}

#[test]
fn a_license_with_no_canonical_text_cannot_be_checked() {
    assert_eq!(
        check_crate(&package("a", Some("WTFPL")), &[mit_file()]),
        ["a 1.0.0 declares WTFPL, none of WTFPL has a canonical text"]
    );
}

#[test]
fn packages_lists_registry_crates_by_name_and_leaves_the_workspace_out() {
    let json = r#"{"packages":[
        {"name":"zed","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/zed/Cargo.toml"},
        {"name":"own","version":"0.0.0","source":null,"license":"MIT","manifest_path":"/w/own/Cargo.toml"},
        {"name":"alpha","version":"2.0.0","source":"registry+x","manifest_path":"/r/alpha/Cargo.toml"}
    ]}"#;
    let found = packages(json).unwrap();

    assert_eq!(
        found.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["alpha", "zed"]
    );
    assert_eq!(found[0].license, None);
    assert_eq!(found[1].dir, PathBuf::from("/r/zed"));
    assert!(
        packages("nope")
            .unwrap_err()
            .starts_with("read cargo metadata: ")
    );
}

#[test]
fn license_files_reads_only_license_named_files_in_name_order() {
    let (dir, _) = checkout(&[
        ("LICENSE-MIT", "m"),
        ("LICENSE-APACHE", "a"),
        ("README.md", "r"),
    ]);
    fs::write(dir.path().join("COPYING"), [0xff, 0xfe]).unwrap();
    fs::create_dir(dir.path().join("LICENSES")).unwrap();

    let names: Vec<String> = license_files(dir.path())
        .unwrap()
        .into_iter()
        .map(|f| f.0)
        .collect();

    assert_eq!(names, ["COPYING", "LICENSE-APACHE", "LICENSE-MIT"]);
    assert!(
        license_files(&dir.path().join("missing"))
            .unwrap_err()
            .starts_with("read ")
    );
}

#[test]
fn check_tree_reports_every_crate_that_fails_and_every_dir_it_cannot_read() {
    let json = r#"{"packages":[
        {"name":"a","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/a/Cargo.toml"},
        {"name":"b","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/b/Cargo.toml"},
        {"name":"c","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/c/Cargo.toml"}
    ]}"#;
    let problems = check_tree(json, |dir| match dir.to_str().unwrap() {
        "/r/a" => Ok(vec![mit_file()]),
        "/r/b" => Ok(vec![]),
        _ => Err("read /r/c: gone".into()),
    })
    .unwrap();

    assert_eq!(
        problems,
        ["b 1.0.0 ships no license file", "read /r/c: gone"]
    );
    assert!(check_tree("nope", |_| Ok(vec![])).is_err());
}

#[test]
fn canon_covers_every_license_deny_toml_allows_or_grants() {
    let deny =
        "allow = [\"MIT\", \"ISC\"]\nexceptions = [\n  { crate = \"x\", allow = [\"NCSA\"] },\n]\n";
    assert!(canon_covers(deny).is_empty());

    let uncovered =
        "allow = [\"MIT\"]\nexceptions = [\n  { crate = \"x\", allow = [\"WTFPL\"] },\n]\n";
    assert_eq!(
        canon_covers(uncovered),
        ["deny.toml grants WTFPL, which the license tool keeps no text for"]
    );
    assert!(canon_covers("").is_empty());
}

#[test]
fn kept_texts_flags_a_license_file_the_repository_keeps_for_a_crate() {
    let (_dir, c) = checkout(&[
        (
            "licenses/x86_64-unknown-linux-musl/THIRD_PARTY_LICENSES",
            "all",
        ),
        ("third_party/serde-1.0.0/LICENSE-MIT", "m"),
        ("vendor/syn/COPYING", "c"),
        ("vendor/syn/README.md", "r"),
        ("vendor/other/LICENSE", "o"),
    ]);
    let found = kept_texts(&c, &[package("serde", None), package("syn", None)]);

    assert_eq!(
        found,
        [
            "the repository keeps a license text at third_party/serde-1.0.0/LICENSE-MIT",
            "the repository keeps a license text at vendor/syn/COPYING",
        ]
    );
    let (_dir, none) = checkout(&[("README.md", "x")]);
    assert!(kept_texts(&none, &[package("serde", None)]).is_empty());
}
