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
fn the_checkout_pins_its_rust_toolchain() {
    let c = checkout();

    assert_eq!(
        c.read("rust-toolchain.toml")
            .and_then(|t| toolchain::rust_toolchain_pinned(&t)),
        Ok(())
    );
}

#[test]
fn the_checkout_s_release_build_runs_through_the_build_workflow() {
    let build = toolchain::release_build(&checkout(), ".github/workflows/release.yml");

    let build = build.as_deref().map_err(String::as_str);
    assert_eq!(
        build
            .and_then(|b| toolchain::builds_with(b, "--remap-path-prefix").map_err(|_| "no remap")),
        Ok(())
    );
}

#[test]
fn the_checkout_s_review_workflow_parses_into_its_trigger_and_jobs() {
    let flow = checkout()
        .read(".github/workflows/review.yml")
        .map(|body| review_workflow::parse(&body));

    assert_eq!(
        flow.as_ref().map(|w| w.triggers.clone()),
        Ok(vec!["workflow_run".to_owned(), "issue_comment".to_owned()])
    );
    assert!(flow.is_ok_and(|w| w.jobs.len() >= 2));
}

/// Every product crate's clippy.toml refuses the core's reach (I4) and
/// the clock and random order derived artifacts may not read (I10).
#[test]
fn the_product_crates_keep_their_bans() {
    let problems = engineering::product_crates::crates_keep_their_bans(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// No product crate keeps mutable global state (ENG-03).
#[test]
fn no_product_crate_keeps_mutable_global_state() {
    let problems = engineering::product_crates::no_mutable_globals(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The workflows that publish a release sign its checksums keyless
/// (ENG-20).
#[test]
fn the_checkout_s_release_signs_its_checksums_keyless() {
    let release = engineering::release_provenance::load(&checkout()).unwrap_or_default();

    assert_eq!(release.checksums_signed_keyless(), Vec::<String>::new());
}

/// The reusable build workflow attests the binaries' provenance, and
/// the release workflow verifies what it published (ENG-20).
#[test]
fn the_checkout_s_release_attests_provenance_in_the_build_workflow() {
    let release = engineering::release_provenance::load(&checkout()).unwrap_or_default();

    assert_eq!(
        release.provenance_from_the_build_workflow(),
        Vec::<String>::new()
    );
}

/// The release attaches and attests an SPDX SBOM (ENG-20).
#[test]
fn the_checkout_s_release_attaches_and_attests_an_sbom() {
    let release = engineering::release_provenance::load(&checkout()).unwrap_or_default();

    assert_eq!(release.sbom_attached_and_attested(), Vec::<String>::new());
}
