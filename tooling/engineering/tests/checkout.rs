//! The checks' public API against the repository checkout itself: its
//! files, its SRS and its review workflow.

use engineering::{Checkout, review_workflow, toolchain};

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn a_checkout_reads_its_files_and_its_srs() {
    let c = checkout();

    assert_eq!(c.contains("Cargo.toml", "[workspace]"), Ok(()));
    assert_eq!(
        c.requirement("ENG-18").map(|r| r.id),
        Ok("ENG-18".to_owned())
    );
}

#[test]
fn a_checkout_names_what_it_cannot_find() {
    let c = checkout();

    assert!(c.read("no-such-file").is_err());
    assert_eq!(
        c.contains("Cargo.toml", "no such text"),
        Err(r#"Cargo.toml does not contain "no such text""#.to_owned())
    );
    assert_eq!(
        c.requirement("ENG-0").map(|r| r.id),
        Err("no requirement ENG-0 in the SRS".to_owned())
    );
}

#[test]
fn the_checkout_pins_its_toolchains() {
    let c = checkout();

    assert_eq!(
        c.read("rust-toolchain.toml")
            .and_then(|t| toolchain::rust_toolchain_pinned(&t)),
        Ok(())
    );
    assert_eq!(
        c.read("go.mod").map(|m| toolchain::go_toolchain_pinned(&m)),
        Ok(true)
    );
}

#[test]
fn the_checkout_s_review_workflow_parses_into_its_trigger_and_jobs() {
    let flow = checkout()
        .read(".github/workflows/review.yml")
        .map(|body| review_workflow::parse(&body));

    assert_eq!(
        flow.as_ref().map(|w| w.triggers.clone()),
        Ok(vec!["workflow_run".to_owned()])
    );
    assert!(flow.is_ok_and(|w| w.jobs.len() >= 2));
}
