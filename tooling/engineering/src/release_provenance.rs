//! ENG-20: release artifacts are signed with Sigstore, keyless, and
//! carry SLSA Build Level 3 provenance and an SPDX SBOM.
//!
//! A release is not something a checkout can hold, so these checks read
//! the workflows that publish it: `release.yml`, and the reusable
//! `build.yml` it calls. They hold the workflows to the shape that makes
//! each published release verifiable, and the release workflow's own
//! `verify` job runs that verification against the live release.

use crate::Checkout;
use crate::review_workflow::{Job, Workflow, parse};

/// The release workflow.
pub const RELEASE: &str = ".github/workflows/release.yml";

/// The reusable workflow that builds, and signs the build provenance.
pub const BUILD: &str = ".github/workflows/build.yml";

/// The repository the release belongs to, as `gh attestation verify`
/// names it.
const REPO: &str = "jeduden/cairn";

/// How `gh attestation verify` names the workflow that must have signed
/// each binary's build provenance.
const BUILD_SIGNER: &str = "jeduden/cairn/.github/workflows/build.yml";

/// How a certificate identity names the release workflow, the signer of
/// the checksums and the SBOM attestation.
const RELEASE_SIGNER: &str = "jeduden/cairn/.github/workflows/release.yml";

/// The call of the reusable build workflow.
const CALLS_BUILD: &str = "uses: ./.github/workflows/build.yml";

/// The action that attests build provenance.
const PROVENANCE: &str = "actions/attest-build-provenance@";

/// The action that attests an SBOM.
const ATTEST_SBOM: &str = "actions/attest-sbom@";

/// The action that generates the SBOM.
const MAKE_SBOM: &str = "anchore/sbom-action@";

/// The Sigstore bundle that carries the checksum file's signature.
const BUNDLE: &str = "checksums.txt.sigstore.json";

/// The OIDC issuer of GitHub Actions' keyless identity.
const ISSUER: &str = "https://token.actions.githubusercontent.com";

/// The predicate type of an SPDX SBOM attestation.
const SPDX_PREDICATE: &str = "https://spdx.dev/Document/v2.3";

/// The two workflows a release runs through.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Release {
    pub release: Workflow,
    pub build: Workflow,
}

/// Reads the release workflow and the reusable build workflow from
/// `checkout`.
///
/// # Errors
///
/// Fails when either file cannot be read.
pub fn load(checkout: &Checkout) -> Result<Release, String> {
    Ok(Release {
        release: parse(&checkout.read(RELEASE)?),
        build: parse(&checkout.read(BUILD)?),
    })
}

impl Release {
    /// Problems with how the checksum file is signed and verified: it
    /// is signed keyless in the publish job, published with its bundle,
    /// and a later job verifies the bundle against the release
    /// workflow's identity.
    #[must_use]
    pub fn checksums_signed_keyless(&self) -> Vec<String> {
        let mut problems = Vec::new();
        match self.publishing() {
            Err(e) => problems.push(e),
            Ok(publish) => problems.extend(signing_problems(publish)),
        }
        match self.verifying() {
            Err(e) => problems.push(e),
            Ok(verify) => problems.extend(cosign_problems(verify)),
        }

        problems
    }

    /// Problems with the binaries' build provenance: it is attested inside the
    /// reusable build workflow and nowhere else, only when the caller
    /// asks, and a later job verifies it against that workflow as the
    /// signer.
    #[must_use]
    pub fn provenance_from_the_build_workflow(&self) -> Vec<String> {
        let mut problems: Vec<String> = self
            .release
            .jobs
            .iter()
            .filter(|j| j.text.contains(PROVENANCE))
            .map(|j| {
                format!(
                    "release.yml attests build provenance outside the reusable build workflow, in job {}",
                    j.name
                )
            })
            .collect();
        match self.build.jobs.iter().find(|j| j.text.contains(PROVENANCE)) {
            None => problems.push("build.yml does not attest build provenance".to_owned()),
            Some(job) => problems.extend(attest_job_problems(job)),
        }
        match self
            .release
            .jobs
            .iter()
            .find(|j| j.text.contains(CALLS_BUILD))
        {
            None => {
                problems.push("release.yml does not call the reusable build workflow".to_owned());
            }
            Some(job) => problems.extend(caller_problems(job)),
        }
        match self.verifying() {
            Err(e) => problems.push(e),
            Ok(verify) => problems.extend(provenance_verify_problems(verify)),
        }

        problems
    }

