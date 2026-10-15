//! What an event is and what produced its content: its event kind, and
//! its provenance class from PRV-01's closed set. Both sit in the
//! chained header; neither is content.

/// Which kind of entry an event is, recorded apart from its
/// provenance. The record grows this set as the plans land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventKind {
    /// A key event of a key event log (PRV-12).
    KeyEvent,
    /// A create room act (LANE-01).
    CreateRoom,
    /// A certificate record: a seat certificate's digest, recorded in a
    /// device seat (PRV-11, PRV-12).
    CertificateRecord,
    /// A transcript line recorded whole, before a parser reads it.
    UnparsedLine,
}

impl EventKind {
    /// The kind's name in a header.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeyEvent => "key_event",
            Self::CreateRoom => "create_room",
            Self::CertificateRecord => "certificate_record",
            Self::UnparsedLine => "unparsed_line",
        }
    }

    /// The kind `name` names.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        [
            Self::KeyEvent,
            Self::CreateRoom,
            Self::CertificateRecord,
            Self::UnparsedLine,
        ]
        .into_iter()
        .find(|k| k.as_str() == name)
    }
}

/// What produced an event's content: PRV-01's closed set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Provenance {
    User,
    Assistant,
    ToolCall,
    /// The result of the named tool.
    ToolResult(String),
    Web,
    /// What the named MCP server returned.
    Mcp(String),
    File,
    SubagentResult,
    HarnessMeta,
    HarnessText,
    Operator,
    Post,
    Summary,
    Structural,
    Unparsed,
}

/// The classes that carry no name, by their names.
const PLAIN: [(&str, Provenance); 13] = [
    ("user", Provenance::User),
    ("assistant", Provenance::Assistant),
    ("tool_call", Provenance::ToolCall),
    ("web", Provenance::Web),
    ("file", Provenance::File),
    ("subagent_result", Provenance::SubagentResult),
    ("harness_meta", Provenance::HarnessMeta),
    ("harness_text", Provenance::HarnessText),
    ("operator", Provenance::Operator),
    ("post", Provenance::Post),
    ("summary", Provenance::Summary),
    ("structural", Provenance::Structural),
    ("unparsed", Provenance::Unparsed),
];

impl Provenance {
    /// The class's name in a header: `tool_result:<tool>` and
    /// `mcp:<server>` carry their name.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::ToolResult(tool) => format!("tool_result:{tool}"),
            Self::Mcp(server) => format!("mcp:{server}"),
            plain => PLAIN
                .iter()
                .find(|(_, p)| p == plain)
                .map_or_else(String::new, |(n, _)| (*n).to_owned()),
        }
    }

    /// The class `name` names.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        let named = |prefix: &str| name.strip_prefix(prefix).filter(|n| !n.is_empty());
        if let Some(tool) = named("tool_result:") {
            return Some(Self::ToolResult(tool.to_owned()));
        }
        if let Some(server) = named("mcp:") {
            return Some(Self::Mcp(server.to_owned()));
        }

        PLAIN
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, p)| p.clone())
    }
}

#[cfg(test)]
mod tests;
