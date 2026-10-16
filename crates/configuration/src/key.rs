//! The settings keys (§9.6) and what each takes: its kind of value and
//! range, its default, what a repository configuration may do with it
//! (SEC-11), and which of its changes await acceptance (ADM-04).
//!
//! A settings key is `<concept>.<setting>`, or per room
//! `retention_policy.<room>.<provenance>`; `trust_policy.<provenance>`
//! is a key only so every settings layer can be told it changes
//! nothing (PRV-05).

use std::fmt;

use record::event::Provenance;
use record::id::RoomId;

use crate::value::Value;

/// What a key's value must be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bool,
    /// An integer from `min` to `max`.
    Integer {
        min: i64,
        max: i64,
    },
    /// One of the listed texts.
    Choice(&'static [&'static str]),
    /// A text of 1 to `max` characters.
    Text {
        max: usize,
    },
    /// A list of at most `items` texts, each of 1 to `max` characters.
    Texts {
        items: usize,
        max: usize,
    },
}

impl Kind {
    /// The value it wants, in words, for a type error.
    #[must_use]
    pub fn wants(self) -> &'static str {
        match self {
            Self::Bool => "want true or false",
            Self::Integer { .. } => "want an integer",
            Self::Choice(_) | Self::Text { .. } => "want a string",
            Self::Texts { .. } => "want an array of strings",
        }
    }
}

/// What a repository configuration may do with a key (SEC-11, §9.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repository {
    /// Nothing: any value of it is ignored.
    Never,
    /// Set it to any valid value.
    Any,
    /// Lower it, or leave it as it is.
    OnlyLower,
    /// Set it to this bool only.
    OnlyBool(bool),
    /// Set it to this choice only.
    OnlyChoice(&'static str),
}

/// Which changes of a key, from the value accepted last, await the
/// widening principal act that accepts the configuration (ADM-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acceptance {
    /// None: every change applies at once.
    Never,
    /// Every change, as of the deployment mode or a retention policy.
    AnyChange,
    /// Lowering it, as of the pin budget.
    Lowering,
    /// Turning it from true to false, as of flags or a required
    /// authenticator.
    TurningOff,
    /// Dropping an entry from the list, which loosens redaction.
    Dropping,
    /// Moving to a later choice, which loosens the recall scope.
    LaterChoice,
}

impl Acceptance {
    /// Whether changing `from` to `to` awaits acceptance.
    #[must_use]
    pub fn awaits(self, kind: Kind, from: Option<&Value>, to: Option<&Value>) -> bool {
        if from == to {
            return false;
        }
        match (self, from, to) {
            (Self::Never, ..) => false,
            (Self::AnyChange, ..) => true,
            (Self::Lowering, Some(Value::Integer(a)), Some(Value::Integer(b))) => b < a,
            (Self::TurningOff, Some(Value::Bool(true)), _) => true,
            (Self::Dropping, Some(Value::Texts(a)), to) => {
                let kept = match to {
                    Some(Value::Texts(b)) => b.as_slice(),
                    _ => &[],
                };
                a.iter().any(|item| !kept.contains(item))
            }
            (Self::LaterChoice, Some(Value::Text(a)), Some(Value::Text(b))) => {
                position(kind, b) > position(kind, a)
            }
            _ => false,
        }
    }
}

/// Where `choice` stands among `kind`'s choices; past them all when it
/// is none of them.
fn position(kind: Kind, choice: &str) -> usize {
    match kind {
        Kind::Choice(choices) => choices
            .iter()
            .position(|c| *c == choice)
            .unwrap_or(choices.len()),
        _ => 0,
    }
}

/// Everything a key takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spec {
    pub kind: Kind,
    /// Its value when no settings layer sets it; none for unset.
    pub default: Option<Default>,
    pub repository: Repository,
    pub acceptance: Acceptance,
}

/// A default value, written as a constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Default {
    Bool(bool),
    Integer(i64),
    Text(&'static str),
    Texts(&'static [&'static str]),
}

impl Default {
    /// The value it stands for.
    #[must_use]
    pub fn value(self) -> Value {
        match self {
            Self::Bool(b) => Value::Bool(b),
            Self::Integer(i) => Value::Integer(i),
            Self::Text(t) => Value::Text(t.to_owned()),
            Self::Texts(ts) => Value::Texts(ts.iter().map(|t| (*t).to_owned()).collect()),
        }
    }
}

/// The deployment modes (PRV-04), the default first.
pub const DEPLOYMENT_MODES: &[&str] = &["automation", "interactive"];

