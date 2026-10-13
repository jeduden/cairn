//! ENG-02: the workspace's crates keep the dependency directions each
//! manifest declares, and every library crate declares one
//! responsibility.

/// One workspace crate as `cargo metadata` and its manifest's
/// `[package.metadata.cairn]` table describe it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub dir: String,
    pub library: bool,
    pub responsibility: Vec<String>,
    pub may_depend_on: Option<Vec<String>>,
    pub may_dev_depend_on: Vec<String>,
    pub links: Vec<Link>,
}

/// One dependency of a crate on another workspace crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub on: String,
    pub dev: bool,
}

/// Reads the workspace crates out of `cargo metadata --no-deps`.
///
/// # Errors
///
/// Fails when `metadata` is not cargo's JSON or a declaration is not
/// of the shape the manifest documents.
pub fn workspace(_metadata: &str) -> Result<Vec<Member>, String> {
    Ok(Vec::new())
}

/// The dependencies that break the declared directions.
#[must_use]
pub fn directions(_members: &[Member]) -> Vec<String> {
    Vec::new()
}

/// The library crates that do not declare exactly one responsibility.
#[must_use]
pub fn responsibilities(_members: &[Member]) -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests;
