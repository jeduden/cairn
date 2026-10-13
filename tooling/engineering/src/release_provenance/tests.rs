use super::*;

const BUILD_YML: &str = "\
name: Build
on:
  workflow_call:
    inputs:
      attest:
        type: boolean
jobs:
  build:
    runs-on: ubuntu-latest
  attest:
    if: ${{ inputs.attest }}
    needs: [build]
    permissions:
      id-token: write
      attestations: write
    steps:
      - uses: actions/attest-build-provenance@abc # v4
";

const RELEASE_YML: &str = "\
name: Release
on:
  workflow_dispatch:
jobs:
  build:
    uses: ./.github/workflows/build.yml
    permissions:
      id-token: write
      attestations: write
    with:
      attest: true
  publish:
    needs: build
    permissions:
      id-token: write
      attestations: write
    steps:
      - uses: anchore/sbom-action@abc
        with:
          format: spdx-json
          output-file: dist/cairn.spdx.json
      - uses: actions/attest-sbom@abc
        with:
          sbom-path: dist/cairn.spdx.json
      - run: cosign sign-blob --bundle checksums.txt.sigstore.json checksums.txt
      - run: |
          gh release create \"$VERSION\" \\
            --generate-notes \\
            cairn-* cairn.spdx.json checksums.txt checksums.txt.sigstore.json
  verify:
    needs: publish
    steps:
      - run: |
          cosign verify-blob --bundle checksums.txt.sigstore.json \\
            --certificate-identity-regexp '^https://github.com/jeduden/cairn/.github/workflows/release.yml@' \\
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \\
            checksums.txt
          for bin in cairn-*; do
            gh attestation verify \"$bin\" --repo jeduden/cairn --signer-workflow jeduden/cairn/.github/workflows/build.yml
            gh attestation verify \"$bin\" --repo jeduden/cairn --predicate-type https://spdx.dev/Document/v2.3
          done
";

fn release(release: &str, build: &str) -> Release {
    Release {
        release: parse(release),
        build: parse(build),
    }
}

fn good() -> Release {
    release(RELEASE_YML, BUILD_YML)
}

fn drifted(from: &str, to: &str) -> Release {
    assert!(RELEASE_YML.contains(from), "{from}");
    release(&RELEASE_YML.replace(from, to), BUILD_YML)
}

fn build_drifted(from: &str, to: &str) -> Release {
    assert!(BUILD_YML.contains(from), "{from}");
    release(RELEASE_YML, &BUILD_YML.replace(from, to))
}

fn has(problems: &[String], want: &str) {
    assert!(
        problems.iter().any(|p| p.contains(want)),
        "{want:?} not in {problems:#?}"
    );
}

#[test]
fn a_good_release_has_no_problem() {
    let r = good();

    assert_eq!(r.checksums_signed_keyless(), Vec::<String>::new());
    assert_eq!(r.provenance_from_the_build_workflow(), Vec::<String>::new());
    assert_eq!(r.sbom_attached_and_attested(), Vec::<String>::new());
}

#[test]
fn load_reads_both_workflows_or_names_the_missing_one() {
    let (_dir, c) = crate::tests::checkout(&[(RELEASE, RELEASE_YML), (BUILD, BUILD_YML)]);
    assert_eq!(load(&c).unwrap(), good());

    let (_dir, c) = crate::tests::checkout(&[(RELEASE, RELEASE_YML)]);
    assert!(load(&c).unwrap_err().starts_with(&format!("read {BUILD}")));
}

#[test]
fn provenance_attested_in_the_release_workflow_is_outside_the_build_workflow() {
    let r = drifted(
        "      - uses: actions/attest-sbom@abc\n",
        "      - uses: actions/attest-build-provenance@abc\n",
    );

    has(
        &r.provenance_from_the_build_workflow(),
        "release.yml attests provenance outside the reusable build workflow, in job publish",
    );
}

#[test]
fn provenance_missing_from_the_build_workflow_is_named() {
    let r = build_drifted("actions/attest-build-provenance@", "actions/other@");

    has(
        &r.provenance_from_the_build_workflow(),
        "build.yml does not attest build provenance",
    );
}

#[test]
fn the_attest_job_needs_both_permissions_and_the_callers_ask() {
    let r = build_drifted("      attestations: write\n", "");
    has(
        &r.provenance_from_the_build_workflow(),
        "job attest lacks attestations: write",
    );

    let r = build_drifted("    if: ${{ inputs.attest }}\n", "");
    has(
        &r.provenance_from_the_build_workflow(),
        "attests whether or not the caller asks",
    );
}

#[test]
fn the_caller_must_ask_and_grant() {
    let r = drifted("      attest: true\n", "      attest: false\n");
    has(
        &r.provenance_from_the_build_workflow(),
        "job build calls the build workflow without attest: true",
    );

    let r = drifted(
        "    uses: ./.github/workflows/build.yml\n    permissions:\n      id-token: write\n",
        "    uses: ./.github/workflows/build.yml\n    permissions:\n",
    );
    has(
        &r.provenance_from_the_build_workflow(),
        "job build lacks id-token: write",
    );

    let r = drifted("uses: ./.github/workflows/build.yml", "uses: x/y@z");
    has(
        &r.provenance_from_the_build_workflow(),
        "release.yml does not call the reusable build workflow",
    );
}

#[test]
fn a_release_without_a_verify_job_is_named_by_every_check() {
    let r = drifted(
        "  verify:\n    needs: publish\n",
        "  verify:\n    needs: build\n",
    );
    let want = "no job verifies the published release after job publish";

    has(&r.provenance_from_the_build_workflow(), want);
    has(&r.checksums_signed_keyless(), want);
    has(&r.sbom_attached_and_attested(), want);
}

