//! The ENG-20 steps: release artifacts are signed, carry SLSA Build
//! Level 3 provenance and an SPDX SBOM.

use cucumber::{then, when};
use engineering::outcome;
use engineering::release_provenance::{self, Release};

use crate::World;
use crate::engineering::eng;

/// The release workflows the scenario read.
#[derive(Debug, Default)]
struct Published {
    release: Release,
}

fn published(w: &mut World) -> &mut Published {
    w.section::<Published>()
}

/// The release workflows the scenario reads: `release.yml`, the
/// release-only `release-build.yml` it calls, the `build.yml` that one
/// calls, and `ci.yml`, in the checkout. A checkout holds no release,
/// so the steps hold those workflows to the shape that makes every
/// release they publish verifiable, and require the `verify` job that
/// downloads what was published and checks it with `cosign` and
/// `gh attestation verify`.
#[when(regex = r"^the release workflows are read$")]
fn the_release_workflows_are_read(w: &mut World) -> Result<(), String> {
    let release = release_provenance::load(eng(w).checkout()?)?;
    published(w).release = release;
    Ok(())
}

#[then(regex = r"^the checksum file is signed keyless with Sigstore and verified once published$")]
fn checksums_verify(w: &mut World) -> Result<(), String> {
    outcome(published(w).release.checksums_signed_keyless())
}

#[then(
    regex = r"^only the release-only build workflow publishes each binary's SLSA Build Level 3 provenance as a GitHub artifact attestation, verified against that workflow's signing identity at main$"
)]
fn binaries_carry_provenance(w: &mut World) -> Result<(), String> {
    outcome(
        published(w)
            .release
            .provenance_from_the_release_build_workflow(),
    )
}

#[then(
    regex = r"^an SPDX SBOM is attached, published as a GitHub artifact attestation and verified once published$"
)]
fn sbom_attached(w: &mut World) -> Result<(), String> {
    outcome(published(w).release.sbom_attached_and_attested())
}
