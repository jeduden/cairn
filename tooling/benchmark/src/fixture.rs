//! What a benchmark's core runs over, made before each sample and
//! outside its time: for ingestion (NFR-04), a fresh home holding one
//! transcript under its transcript roots. `HOME` and `CAIRN_HOME` both
//! point inside it, so a sample never reaches the machine's own home,
//! Cairn home or Claude Code configuration (ENG-14).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A recorded Claude Code 2.1.296 transcript whose lines all fit their
/// recorded shapes: the lines a synthetic transcript repeats.
const RECORDED: &str = include_str!(
    "../../../testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/686aa14f-b535-45bc-9ed0-5400ea84bec7.jsonl"
);

/// The recall requests the search benchmark feeds the core: it
/// initializes, then asks `event_search` for a word nearly every line
/// holds, so the search ranks the newest 2,000 matches (NFR-03).
pub const SEARCH_REQUESTS: &str = concat!(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
    "\n",
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"event_search","arguments":{"query":"message","scope":"room","k":50}}}"#,
    "\n",
);

/// A transcript of `lines` lines: the recorded transcript's, repeated,
/// each copy's `uuid`s made its own, so no line repeats another's.
#[must_use]
pub fn transcript(lines: usize) -> String {
    let recorded: Vec<&str> = RECORDED.lines().collect();
    let mut out = String::new();
    for n in 0..lines {
        let copy = n / recorded.len().max(1);
        let line = recorded.get(n % recorded.len().max(1)).unwrap_or(&"");
        out.push_str(&line.replace("\"uuid\":\"", &format!("\"uuid\":\"c{copy}-")));
        out.push('\n');
    }

    out
}

/// A fresh home for one sample, removed when dropped.
#[derive(Debug)]
pub struct Home {
    root: PathBuf,
}

/// Counts the homes this process made, so no two share a name.
static MADE: AtomicU64 = AtomicU64::new(0);

impl Home {
    /// A fresh home under `base`, readable only by its owner, holding
    /// `transcript` under its default transcript root.
    ///
    /// # Errors
    ///
    /// Fails when the home or the transcript cannot be written.
    pub fn with_transcript(base: &Path, transcript: &str) -> io::Result<Self> {
        let n = MADE.fetch_add(1, Ordering::Relaxed);
        let root = base.join(format!("cairn-benchmark-{}-{n}", std::process::id()));
        let projects = root.join(".claude").join("projects").join("-w-bench");
        fs::create_dir_all(&projects)?;
        let home = Self { root };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&home.root, fs::Permissions::from_mode(0o700))?;
        }
        fs::write(projects.join("bench.jsonl"), transcript)?;

        Ok(home)
    }

    /// The variables the core starts with: `HOME` and `CAIRN_HOME`, both
    /// inside this home.
    #[must_use]
    pub fn env(&self) -> [(&'static str, PathBuf); 2] {
        [
            ("HOME", self.root.clone()),
            ("CAIRN_HOME", self.root.join(".cairn")),
        ]
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(test)]
mod tests;
