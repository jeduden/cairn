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
    /// A settings layer failed validation, and a recording command
    /// went on without it (ADM-04, I9).
    SettingsLayerInvalid,
    /// A repository configuration key that does more than tighten was
    /// ignored (SEC-11).
    RepositoryConfigurationKeyIgnored,
    /// A settings layer's trust policy key was ignored (PRV-05).
    TrustPolicyKeyIgnored,
    /// Managed policy is writable by the principal, or does not parse
    /// or validate (ADM-04).
    ManagedPolicyInvalid,
    /// A principal act was refused: no terminal (OWN-12), or a widening
    /// act OWN-22 or managed policy does not permit (SEC-22).
    PrincipalActRefused,
}

impl Counter {
    /// Its name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HookRejected => "hook_rejected",
            Self::HookPanicked => "hook_panicked",
            Self::McpPanicked => "mcp_panicked",
            Self::SettingsLayerInvalid => "settings_layer_invalid",
            Self::RepositoryConfigurationKeyIgnored => "repository_configuration_key_ignored",
            Self::TrustPolicyKeyIgnored => "trust_policy_key_ignored",
            Self::ManagedPolicyInvalid => "managed_policy_invalid",
            Self::PrincipalActRefused => "principal_act_refused",
        }
    }
}

/// What one entry reports. Every field is text Cairn ships, or a
/// settings key's name, so no record content or secret can reach the
/// log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occurrence<'a> {
    Degraded {
        kind: Kind,
        counter: Counter,
        operation: &'static str,
        detail: &'static str,
    },
    /// An operation degraded over one settings key: its detail is
    /// `detail` followed by `key`, a settings key's name or a
    /// provenance class, shown only when it holds nothing but the
    /// characters those can.
    Keyed {
        kind: Kind,
        counter: Counter,
        operation: &'static str,
        detail: &'static str,
        key: &'a str,
    },
    /// A principal act was taken, such as acknowledging the counters;
    /// it feeds no counter.
    PrincipalAct {
        operation: &'static str,
        detail: &'static str,
    },
}

/// The kind of an entry noting a principal act.
pub const PRINCIPAL_ACT: &str = "principal_act";

/// `key` as an entry may hold it: as it is when it holds only the
/// characters a settings key or a provenance class can, else a fixed
/// phrase.
fn shown(key: &str) -> &str {
    let plain = !key.is_empty()
        && key.len() <= 160
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.:*-".contains(c));
    if plain {
        key
    } else {
        "a key no settings key spells"
    }
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
        occurrence: &Occurrence<'_>,
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
            } => (kind, counter, operation, detail.to_owned()),
            Occurrence::Keyed {
                kind,
                counter,
                operation,
                detail,
                key,
            } => (kind, counter, operation, format!("{detail} {}", shown(key))),
            Occurrence::PrincipalAct { operation, detail } => {
                return Self::noted(PRINCIPAL_ACT, operation, detail, number, time, prev_hash);
            }
        };
        let mut entry = Self {
            number,
            time,
            kind: kind.as_str().to_owned(),
            counter: Some(counter.as_str().to_owned()),
            operation: operation.to_owned(),
            detail,
            prev_hash,
            hash: Digest::ZERO,
        };
        entry.hash = entry.digest()?;

        Ok(entry)
    }

    /// An entry of `kind` feeding no counter.
    fn noted(
        kind: &str,
        operation: &str,
        detail: &str,
        number: u64,
        time: u64,
        prev_hash: Digest,
    ) -> Result<Self, AuditError> {
        let mut entry = Self {
            number,
            time,
            kind: kind.to_owned(),
            counter: None,
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