/// The recall scopes (RCL-05), narrowest first.
pub const RECALL_SCOPES: &[&str] = &["run", "room", "rooms"];

/// A token budget's range, in model tokens.
const TOKENS: Kind = Kind::Integer {
    min: 1,
    max: 100_000,
};

/// The keys with fixed names, and what each takes.
const FIXED: &[(&str, Spec)] = &[
    (
        "node.deployment_mode",
        spec(
            Kind::Choice(DEPLOYMENT_MODES),
            Some(Default::Text("automation")),
            Repository::Never,
            Acceptance::AnyChange,
        ),
    ),
    (
        "transcript.roots",
        spec(
            Kind::Texts {
                items: 64,
                max: 4096,
            },
            Some(Default::Texts(&["~/.claude/projects"])),
            Repository::Never,
            Acceptance::Never,
        ),
    ),
    (
        "home.id",
        spec(
            Kind::Text { max: 128 },
            None,
            Repository::Never,
            Acceptance::Never,
        ),
    ),
    (
        "payload.threshold_bytes",
        spec(
            Kind::Integer {
                min: 1,
                max: 16 << 20,
            },
            Some(Default::Integer(8192)),
            Repository::Any,
            Acceptance::Never,
        ),
    ),
    (
        "restore_block.landmarks_on_start",
        spec(
            Kind::Bool,
            Some(Default::Bool(false)),
            Repository::OnlyBool(false),
            Acceptance::Never,
        ),
    ),
    (
        "restore_block.on_prompt",
        spec(
            Kind::Bool,
            Some(Default::Bool(false)),
            Repository::OnlyBool(false),
            Acceptance::Never,
        ),
    ),
    (
        "restore_block.max_model_tokens",
        spec(
            TOKENS,
            Some(Default::Integer(2000)),
            Repository::OnlyLower,
            Acceptance::Never,
        ),
    ),
    (
        "pin_budget.max_model_tokens",
        spec(
            TOKENS,
            Some(Default::Integer(1000)),
            Repository::Never,
            Acceptance::Lowering,
        ),
    ),
    (
        "room_summary.max_model_tokens",
        spec(
            TOKENS,
            Some(Default::Integer(2000)),
            Repository::OnlyLower,
            Acceptance::Never,
        ),
    ),
    (
        "recall.max_k",
        spec(
            Kind::Integer { min: 1, max: 200 },
            Some(Default::Integer(50)),
            Repository::OnlyLower,
            Acceptance::Never,
        ),
    ),
    (
        "recall.default_scope",
        spec(
            Kind::Choice(RECALL_SCOPES),
            Some(Default::Text("run")),
            Repository::OnlyChoice("run"),
            Acceptance::LaterChoice,
        ),
    ),
    (
        "redaction.extra_patterns",
        spec(
            Kind::Texts {
                items: 256,
                max: 1024,
            },
            Some(Default::Texts(&[])),
            Repository::Never,
            Acceptance::Dropping,
        ),
    ),
    (
        "flag.enabled",
        spec(
            Kind::Bool,
            Some(Default::Bool(true)),
            Repository::OnlyBool(true),
            Acceptance::TurningOff,
        ),
    ),
    (
        "kernel.enabled",
        spec(
            Kind::Bool,
            Some(Default::Bool(true)),
            Repository::OnlyBool(false),
            Acceptance::Never,
        ),
    ),
    (
        "kernel.wall_seconds",
        spec(
            Kind::Integer { min: 1, max: 600 },
            Some(Default::Integer(10)),
            Repository::OnlyLower,
            Acceptance::Never,
        ),
    ),
    (
        "kernel.memory_mib",
        spec(
            Kind::Integer { min: 16, max: 8192 },
            Some(Default::Integer(512)),
            Repository::OnlyLower,
            Acceptance::Never,
        ),
    ),
    (
        "authenticator.required",
        spec(
            Kind::Bool,
            Some(Default::Bool(false)),
            // The principal or managed policy requires one (OWN-11); a
            // repository may not refuse the principal's widening acts.
            Repository::Never,
            Acceptance::TurningOff,
        ),
    ),
    (
        "risk_acceptance.forbidden",
        spec(
            Kind::Bool,
            Some(Default::Bool(false)),
            // Managed policy alone sets it (OWN-22, SEC-22).
            Repository::Never,
            Acceptance::TurningOff,
        ),
    ),
];

/// A retention policy's window, in days, kept forever when unset.
const RETENTION: Spec = spec(
    Kind::Integer {
        min: 1,
        max: 365_000,
    },
    None,
    Repository::Never,
    Acceptance::AnyChange,
);

