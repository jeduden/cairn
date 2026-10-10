//! Steps of `features/engineering.feature` shared across §10: the
//! checkout a scenario inspects, ENG-01's toolchain and release flags,
//! and ENG-24's disclosure policy.

use adr::Adr;
use cucumber::{given, then};
use drift::Case;
use engineering::Checkout;
use engineering::review_workflow::Workflow;

use crate::World;

/// What the §10 scenarios track beside the shared world: the checkout
/// they inspect and what a step read out of it.
#[derive(Debug, Default)]
pub struct Engineering {
    pub checkout: Option<Checkout>,
    pub deps: Vec<String>,
    pub adr_dir: String,
    pub adrs: Vec<Adr>,
    pub drifts: Vec<Case>,
    pub flow: Workflow,
}

impl Engineering {
    /// The checkout the scenario opened on.
    pub fn checkout(&self) -> Result<&Checkout, String> {
        self.checkout
            .as_ref()
            .ok_or_else(|| "the scenario must open on the repository checkout".to_owned())
    }
}

pub fn eng(w: &mut World) -> &mut Engineering {
    w.section::<Engineering>()
}

const RELEASE: &str = ".github/workflows/release.yml";

#[given(regex = r"^the repository checkout$")]
fn the_repository_checkout(w: &mut World) {
    eng(w).checkout = Some(Checkout::new(&testkit::repo_root()));
}

#[then(regex = r#"^"([^"]+)" pins the Go toolchain with a toolchain directive$"#)]
fn pins_the_go_toolchain(w: &mut World, file: String) -> Result<(), String> {
    let checkout = eng(w).checkout()?;
    if engineering::toolchain::go_toolchain_pinned(&checkout.read(&file)?) {
        Ok(())
    } else {
        Err(format!("{file} has no toolchain directive"))
    }
}

#[then(regex = r#"^the release workflow sets CGO_ENABLED to "0"$"#)]
fn release_is_static(w: &mut World) -> Result<(), String> {
    eng(w).checkout()?.contains(RELEASE, r#"CGO_ENABLED: "0""#)
}

#[then(regex = r#"^the release workflow builds with "([^"]+)"$"#)]
fn release_builds_with(w: &mut World, flag: String) -> Result<(), String> {
    eng(w).checkout()?.contains(RELEASE, &flag)
}

#[then(regex = r#"^"SECURITY.md" links the private vulnerability reporting channel$"#)]
fn security_links_the_channel(w: &mut World) -> Result<(), String> {
    eng(w)
        .checkout()?
        .contains("SECURITY.md", "/security/advisories/new")
}

#[then(regex = r#"^"SECURITY.md" states a 90-day coordinated disclosure policy$"#)]
fn security_states_the_policy(w: &mut World) -> Result<(), String> {
    eng(w)
        .checkout()?
        .contains("SECURITY.md", "90-day coordinated disclosure")
}

#[then(
    regex = r#"^"SECURITY.md" states that security fixes are backported to the latest minor release$"#
)]
fn security_states_backports(w: &mut World) -> Result<(), String> {
    eng(w)
        .checkout()?
        .contains("SECURITY.md", "backported to the latest minor release")
}