    /// Problems with the SBOM: an SPDX document generated, attested to
    /// every binary and attached to the release, and a later job
    /// verifies the attestation.
    #[must_use]
    pub fn sbom_attached_and_attested(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let sbom = self.sbom_file(&mut problems);
        match self
            .release
            .jobs
            .iter()
            .find(|j| j.text.contains(ATTEST_SBOM))
        {
            None => problems.push("release.yml does not attest the SBOM".to_owned()),
            Some(job) => problems.extend(
                sbom.iter()
                    .filter(|file| value_of(&job.text, "sbom-path").as_ref() != Some(*file))
                    .map(|file| {
                        format!("job {} does not attest the generated SBOM {file}", job.name)
                    }),
            ),
        }
        match self.publishing() {
            Err(e) => problems.push(e),
            Ok(publish) => problems.extend(sbom.iter().filter_map(|file| {
                let name = file.rsplit('/').next().unwrap_or(file);
                (!release_assets(publish).iter().any(|a| a == name))
                    .then(|| format!("the release does not attach the SBOM {name}"))
            })),
        }
        match self.verifying() {
            Err(e) => problems.push(e),
            Ok(verify) => {
                let spdx = format!("--predicate-type {SPDX_PREDICATE}");
                if !attestation_commands(verify)
                    .iter()
                    .any(|c| c.contains(&spdx))
                {
                    problems.push(format!(
                        "job {} does not verify the SBOM attestation ({spdx})",
                        verify.name
                    ));
                }
            }
        }

        problems
    }

    /// The SPDX file the release workflow generates, or why there is
    /// none.
    fn sbom_file(&self, problems: &mut Vec<String>) -> Option<String> {
        let Some(job) = self
            .release
            .jobs
            .iter()
            .find(|j| j.text.contains(MAKE_SBOM))
        else {
            problems.push("release.yml does not generate an SBOM".to_owned());
            return None;
        };
        if value_of(&job.text, "format").as_deref() != Some("spdx-json") {
            problems.push(format!("job {} does not generate an SPDX SBOM", job.name));
        }
        let file = value_of(&job.text, "output-file");
        if file.is_none() {
            problems.push(format!("job {} names no SBOM file", job.name));
        }

        file
    }

    /// The job that creates the release.
    fn publishing(&self) -> Result<&Job, String> {
        self.release
            .jobs
            .iter()
            .find(|j| logical(&j.text).contains("gh release create"))
            .ok_or_else(|| "no job in release.yml creates the release".to_owned())
    }

    /// The job that verifies the published release: one that runs only
    /// after the publishing job.
    fn verifying(&self) -> Result<&Job, String> {
        let publish = self.publishing()?;
        self.release
            .jobs
            .iter()
            .find(|j| needs(j).contains(&publish.name))
            .ok_or_else(|| {
                format!(
                    "no job verifies the published release after job {}",
                    publish.name
                )
            })
    }
}

/// Problems with the publishing job's keyless signature and assets.
fn signing_problems(publish: &Job) -> Vec<String> {
    let text = logical(&publish.text);
    let mut problems = Vec::new();
    let sign = format!("cosign sign-blob --bundle {BUNDLE} checksums.txt");
    if !text.contains(&sign) {
        problems.push(format!("job {} does not run {sign:?}", publish.name));
    }
    if text.contains("--key") {
        problems.push(format!(
            "job {} signs with a key, not keyless",
            publish.name
        ));
    }
    if !text.contains("id-token: write") {
        problems.push(format!(
            "job {} lacks the id-token write permission for a keyless signature",
            publish.name
        ));
    }
    let assets = release_assets(publish);
    for want in ["checksums.txt", BUNDLE] {
        if !assets.iter().any(|a| a == want) {
            problems.push(format!("the release does not attach {want}"));
        }
    }

    problems
}

