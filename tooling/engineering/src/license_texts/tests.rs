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
        normalize("Copyright (c) 2020 A\n© 2021 B\nCopyright 2023 D\nHello\n  World \n"),
        "hello world"
    );
    assert_eq!(
        normalize("the\ncopyright\nholders"),
        "the copyright holders"
    );
    assert_eq!(normalize("(c) You must retain"), "(c) you must retain");
}

#[test]
fn fits_lets_a_template_fill_in_what_it_leaves_free() {
    let bsd3 = canonical("BSD-3-Clause").unwrap();
    let text = bsd3.replace(FREE, "the University of Somewhere");

    assert!(fits(&text, bsd3));
    assert!(fits(
        &format!("BSD 3-Clause License\n\nCopyright (c) 2020 X\n\n{text}"),
        bsd3
    ));
    assert!(!fits(
        &text.replace("nor the names of its", "or anyone"),
        bsd3
    ));
    assert!(!fits("Permission granted.", canonical("MIT").unwrap()));
}

#[test]
fn fits_refuses_a_term_added_after_the_text_or_a_long_preamble() {
    let mit = canonical("MIT").unwrap();

    assert!(!fits(
        &format!("{mit}\nYou may not use this commercially."),
        mit
    ));
    assert!(!fits(
        &format!("Use is limited to non commercial work, as agreed by all parties here.\n{mit}"),
        mit
    ));
}

#[test]
fn a_clause_starting_c_is_license_text_not_a_copyright_line() {
    let apache = canonical("Apache-2.0").unwrap();
    let clause = "(c) You must retain, in the Source form";
    let altered = apache.replace(clause, "(c) You may drop, in the Source form");

    assert!(apache.contains(clause));
    assert!(!fits(&altered, apache));
}

#[test]
fn the_apache_appendix_is_optional_and_nothing_else_may_follow() {
    let apache = canonical("Apache-2.0").unwrap();
    let with = format!("{apache}\n{APACHE_APPENDIX}");
    let bracketed = with.replace('{', "[").replace('}', "]");

    assert!(fits(apache, apache));
    assert!(fits(&with, apache));
    assert!(fits(&bracketed, apache));
    assert!(!fits(&format!("{apache}\nAnd you owe us a fee."), apache));
    assert!(!fits(&format!("{with}\nAnd you owe us a fee."), apache));
}

#[test]
fn only_apache_may_add_the_appendix() {
    let mit = canonical("MIT").unwrap();

    assert!(!fits(&format!("{mit}\n{APACHE_APPENDIX}"), mit));
    assert!(!fits(
        &format!(
            "{mit}\nAPPENDIX: How to apply the Apache License to your work. Not for commercial use."
        ),
        mit
    ));
}

#[test]
fn appendix_is_exactly_the_apache_boilerplate() {
    let words = normalize(APACHE_APPENDIX);

    assert!(appendix(&words));
    assert!(appendix(&format!("{words} ")));
    assert!(!appendix(&format!("{words} and more")));
    assert!(!appendix("appendix: how to apply the apache license"));
    assert!(!appendix(""));
}

#[test]
fn unmet_names_what_keeps_an_expression_from_being_satisfied() {
    let met = |id: &str| id == "MIT";
    let expr = |s: &str| parse(s).unwrap();

    assert_eq!(unmet(&expr("MIT"), &met), None);
    assert_eq!(unmet(&expr("ISC"), &met), Some(vec!["ISC".to_owned()]));
    assert_eq!(unmet(&expr("MIT OR ISC"), &met), None);
    assert_eq!(
        unmet(&expr("ISC OR NCSA"), &met),
        Some(vec!["ISC".to_owned(), "NCSA".to_owned()])
    );
    assert_eq!(
        unmet(&expr("MIT AND ISC"), &met),
        Some(vec!["ISC".to_owned()])
    );
    assert_eq!(unmet(&expr("MIT AND (ISC OR MIT)"), &met), None);
}

