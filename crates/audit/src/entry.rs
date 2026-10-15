//! One audit entry (OPS-02): what happened, the counter it feeds, and
//! its place in the log's hash chain.

use record::digest::Digest;

use crate::AuditError;

/// The kind of the entry `cairn counter ack` appends.
pub const COUNTER_ACK: &str = "counter_ack";

/// How an operation degraded (OPS-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Failed,
    Dropped,
    Rejected,
    Redacted,
    TimedOut,
    Truncated,
    Coalesced,
}

impl Kind {
    /// Its name in an entry.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        ""
    }
}

/// A named counter (OPS-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counter {
    HookRejected,
    HookPanicked,
    McpPanicked,
}

impl Counter {
    /// Its name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        ""
    }
}

/// What one entry reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occurrence {
    Degraded {
        kind: Kind,
        counter: Counter,
        operation: &'static str,
        detail: &'static str,
    },
    CounterAck,
}

/// One entry of the audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub number: u64,
    pub time: u64,
    pub kind: String,
    pub counter: Option<String>,
    pub operation: String,
    pub detail: String,
    pub prev_hash: Digest,
    pub hash: Digest,
}

impl Entry {
    /// The entry numbered `number`.
    ///
    /// # Errors
    ///
    /// Fails when it has no canonical encoding.
    pub fn next(
        _occurrence: &Occurrence,
        _number: u64,
        _time: u64,
        _prev_hash: Digest,
    ) -> Result<Self, AuditError> {
        Err(AuditError::Broken {
            file: String::new(),
            line: 0,
            why: "",
        })
    }

    /// Its line.
    ///
    /// # Errors
    ///
    /// Fails when it has no canonical encoding.
    pub fn line(&self) -> Result<String, AuditError> {
        Ok(String::new())
    }

    /// The entry `line` holds.
    ///
    /// # Errors
    ///
    /// Fails, saying why, on a line that is not one.
    pub fn parse(_line: &str) -> Result<Self, &'static str> {
        Err("")
    }
}

#[cfg(test)]
mod tests;