/// A trust policy key: it is validated, then ignored in every layer.
const TRUST_POLICY: Spec = spec(
    Kind::Choice(&["trusted", "untrusted"]),
    None,
    Repository::Never,
    Acceptance::Never,
);

const fn spec(
    kind: Kind,
    default: Option<Default>,
    repository: Repository,
    acceptance: Acceptance,
) -> Spec {
    Spec {
        kind,
        default,
        repository,
        acceptance,
    }
}

/// A settings key that names a setting, with what it takes; keys
/// compare by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsKey {
    name: String,
    spec: Spec,
}

impl PartialOrd for SettingsKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SettingsKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl SettingsKey {
    /// Its name, dotted.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What it takes.
    #[must_use]
    pub fn spec(&self) -> Spec {
        self.spec
    }

    /// The provenance class it would trust, for a trust policy key.
    #[must_use]
    pub fn trust_policy(&self) -> Option<&str> {
        self.name.strip_prefix("trust_policy.")
    }

    /// Whether only managed policy may set it: forbidding risk
    /// acceptance (OWN-22, SEC-22).
    #[must_use]
    pub fn managed_policy_only(&self) -> bool {
        self.name == "risk_acceptance.forbidden"
    }

    /// Its default value; none when it is unset by default.
    #[must_use]
    pub fn default_value(&self) -> Option<Value> {
        self.spec().default.map(Default::value)
    }
}

impl fmt::Display for SettingsKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

/// The segments of a dotted key name. A provenance class or a room id
/// holds no dot, so the split is exact for every key that exists.
fn split(name: &str) -> Vec<&str> {
    name.split('.').collect()
}

/// What a path of key segments names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup {
    /// A settings key.
    Key(SettingsKey),
    /// A table some keys lie under.
    Table,
    /// Nothing.
    Unknown,
}

/// What `segments` names.
#[must_use]
pub fn lookup(segments: &[&str]) -> Lookup {
    let name = segments.join(".");
    if let Some((_, spec)) = FIXED.iter().find(|(n, _)| *n == name) {
        return Lookup::Key(SettingsKey { name, spec: *spec });
    }
    let table = |prefix: &str| {
        FIXED
            .iter()
            .any(|(n, _)| n.starts_with(&format!("{prefix}.")))
    };
    match segments {
        ["trust_policy"] | ["retention_policy"] => Lookup::Table,
        ["retention_policy", room] if is_room(room) => Lookup::Table,
        ["retention_policy", room, class] if is_room(room) && is_class(class) => {
            Lookup::Key(SettingsKey {
                name,
                spec: RETENTION,
            })
        }
        ["trust_policy", class] if is_class(class) => Lookup::Key(SettingsKey {
            name,
            spec: TRUST_POLICY,
        }),
        _ if !name.is_empty() && table(&name) => Lookup::Table,
        _ => Lookup::Unknown,
    }
}

/// The key `name` names, if it names one.
#[must_use]
pub fn key(name: &str) -> Option<SettingsKey> {
    match lookup(&split(name)) {
        Lookup::Key(key) => Some(key),
        Lookup::Table | Lookup::Unknown => None,
    }
}

/// Every key with a fixed name.
pub fn fixed() -> impl Iterator<Item = SettingsKey> {
    FIXED.iter().map(|(name, spec)| SettingsKey {
        name: (*name).to_owned(),
        spec: *spec,
    })
}

/// Whether `room` names a room in a retention policy: `*` for every
/// room, or a room id.
fn is_room(room: &str) -> bool {
    room == "*" || RoomId::parse(room).is_ok()
}

/// Whether `class` names a provenance class (PRV-01).
fn is_class(class: &str) -> bool {
    !class.contains('.') && Provenance::parse(class).is_some()
}

/// What stands for a name no settings key could spell.
pub const UNSPELLED: &str = "a key no settings key spells";

/// `name` when it holds only the characters a settings key or a
/// provenance class can, and no more of them than one may hold; none
/// otherwise. What a repository configuration names reaches a message
/// or an audit entry only through this, so it cannot put its own text
/// there (T5).
#[must_use]
pub fn spelled(name: &str) -> Option<&str> {
    let plain = name.len() <= 160
        && !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.:*-".contains(c));

    plain.then_some(name)
}

/// `name` as an error may show it: quoted when a settings key could
/// spell it, else [`UNSPELLED`].
#[must_use]
pub fn shown(name: &str) -> String {
    spelled(name).map_or_else(|| UNSPELLED.to_owned(), |n| format!("\"{n}\""))
}

#[cfg(test)]
mod tests;
