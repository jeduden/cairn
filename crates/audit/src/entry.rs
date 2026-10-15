//! One audit entry (OPS-02): what happened, the counter it feeds, and
//! its place in the log's hash chain. Its line is the RFC 8785
//! encoding of its fields (§8.3); its hash is SHA-256 over the encoding
//! of every field but the hash, the previous entry's hash among them.

use record::canonical;
use record::digest::Digest;
use serde_json::{Map, Value, json};

use crate::AuditError;

/// The fields of an entry, the hash included; `counter` only on an
/// entry that feeds one.
const FIELDS: [&str; 8] = [
    "counter",
    "detail",
    "entry",
    "hash",
    "kind",
    "operation",
    "prev_hash",
    "time",
];

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
        match self {
            Self::Failed => "failed",
            Self::Dropped => "dropped",
            Self::Rejected => "rejected",
            Self::Redacted => "redacted",
            Self::TimedOut => "timed-out",
            Self::Truncated => "truncated",
            Self::Coalesced => "coalesced",
        }
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
        match self {
            Self::HookRejected => "hook_rejected",
            Self::HookPanicked => "hook_panicked",
            Self::McpPanicked => "mcp_panicked",
        }
    }
}

/// What one entry reports. Every field is text Cairn ships, so no
/// record content or secret can reach the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occurrence {
    Degraded {
        kind: Kind,
        counter: Counter,
        operation: &'static str,
        detail: &'static str,
    },
    /// The counters were acknowledged (`cairn counter ack`). Until
    /// phase 3 records that neutral principal act in the device seat,
    /// this entry is where the acknowledgement lives.
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
    /// The entry numbered `number` reporting `occurrence` at `time`
    /// (seconds since the Unix epoch), chained to `prev_hash`.
    ///
    /// # Errors
    ///
    /// Fails when `number` or `time` lies past the integers the
    /// canonical encoding takes.
    pub fn next(
        occurrence: &Occurrence,
        number: u64,
        time: u64,
        prev_hash: Digest,
    ) -> Result<Self, AuditError> {
        let (kind, counter, operation, detail) = match *occurrence {
            Occurrence::Degraded {
                kind,
                counter,
                operation,
                detail,
            } => (kind.as_str(), Some(counter.as_str()), operation, detail),
            Occurrence::CounterAck => (COUNTER_ACK, None, "counter ack", "counters acknowledged"),
        };
        let mut entry = Self {
            number,
            time,
            kind: kind.to_owned(),
            counter: counter.map(str::to_owned),
            operation: operation.to_owned(),
            detail: detail.to_owned(),
            prev_hash,
            hash: Digest::ZERO,
        };
        entry.hash = entry.digest()?;

        Ok(entry)
    }

    /// Its line: the canonical encoding of its fields, its hash among
    /// them, with no line break.
    ///
    /// # Errors
    ///
    /// Fails when it has no canonical encoding.
    pub fn line(&self) -> Result<String, AuditError> {
        let mut value = self.body();
        value.insert("hash".to_owned(), json!(self.hash.to_hex()));

        canonical::encode(&Value::Object(value)).map_err(AuditError::Encode)
    }

    /// The entry `line` holds, its hash checked.
    ///
    /// # Errors
    ///
    /// Fails, saying why, on a line that is not the canonical encoding
    /// of a whole entry, or whose hash does not match.
    pub fn parse(line: &str) -> Result<Self, &'static str> {
        let value: Value = serde_json::from_str(line).map_err(|_| "not JSON")?;
        let entry = Self::from_value(&value).ok_or("not an entry")?;
        if canonical::encode(&value).ok().as_deref() != Some(line) {
            return Err("not canonical JSON");
        }
        if entry.digest().ok() != Some(entry.hash) {
            return Err("hash does not match");
        }

        Ok(entry)
    }

    /// The entry `value` spells, every field present and typed, and no
    /// other.
    fn from_value(value: &Value) -> Option<Self> {
        let object = value.as_object()?;
        let fields = object.keys().all(|k| FIELDS.contains(&k.as_str()));
        let text = |k: &str| object.get(k)?.as_str().map(str::to_owned);
        let digest = |k: &str| Digest::from_hex(object.get(k)?.as_str()?).ok();
        let counter = match object.get("counter") {
            None => None,
            Some(c) => Some(c.as_str()?.to_owned()),
        };
        let number = object.get("entry")?.as_u64().filter(|n| *n > 0)?;

        fields.then_some(Self {
            number,
            time: object.get("time")?.as_u64()?,
            kind: text("kind")?,
            counter,
            operation: text("operation")?,
            detail: text("detail")?,
            prev_hash: digest("prev_hash")?,
            hash: digest("hash")?,
        })
    }

    /// Every field but the hash.
    fn body(&self) -> Map<String, Value> {
        let mut body = Map::new();
        body.insert("entry".to_owned(), json!(self.number));
        body.insert("time".to_owned(), json!(self.time));
        body.insert("kind".to_owned(), json!(self.kind));
        if let Some(counter) = &self.counter {
            body.insert("counter".to_owned(), json!(counter));
        }
        body.insert("operation".to_owned(), json!(self.operation));
        body.insert("detail".to_owned(), json!(self.detail));
        body.insert("prev_hash".to_owned(), json!(self.prev_hash.to_hex()));

        body
    }

    /// SHA-256 over the canonical encoding of every field but the hash.
    fn digest(&self) -> Result<Digest, AuditError> {
        let encoded = canonical::encode(&Value::Object(self.body())).map_err(AuditError::Encode)?;

        Ok(Digest::of(&[encoded.as_bytes()]))
    }
}

#[cfg(test)]
mod tests;
