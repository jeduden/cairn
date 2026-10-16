//! A setting's value, as a settings layer gives it once validated: a
//! bool, an integer, one text or a list of texts. The record keeps the
//! accepted ones as JSON, inside the principal act that accepted them
//! (ADM-04).

use serde_json::json;

/// A validated setting's value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Bool(bool),
    Integer(i64),
    Text(String),
    Texts(Vec<String>),
}

impl Value {
    /// The value as JSON.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Bool(b) => json!(b),
            Self::Integer(i) => json!(i),
            Self::Text(t) => json!(t),
            Self::Texts(ts) => json!(ts),
        }
    }

    /// The value `json` spells, if it spells one.
    #[must_use]
    pub fn from_json(json: &serde_json::Value) -> Option<Self> {
        match json {
            serde_json::Value::Bool(b) => Some(Self::Bool(*b)),
            serde_json::Value::Number(n) => n.as_i64().map(Self::Integer),
            serde_json::Value::String(s) => Some(Self::Text(s.clone())),
            serde_json::Value::Array(items) => items
                .iter()
                .map(|i| i.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()
                .map(Self::Texts),
            serde_json::Value::Null | serde_json::Value::Object(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
