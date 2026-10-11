//! Does anything stored confirm a guess at purged content?

use std::path::PathBuf;

use serde_json::Value;

use crate::{Error, Result};

/// A guess at one event's content: its text and its whole content.
pub struct Guess {
    pub text: String,
    pub content: Value,
}

/// Each way the named blobs confirm the guess, empty if none does.
pub fn confirmations(_blobs: &[(String, Vec<u8>)], _guess: &Guess) -> Vec<String> {
    vec!["scan not built".to_string()]
}

/// Reads every file into a named blob.
pub fn read_all(_files: &[PathBuf]) -> Result<Vec<(String, Vec<u8>)>> {
    Err(Error::new("scan not built"))
}
