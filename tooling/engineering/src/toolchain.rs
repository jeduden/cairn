//! ENG-01: every toolchain the repository builds with is fixed to an
//! exact version, Rust's by `rust-toolchain.toml`, and release builds
//! strip build paths, stamp the commit and link statically.

use crate::Checkout;

/// The prefix a step's `uses:` gives a workflow of this repository.
const LOCAL_WORKFLOW: &str = "uses: ./.github/workflows/";

/// The release build as the release workflow runs it: `workflow`'s
/// text, followed by each workflow of this repository it calls, YAML
/// comments left out so a flag only a comment names does not count.
///
/// # Errors
///
/// Fails when `workflow` or a workflow it calls cannot be read.
pub fn release_build(checkout: &Checkout, workflow: &str) -> Result<String, String> {
    let mut text = uncommented(&checkout.read(workflow)?);
    let called: Vec<String> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix(LOCAL_WORKFLOW))
        .map(|name| format!(".github/workflows/{}", name.trim()))
        .collect();
    for file in called {
        text.push_str(&uncommented(&checkout.read(&file)?));
    }

    Ok(text)
}

/// `body` without its YAML comments.
fn uncommented(body: &str) -> String {
    body.split_inclusive('\n')
        .map(crate::review_workflow::uncomment)
        .collect()
}

/// Checks that the release build's text contains `flag`.
///
/// # Errors
///
/// Fails when it does not.
pub fn builds_with(release_build: &str, flag: &str) -> Result<(), String> {
    if release_build.contains(flag) {
        Ok(())
    } else {
        Err(format!("the release build does not contain {flag:?}"))
    }
}

/// Checks that `toml`, a `rust-toolchain.toml`, fixes the Rust toolchain
/// to an exact release such as `channel = "1.99.0"`.
///
/// # Errors
///
/// Fails when the file names no channel, or one that is not
/// `MAJOR.MINOR.PATCH`.
pub fn rust_toolchain_pinned(toml: &str) -> Result<(), String> {
    let channel = toml
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("channel")?
                .trim()
                .strip_prefix('=')
                .map(str::trim)
        })
        .ok_or("rust-toolchain.toml names no channel")?
        .trim_matches('"');
    let parts: Vec<&str> = channel.split('.').collect();
    let exact = parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));

    if exact {
        Ok(())
    } else {
        Err(format!(
            "rust-toolchain.toml does not pin an exact release: {channel:?}"
        ))
    }
}

#[cfg(test)]
mod tests;
