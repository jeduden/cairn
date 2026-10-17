//! Which run each transcript line belongs to (REC-02, REC-04): a
//! transcript is split by the harness's agent id, a line carrying none
//! belonging to the run of the transcript's first entry; each run's
//! harness session is its lines' `sessionId`, else the transcript's.
//! Attribution comes from the transcript alone, never a hook input.

use serde_json::Value;

/// The run a line belongs to, within its transcript source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RunKey {
    /// The harness's agent id; none for a harness session's main run.
    pub agent_id: Option<String>,
    pub harness_session: String,
}

/// What a transcript's lines that name no run fall back to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Defaults {
    agent_id: Option<String>,
    harness_session: String,
}

impl Defaults {
    /// The defaults of the transcript whose parsed lines are `lines`,
    /// named `stem`: the agent id of its first entry (a line carrying a
    /// `uuid`), and the first `sessionId` any line carries, else `stem`.
    #[must_use]
    pub fn of(lines: &[Value], stem: &str) -> Self {
        let agent_id = lines
            .iter()
            .find(|l| l["uuid"].is_string())
            .and_then(|l| text(l, "agentId"));
        let harness_session = lines
            .iter()
            .find_map(|l| text(l, "sessionId"))
            .unwrap_or_else(|| stem.to_owned());

        Self {
            agent_id,
            harness_session,
        }
    }

    /// The transcript's own run: the one its defaults name.
    #[must_use]
    pub fn own(&self) -> RunKey {
        RunKey {
            agent_id: self.agent_id.clone(),
            harness_session: self.harness_session.clone(),
        }
    }

    /// The run the parsed line `line` belongs to.
    #[must_use]
    pub fn key(&self, line: &Value) -> RunKey {
        RunKey {
            agent_id: text(line, "agentId").or_else(|| self.agent_id.clone()),
            harness_session: text(line, "sessionId")
                .unwrap_or_else(|| self.harness_session.clone()),
        }
    }
}

/// The non-empty string at `field` of `line`.
fn text(line: &Value, field: &str) -> Option<String> {
    line[field]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests;
