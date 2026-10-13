//! The ENG-20 steps: release artifacts are signed, carry SLSA Build
//! Level 3 provenance and an SPDX SBOM.

use cucumber::{given, then};
use engineering::release_provenance::{self, Release};
use engineering::{Checkout, outcome};

use crate::World;

/// The release workflows the scenario read.
#[derive(Debug, Default)]
struct Published {
    release: Release,
}

fn published(w: &mut World) -> &mut Published {
    w.section::<Published>()
}

/// "A published release" is read from the workflows that publish it:
/// `release.yml` and the reusable `build.yml` it calls, in this
/// checkout. A checkout holds no release, so the steps hold those
/// workflows to the shape that makes every release they publish
/// verifiable. Checking an actual release is the workflow's own
/// `verify` job, which downloads what it published and verifies it
/// with `cosign` and `gh attestation verify`, as the steps below
/// require it to.
#[given(regex = r"^a published release$")]
fn a_published_release(w: &mut World) -> Result<(), String> {
    let checkout = Checkout::new(&testkit::repo_root());
    published(w).release = release_provenance::load(&checkout)?;
    Ok(())
}

#[then(regex = r"^the checksum file verifies with its keyless Sigstore bundle$")]
fn checksums_verify(w: &mut World) -> Result<(), String> {
    outcome(published(w).release.checksums_signed_keyless())
}

#[then(regex = r"^every binary carries SLSA Build Level 3 provenance$")]
fn binaries_carry_provenance(w: &mut World) -> Result<(), String> {
    outcome(published(w).release.provenance_from_the_build_workflow())
}

#[then(regex = r"^an SPDX SBOM is attached and attested$")]
fn sbom_attached(w: &mut World) -> Result<(), String> {
    outcome(published(w).release.sbom_attached_and_attested())
}
