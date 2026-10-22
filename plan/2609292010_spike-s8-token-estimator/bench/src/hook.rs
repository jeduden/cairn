//! Two hook handler shapes, each one process run, as the harness runs
//! a hook handler: read the hook input from stdin, open the store, do
//! the hook's work, write the answer, exit.
//!
//! - `UserPromptSubmit`: record the prompt and ingest the transcript
//!   lines written since the last hook, five here, in one transaction.
//! - `SessionStart`: record the start, read the room's pins and the
//!   run's newest 200 events, and build a restore block within
//!   INJ-07's 2,000 model tokens, measured with the S8 estimator.

use std::io::{Read, Write};
use std::path::Path;

use crate::estimate::estimate;
use crate::store::{self, Event, Result};

/// SEC-05's hook stdin limit.
const STDIN_LIMIT: u64 = 1 << 20;
/// Transcript lines one `UserPromptSubmit` ingests.
const TAIL: i64 = 5;
const PIN_BUDGET: u64 = 1_000;
const BLOCK_BUDGET: u64 = 2_000;

fn input() -> Result<serde_json::Value> {
    let mut buf = Vec::new();
    std::io::stdin().take(STDIN_LIMIT + 1).read_to_end(&mut buf)?;
    if buf.len() as u64 > STDIN_LIMIT {
        return Err("hook input above 1 MiB".into());
    }
    Ok(serde_json::from_slice(&buf)?)
}

pub fn user_prompt_submit(db: &Path, source: i64) -> Result<()> {
    let v = input()?;
    let prompt = v["prompt"].as_str().unwrap_or_default();
    let mut db = store::open(db)?;
    let tx = store::begin(&mut db)?;
    let mut line = store::next_line(&tx, source)?;
    store::append(&tx, &Event { source, line, kind: "user", tool: None, body: prompt })?;
    for i in 0..TAIL {
        line += 1;
        let body = format!("tool output line {i} for the prompt: {prompt}");
        store::append(&tx, &Event { source, line, kind: "tool_result", tool: Some("Bash"), body: &body })?;
    }
    tx.commit()?;
    db.close().map_err(|(_, e)| e)?;
    Ok(())
}

pub fn session_start(db: &Path, source: i64, room: &str) -> Result<()> {
    let v = input()?;
    let how = v["source"].as_str().unwrap_or("startup");
    let mut db = store::open(db)?;
    {
        let tx = store::begin(&mut db)?;
        let line = store::next_line(&tx, source)?;
        let body = format!("session start: {how}");
        store::append(&tx, &Event { source, line, kind: "session_start", tool: None, body: &body })?;
        tx.commit()?;
    }
    let mut block = String::from("<parinn-restore>\n");
    let mut used = estimate(&block);
    let mut st = db.prepare_cached(
        "SELECT id, body FROM pins WHERE room = ? AND active = 1 ORDER BY priority, id",
    )?;
    let pins = st.query_map([room], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    let mut pinned = 0;
    let mut omitted = 0;
    for p in pins {
        let (id, body) = p?;
        let entry = format!("pin {id}: {body}\n");
        let cost = estimate(&entry);
        if pinned + cost > PIN_BUDGET {
            omitted += 1;
            continue;
        }
        pinned += cost;
        block.push_str(&entry);
    }
    used += pinned;
    let mut st = db.prepare_cached(
        "SELECT seq, kind, tool, body FROM events WHERE source = ? AND generation = 0
         ORDER BY line DESC LIMIT 200",
    )?;
    let rows = st.query_map([source], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, String>(3)?))
    })?;
    for r in rows {
        let (seq, kind, tool, body) = r?;
        let head: String = body.chars().take(60).filter(|c| c.is_ascii_alphanumeric() || " ._/@:+-".contains(*c)).collect();
        let line = format!("- {seq} {kind} {} {head}\n", tool.as_deref().unwrap_or("-"));
        let cost = estimate(&line);
        if used + cost > BLOCK_BUDGET {
            break;
        }
        used += cost;
        block.push_str(&line);
    }
    block.push_str(&format!("{omitted} pins omitted\n</parinn-restore>\n"));
    let out = serde_json::json!({
        "hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": block}
    });
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{out}")?;
    Ok(())
}