/// Problems with the verify job's check of the checksum bundle.
fn cosign_problems(verify: &Job) -> Vec<String> {
    let text = logical(&verify.text);
    let Some(line) = text.lines().find(|l| l.contains("cosign verify-blob")) else {
        return vec![format!(
            "job {} does not run cosign verify-blob",
            verify.name
        )];
    };
    let mut problems = Vec::new();
    if !line.contains(&format!("--bundle {BUNDLE}")) {
        problems.push(format!("job {} verifies no {BUNDLE}", verify.name));
    }
    if !line.contains("--certificate-identity") || !line.contains(RELEASE_SIGNER) {
        problems.push(format!(
            "job {} does not pin the certificate identity to {RELEASE_SIGNER}",
            verify.name
        ));
    }
    if !line.contains(&format!("--certificate-oidc-issuer {ISSUER}")) {
        problems.push(format!(
            "job {} does not pin the OIDC issuer {ISSUER}",
            verify.name
        ));
    }

    problems
}

/// Problems with the job in the build workflow that attests the
/// build provenance.
fn attest_job_problems(job: &Job) -> Vec<String> {
    let mut problems = permission_problems(job);
    let condition = job
        .text
        .lines()
        .find_map(|l| l.strip_prefix("    if:"))
        .unwrap_or_default();
    if !condition.contains("inputs.attest") {
        problems.push(format!(
            "build.yml job {} attests whether or not the caller asks",
            job.name
        ));
    }

    problems
}

/// Problems with the release job that calls the build workflow.
fn caller_problems(job: &Job) -> Vec<String> {
    let mut problems = permission_problems(job);
    if !job.text.contains("attest: true") {
        problems.push(format!(
            "job {} calls the build workflow without attest: true",
            job.name
        ));
    }

    problems
}

/// The two permissions an attestation needs, as `job` must hold them.
fn permission_problems(job: &Job) -> Vec<String> {
    ["id-token: write", "attestations: write"]
        .into_iter()
        .filter(|p| !job.text.contains(p))
        .map(|p| format!("job {} lacks {p}", job.name))
        .collect()
}

/// Problems with the verify job's check of every binary's build provenance.
fn provenance_verify_problems(verify: &Job) -> Vec<String> {
    let mut problems = Vec::new();
    let signer = format!("--signer-workflow {BUILD_SIGNER}");
    let repo = format!("--repo {REPO}");
    let verifies = attestation_commands(verify)
        .into_iter()
        .any(|c| c.contains(&signer) && c.contains(&repo));
    if !verifies {
        problems.push(format!(
            "job {} does not verify build provenance with {repo} {signer}",
            verify.name
        ));
    }
    if !verify.text.contains("cairn-*") {
        problems.push(format!(
            "job {} does not verify every binary (cairn-*)",
            verify.name
        ));
    }

    problems
}

/// The `gh attestation verify` commands of `job`, each as one line.
fn attestation_commands(job: &Job) -> Vec<String> {
    logical(&job.text)
        .lines()
        .filter(|l| l.contains("gh attestation verify"))
        .map(str::to_owned)
        .collect()
}

/// The names `gh release create` attaches in `job`: the words after
/// `--generate-notes`, the last option the repository's workflow gives.
fn release_assets(job: &Job) -> Vec<String> {
    let text = logical(&job.text);
    text.lines()
        .find(|l| l.contains("gh release create"))
        .map(|line| {
            line.split_whitespace()
                .skip_while(|w| *w != "--generate-notes")
                .skip(1)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The jobs `job` needs, from a one-line `needs:`.
fn needs(job: &Job) -> Vec<String> {
    job.text
        .lines()
        .find_map(|l| l.strip_prefix("    needs:"))
        .map(|rest| {
            rest.trim()
                .trim_matches(['[', ']'])
                .split(',')
                .map(|n| n.trim().to_owned())
                .filter(|n| !n.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// The value a `key:` line gives in `text`, without quotes.
fn value_of(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        l.trim_start()
            .strip_prefix(key)?
            .strip_prefix(':')
            .map(|v| v.trim().trim_matches(['"', '\'']).to_owned())
    })
}

/// `text` with each shell line continuation joined, so a command is one
/// line however the workflow wraps it.
fn logical(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines().map(str::trim) {
        match line.strip_suffix('\\') {
            Some(head) => {
                out.push_str(head.trim_end());
                out.push(' ');
            }
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }

    out
}

#[cfg(test)]
mod tests;
