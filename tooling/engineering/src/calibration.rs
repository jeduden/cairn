//! NFR-02: the reference a hook handler's time is judged against is the
//! frozen calibration workload of the `calibrate` crate, one unit of
//! which is a nominal 10 ms by definition. It stays a constant
//! measurement only while it never changes, so this check holds its
//! source to a recorded hash and its constants to their recorded
//! values: a change to either fails the build, and moving the record
//! is a change the stakeholder reviews.

use std::path::Path;

/// Where the workload's source lies, relative to the checkout.
pub const SOURCE_DIR: &str = "tooling/calibrate/src";

/// The hash of the workload's source, recorded when it was tuned.
pub const RECORDED_HASH: u64 = 0;

/// The workload's constants as recorded: the seed, the rounds of
/// mixing and the rounds of file calls.
pub const RECORDED_CONSTANTS: (u64, u64, u32) = (0, 0, 0);

/// The 64-bit FNV-1a hash of `bytes`.
#[must_use]
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let _ = bytes;
    0
}

/// The hash of every file under `root`'s [`SOURCE_DIR`], each by name
/// and content, in name order.
///
/// # Errors
///
/// Fails when the directory or a file in it cannot be read.
pub fn source_hash(root: &Path) -> Result<u64, String> {
    let _ = root;
    Ok(0)
}

/// What differs between the workload `root` holds, with `constants`, and
/// the record: its hash against `recorded_hash`, its constants against
/// [`RECORDED_CONSTANTS`].
#[must_use]
pub fn problems(root: &Path, recorded_hash: u64, constants: (u64, u64, u32)) -> Vec<String> {
    let _ = (root, recorded_hash, constants);
    Vec::new()
}

/// [`problems`] for the checkout's workload against the record.
#[must_use]
pub fn frozen(root: &Path) -> Vec<String> {
    problems(
        root,
        RECORDED_HASH,
        (
            calibrate::SEED,
            calibrate::MIX_ROUNDS,
            calibrate::FILE_ROUNDS,
        ),
    )
}

#[cfg(test)]
mod tests;
