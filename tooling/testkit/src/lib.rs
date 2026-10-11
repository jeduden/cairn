//! Test-only helpers the tooling crates share: a temporary directory
//! and the path of the repository the tests inspect. Only test code
//! depends on this crate; nothing here links into a shipped
//! executable.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// How many names a [`TempDir`] tries before it gives up.
const ATTEMPTS: u32 = 1000;

/// The repository checkout the tests inspect: two levels above this
/// crate's manifest, as for every crate under `tooling/`.
#[must_use]
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Points `command` at `home` (ENG-14): its `HOME`, and a `CAIRN_HOME`
/// inside it, so an executable a test starts cannot reach the real home
/// or Claude Code configuration.
pub fn isolate<'a>(command: &'a mut Command, home: &Path) -> &'a mut Command {
    command
        .env("HOME", home)
        .env("CAIRN_HOME", home.join(".cairn"))
}

/// A directory under the system's temporary directory, removed with
/// everything in it when dropped.
#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Creates a fresh, empty directory readable only by its owner.
    ///
    /// # Errors
    ///
    /// Fails when the directory cannot be created.
    pub fn new() -> io::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut names = (0..ATTEMPTS)
            .map(|n| OsString::from(format!("cairn-{}-{stamp}-{n}", std::process::id())));
        create_unique(&std::env::temp_dir(), &mut names).map(|path| Self { path })
    }

    /// The directory's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // A directory left behind by a failed removal is the system
        // temporary directory's to clean; a test has nothing to do.
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Creates the first of `names` under `base` that does not exist yet.
/// `create_dir` refuses an existing name, so another process or user
/// cannot hand the test a directory it planted.
fn create_unique(base: &Path, names: &mut dyn Iterator<Item = OsString>) -> io::Result<PathBuf> {
    for name in names {
        let path = base.join(name);
        match owner_only().create(&path) {
            Ok(()) => return Ok(path),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("testkit: no free directory name under {}", base.display()),
    ))
}

#[cfg(unix)]
fn owner_only() -> fs::DirBuilder {
    use std::os::unix::fs::DirBuilderExt;

    let mut b = fs::DirBuilder::new();
    b.mode(0o700);
    b
}

#[cfg(not(unix))]
fn owner_only() -> fs::DirBuilder {
    fs::DirBuilder::new()
}

#[cfg(test)]
mod tests;
