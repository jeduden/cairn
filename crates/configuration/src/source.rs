//! Where each settings layer is read from, and the checks reading it
//! makes (ADM-04):
//!
//! - the principal's configuration: `config.toml` in the home;
//! - the repository configuration: `.cairn.toml` at the top of the git
//!   repository containing the working directory;
//! - managed policy: one documented path per OS, refused when the
//!   running OS user could write the file or its directory.
//!
//! A layer that is absent sets nothing. A layer that cannot be read or
//! fails validation is reported, never quoted, and the others still
//! load, so the core keeps recording (I9).

use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use record::digest::Digest;

use crate::layer::{LayerError, LayerValues, parse};
use crate::resolve::{Layers, SettingsLayer};

/// The largest settings layer read, in bytes.
pub const MAX_LAYER_BYTES: u64 = 1 << 20;

/// The principal's configuration's file name in the home.
pub const CONFIGURATION: &str = "config.toml";

/// The repository configuration's file name at a repository's top.
pub const REPOSITORY_CONFIGURATION: &str = ".cairn.toml";

/// Managed policy's documented path on this OS.
#[cfg(target_os = "macos")]
pub const MANAGED_POLICY: &str = "/Library/Application Support/Cairn/managed_policy.toml";
/// Managed policy's documented path on this OS.
#[cfg(windows)]
pub const MANAGED_POLICY: &str = r"C:\Program Files\Cairn\managed_policy.toml";
/// Managed policy's documented path on this OS.
#[cfg(not(any(target_os = "macos", windows)))]
pub const MANAGED_POLICY: &str = "/etc/cairn/managed_policy.toml";

/// Why a settings layer gives no settings.
#[derive(Debug)]
pub enum Failure {
    /// The file cannot be read.
    Unreadable(std::io::Error),
    /// The file is larger than [`MAX_LAYER_BYTES`].
    TooLarge,
    /// The file is not UTF-8.
    NotText,
    /// The text does not parse or validate.
    Layer(LayerError),
    /// The running OS user could write the file or its directory, so
    /// it is no managed policy (ADM-04).
    Writable(PathBuf),
    /// The path is a symbolic link, whose target no check can pin.
    SymbolicLink,
}

/// A settings layer that gives no settings, and why.
#[derive(Debug)]
pub struct LayerFailure {
    pub layer: SettingsLayer,
    pub path: PathBuf,
    pub failure: Failure,
}

impl fmt::Display for LayerFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: ", self.layer, self.path.display())?;
        match &self.failure {
            Failure::Unreadable(e) => write!(f, "{e}"),
            Failure::TooLarge => write!(f, "larger than {MAX_LAYER_BYTES} bytes"),
            Failure::NotText => f.write_str("not UTF-8 text"),
            Failure::Layer(e) => write!(f, "{e}"),
            Failure::Writable(path) => write!(
                f,
                "{} is writable by the running OS user, so it is no managed policy",
                path.display()
            ),
            Failure::SymbolicLink => f.write_str("a symbolic link, refused"),
        }
    }
}

impl std::error::Error for LayerFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            Failure::Unreadable(e) => Some(e),
            _ => None,
        }
    }
}

/// Where the settings layers are read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sources {
    pub configuration: PathBuf,
    pub repository: Option<PathBuf>,
    pub managed: PathBuf,
    /// The running OS user, whom managed policy's files must not let
    /// write; none where the OS has no such check yet.
    pub running: Option<u32>,
}

/// The settings layers as read: each that gave settings, the digest of
/// the principal's configuration's text, and each that failed.
#[derive(Debug, Default)]
pub struct Loaded {
    pub layers: Layers,
    pub configuration_digest: Option<Digest>,
    pub failures: Vec<LayerFailure>,
}

impl Loaded {
    /// Whether the principal's configuration or the repository
    /// configuration failed, which a command that shows or accepts
    /// settings refuses to go past (ADM-04).
    #[must_use]
    pub fn principal_layer_failed(&self) -> bool {
        self.failures
            .iter()
            .any(|f| f.layer != SettingsLayer::ManagedPolicy)
    }

    /// Whether managed policy failed: B1 to B3 components stay off and
    /// widening acts are refused while it does.
    #[must_use]
    pub fn managed_policy_failed(&self) -> bool {
        self.failures
            .iter()
            .any(|f| f.layer == SettingsLayer::ManagedPolicy)
    }
}