#[test]
fn license_files_also_reads_a_reuse_style_licenses_directory() {
    let (dir, _) = checkout(&[
        ("LICENSE", "top"),
        ("LICENSES/MIT.txt", "m"),
        ("LICENSES/Apache-2.0.txt", "a"),
        ("src/LICENSE", "deep"),
    ]);

    let names: Vec<String> = license_files(dir.path())
        .unwrap()
        .into_iter()
        .map(|f| f.0)
        .collect();

    assert_eq!(
        names,
        ["LICENSE", "LICENSES/Apache-2.0.txt", "LICENSES/MIT.txt"]
    );
}

fn id(s: &str) -> Expr {
    Expr::Id(s.to_owned())
}

#[test]
fn parse_reads_or_and_with_and_the_legacy_slash() {
    let either = Expr::Or(vec![id("MIT"), id("Apache-2.0")]);

    assert_eq!(parse("MIT OR Apache-2.0").unwrap(), either);
    assert_eq!(parse("MIT/Apache-2.0").unwrap(), either);
    assert_eq!(
        parse("Apache-2.0 WITH LLVM-exception OR MIT").unwrap(),
        Expr::Or(vec![id("Apache-2.0"), id("MIT")])
    );
    assert_eq!(parse("MIT").unwrap(), id("MIT"));
}

#[test]
fn parse_binds_and_tighter_than_or_and_honours_parentheses() {
    assert_eq!(
        parse("(MIT OR Apache-2.0) AND Unicode-3.0").unwrap(),
        Expr::And(vec![
            Expr::Or(vec![id("MIT"), id("Apache-2.0")]),
            id("Unicode-3.0")
        ])
    );
    assert_eq!(
        parse("Apache-2.0 OR (MIT AND Unicode-3.0)").unwrap(),
        Expr::Or(vec![
            id("Apache-2.0"),
            Expr::And(vec![id("MIT"), id("Unicode-3.0")])
        ])
    );
    assert_eq!(
        parse("A OR B AND C").unwrap(),
        Expr::Or(vec![id("A"), Expr::And(vec![id("B"), id("C")])])
    );
}

#[test]
fn parse_refuses_what_is_not_an_expression() {
    for bad in [
        "", "MIT OR", "(MIT", "MIT)", "AND MIT", "MIT WITH", "MIT MIT",
    ] {
        assert!(parse(bad).is_err(), "{bad:?}");
    }
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
        [
            "a 1.0.0 declares WTFPL, but no license file fits WTFPL (the tool keeps no text for WTFPL)"
        ]
    );
}

#[test]
fn a_nested_expression_is_satisfied_by_either_side_of_its_or() {
    let nested = package("a", Some("Apache-2.0 OR (MIT AND Unicode-3.0)"));
    let apache = file("LICENSE", canonical("Apache-2.0").unwrap());

    assert!(check_crate(&nested, &[apache]).is_empty());
    assert_eq!(
        check_crate(&nested, &[mit_file()]),
        [
            "a 1.0.0 declares Apache-2.0 OR (MIT AND Unicode-3.0), but no license file fits Apache-2.0 or Unicode-3.0"
        ]
    );
}

#[test]
fn a_license_that_does_not_parse_fails() {
    assert_eq!(
        check_crate(&package("a", Some("MIT OR")), &[mit_file()]),
        ["a 1.0.0 declares a license that does not parse: expression ends early"]
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
    let nameless = r#"{"packages":[{"version":"1.0.0","source":"registry+x","manifest_path":"/r/x/Cargo.toml"}]}"#;
    assert!(
        packages(nameless)
            .unwrap_err()
            .starts_with("cargo metadata lists a crate without a name, version or manifest path")
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
fn license_files_without_a_licenses_directory_reads_the_top_only() {
    let (dir, _) = checkout(&[("LICENSE", "top")]);

    let files = license_files(dir.path()).unwrap();

    assert_eq!(files, [("LICENSE".to_owned(), "top".to_owned())]);
}
