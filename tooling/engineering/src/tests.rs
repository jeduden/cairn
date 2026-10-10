use super::*;

/// Lays `files` into a fresh directory standing in for the
/// repository.
pub(crate) fn checkout(files: &[(&str, &str)]) -> (testkit::TempDir, Checkout) {
    let dir = testkit::TempDir::new().unwrap();
    for (name, body) in files {
        let path = dir.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
    let checkout = Checkout::new(dir.path());
    (dir, checkout)
}

#[test]
fn contains_names_the_file_and_the_text() {
    let (_dir, c) = checkout(&[(".github/workflows/release.yml", "go build -trimpath\n")]);

    assert!(
        c.contains(".github/workflows/release.yml", "-trimpath")
            .is_ok()
    );
    assert_eq!(
        c.contains(".github/workflows/release.yml", "-buildvcs=true")
            .unwrap_err(),
        r#".github/workflows/release.yml does not contain "-buildvcs=true""#
    );
    assert!(
        c.contains("nope.md", "x")
            .unwrap_err()
            .starts_with("read nope.md: ")
    );
}

#[test]
fn requirement_reads_one_row_or_says_what_is_missing() {
    let row = "| ID | Pri | Requirement | Ver |\n|---|---|---|---|\n| ENG-18 | P0 | Text. | I |\n";
    let (_dir, c) = checkout(&[("docs/srs/10.md", row)]);

    assert_eq!(c.requirement("ENG-18").unwrap().text, "Text.");
    assert_eq!(
        c.requirement("ENG-19").unwrap_err(),
        "no requirement ENG-19 in the SRS"
    );

    let (_dir, broken) = checkout(&[(
        "docs/srs/10.md",
        "| ID | Requirement |\n|---|---|\n| bad | x |\n",
    )]);
    assert!(
        broken
            .requirement("ENG-18")
            .unwrap_err()
            .contains("malformed requirement id")
    );
}

#[test]
fn outcome_passes_no_problems_and_joins_the_rest() {
    assert!(outcome(vec![]).is_ok());
    assert_eq!(outcome(vec!["a".into(), "b".into()]).unwrap_err(), "a\nb");
}

#[test]
fn list_prints_items_in_brackets() {
    assert_eq!(list(&["a", "b"]), "[a b]");
    assert_eq!(list(&[] as &[&str]), "[]");
}