#[test]
fn a_release_nobody_creates_has_no_publish_job() {
    let r = drifted("gh release create", "gh release view");
    let want = "no job in release.yml creates the release";

    has(&r.checksums_signed_keyless(), want);
    has(&r.provenance_from_the_build_workflow(), want);
    has(&r.sbom_attached_and_attested(), want);
}

#[test]
fn the_verify_job_pins_the_build_workflow_as_signer_for_every_binary() {
    let r = drifted(
        "--signer-workflow jeduden/cairn/.github/workflows/build.yml",
        "--signer-workflow jeduden/cairn/.github/workflows/release.yml",
    );
    has(
        &r.provenance_from_the_build_workflow(),
        "does not verify provenance with --repo jeduden/cairn --signer-workflow jeduden/cairn/.github/workflows/build.yml",
    );

    let r = drifted("for bin in cairn-*; do", "for bin in cairn-linux-amd64; do");
    has(
        &r.provenance_from_the_build_workflow(),
        "does not verify every binary (cairn-*)",
    );
}

#[test]
fn the_checksums_are_signed_keyless_and_published_with_their_bundle() {
    let r = drifted(
        "cosign sign-blob --bundle checksums.txt.sigstore.json checksums.txt",
        "cosign sign-blob --key k.pem --bundle checksums.txt.sigstore.json checksums.txt",
    );
    let problems = r.checksums_signed_keyless();
    has(&problems, "does not run");
    has(&problems, "signs with a key, not keyless");

    let r = drifted(
        "    needs: build\n    permissions:\n      id-token: write\n",
        "    needs: build\n    permissions:\n",
    );
    has(
        &r.checksums_signed_keyless(),
        "job publish lacks the id-token write permission",
    );

    let r = drifted(
        "cairn-* cairn.spdx.json checksums.txt checksums.txt.sigstore.json",
        "cairn-* cairn.spdx.json",
    );
    let problems = r.checksums_signed_keyless();
    has(&problems, "the release does not attach checksums.txt");
    has(
        &problems,
        "the release does not attach checksums.txt.sigstore.json",
    );
}

#[test]
fn the_verify_job_checks_the_bundle_against_the_release_workflows_identity() {
    let r = drifted("cosign verify-blob", "cosign version");
    has(
        &r.checksums_signed_keyless(),
        "job verify does not run cosign verify-blob",
    );

    let r = drifted(
        "--certificate-identity-regexp '^https://github.com/jeduden/cairn/.github/workflows/release.yml@'",
        "--certificate-identity-regexp '^https://github.com/other/'",
    );
    has(
        &r.checksums_signed_keyless(),
        "does not pin the certificate identity to jeduden/cairn/.github/workflows/release.yml",
    );

    let r = drifted(
        "--certificate-oidc-issuer https://token.actions.githubusercontent.com",
        "--certificate-oidc-issuer https://example.com",
    );
    has(
        &r.checksums_signed_keyless(),
        "does not pin the OIDC issuer https://token.actions.githubusercontent.com",
    );

    let r = drifted(
        "cosign verify-blob --bundle checksums.txt.sigstore.json",
        "cosign verify-blob",
    );
    has(
        &r.checksums_signed_keyless(),
        "job verify verifies no checksums.txt.sigstore.json",
    );
}

#[test]
fn the_sbom_is_generated_as_spdx_attested_and_attached() {
    let r = drifted("anchore/sbom-action@", "anchore/other@");
    has(&r.sbom_attached_and_attested(), "does not generate an SBOM");

    let r = drifted("format: spdx-json", "format: cyclonedx-json");
    has(
        &r.sbom_attached_and_attested(),
        "job publish does not generate an SPDX SBOM",
    );

    let r = drifted("          output-file: dist/cairn.spdx.json\n", "");
    has(
        &r.sbom_attached_and_attested(),
        "job publish names no SBOM file",
    );

    let r = drifted("uses: actions/attest-sbom@", "uses: actions/other@");
    has(&r.sbom_attached_and_attested(), "does not attest the SBOM");

    let r = drifted(
        "sbom-path: dist/cairn.spdx.json",
        "sbom-path: dist/other.spdx.json",
    );
    has(
        &r.sbom_attached_and_attested(),
        "does not attest the generated SBOM dist/cairn.spdx.json",
    );

    let r = drifted(
        "cairn-* cairn.spdx.json checksums.txt",
        "cairn-* checksums.txt",
    );
    has(
        &r.sbom_attached_and_attested(),
        "the release does not attach the SBOM cairn.spdx.json",
    );

    let r = drifted(" --predicate-type https://spdx.dev/Document/v2.3", "");
    has(
        &r.sbom_attached_and_attested(),
        "does not verify the SBOM attestation (--predicate-type https://spdx.dev/Document/v2.3)",
    );
}

#[test]
fn needs_reads_a_name_or_a_list() {
    let job = |text: &str| Job {
        name: "j".to_owned(),
        text: text.to_owned(),
    };

    assert_eq!(needs(&job("    needs: a\n")), ["a"]);
    assert_eq!(needs(&job("    needs: [a, b]\n")), ["a", "b"]);
    assert!(needs(&job("    runs-on: x\n")).is_empty());
}

#[test]
fn logical_joins_continued_lines_and_trims_indents() {
    assert_eq!(logical("  a \\\n    b \\\n    c\n  d\n"), "a b c\nd\n");
}