/// Reads every settings layer `sources` names.
#[must_use]
pub fn load(sources: &Sources) -> Loaded {
    let mut loaded = Loaded::default();
    let layer = SettingsLayer::Configuration;
    match read(&sources.configuration) {
        Ok(Some(text)) => {
            loaded.configuration_digest = Some(Digest::of(&[text.as_bytes()]));
            loaded.layers.configuration = values(&mut loaded, layer, &sources.configuration, &text);
        }
        Ok(None) => {}
        Err(failure) => loaded
            .failures
            .push(failed(layer, &sources.configuration, failure)),
    }
    if let Some(path) = &sources.repository {
        let layer = SettingsLayer::RepositoryConfiguration;
        loaded.layers.repository = layer_at(&mut loaded, layer, path);
    }
    let managed = &sources.managed;
    match managed_check(managed, sources.running) {
        Ok(()) => {
            loaded.layers.managed = layer_at(&mut loaded, SettingsLayer::ManagedPolicy, managed);
        }
        Err(failure) => {
            loaded
                .failures
                .push(failed(SettingsLayer::ManagedPolicy, managed, failure))
        }
    }

    loaded
}

/// The settings the layer at `path` sets, its failure noted in
/// `loaded`.
fn layer_at(loaded: &mut Loaded, layer: SettingsLayer, path: &Path) -> LayerValues {
    match read(path) {
        Ok(Some(text)) => values(loaded, layer, path, &text),
        Ok(None) => LayerValues::new(),
        Err(failure) => {
            loaded.failures.push(failed(layer, path, failure));
            LayerValues::new()
        }
    }
}

/// The settings `text` sets, its failure noted in `loaded`.
fn values(loaded: &mut Loaded, layer: SettingsLayer, path: &Path, text: &str) -> LayerValues {
    parse(text).unwrap_or_else(|e| {
        loaded.failures.push(failed(layer, path, Failure::Layer(e)));
        LayerValues::new()
    })
}

fn failed(layer: SettingsLayer, path: &Path, failure: Failure) -> LayerFailure {
    LayerFailure {
        layer,
        path: path.to_path_buf(),
        failure,
    }
}

/// The text of the file at `path`; none when there is no file.
fn read(path: &Path) -> Result<Option<String>, Failure> {
    let file = match fs::File::open(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        file => file.map_err(Failure::Unreadable)?,
    };
    let mut bytes = Vec::new();
    file.take(MAX_LAYER_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(Failure::Unreadable)?;
    if bytes.len() as u64 > MAX_LAYER_BYTES {
        return Err(Failure::TooLarge);
    }

    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| Failure::NotText)
}

/// Refuses managed policy at `path` unless neither it nor its
/// directory could be written by the OS user `running`: each belongs
/// to another OS user and lets no group or other user write. A path
/// with no file passes; there is no managed policy.
fn managed_check(path: &Path, running: Option<u32>) -> Result<(), Failure> {
    let meta = match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        meta => meta.map_err(Failure::Unreadable)?,
    };
    if meta.file_type().is_symlink() {
        return Err(Failure::SymbolicLink);
    }
    let Some(running) = running else {
        return Ok(());
    };
    let dir = path.parent().unwrap_or(path);
    for (p, m) in [(path, Ok(meta)), (dir, fs::metadata(dir))] {
        let m = m.map_err(Failure::Unreadable)?;
        if writable(&m, running) {
            return Err(Failure::Writable(p.to_path_buf()));
        }
    }

    Ok(())
}

/// Whether `meta` lets the OS user `running` write: it is the owner,
/// who may change the mode, or a group or other user may write.
#[cfg(unix)]
fn writable(meta: &fs::Metadata, running: u32) -> bool {
    use std::os::unix::fs::MetadataExt;

    meta.uid() == running || meta.mode() & 0o022 != 0
}

/// Without Unix modes, no check is possible yet: Windows' ACLs come
/// later, and `running` is none there.
#[cfg(not(unix))]
fn writable(_meta: &fs::Metadata, _running: u32) -> bool {
    true
}

/// The repository configuration for `working`: `.cairn.toml` at the
/// top of the git repository containing it, the nearest directory up
/// from it containing `.git`; none outside a repository.
#[must_use]
pub fn repository_configuration(working: &Path) -> Option<PathBuf> {
    working
        .ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(|top| top.join(REPOSITORY_CONFIGURATION))
}

#[cfg(test)]
mod tests;
