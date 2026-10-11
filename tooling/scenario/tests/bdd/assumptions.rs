//! Steps of `features/assumptions.feature`, SRS §2.3's contract tests
//! (ENG-17): each replays what the M0 spikes recorded from Claude Code
//! under `testdata/claude-code/<version>/`, for every recorded version,
//! through the `harness-contract` checks. A version's `manifest.json`
//! names its recordings by role.

use std::fs;
use std::path::{Path, PathBuf};

use cucumber::{given, then, when};
use engineering::outcome;
use harness_contract::measured::{self, Measurement};
use harness_contract::transcript_tree::{self, Tree};
use harness_contract::{hook_input, recording, text, transcript};
use serde_json::Value;

use crate::World;

/// What the §2.3 scenarios track beside the shared world, one entry per
/// recorded Claude Code version.
#[derive(Debug, Default)]
pub struct Assumptions {
    versions: Vec<Version>,
    /// The hook flow a step named.
    flow: String,
    /// The working directories a step named.
    cwds: Vec<String>,
}

/// One recorded Claude Code version and what a step read from it.
#[derive(Debug, Default)]
struct Version {
    dir: PathBuf,
    manifest: Value,
    /// Transcripts a step chose, by path; their lines once read.
    transcripts: Vec<PathBuf>,
    lines: Vec<Value>,
    /// The hook inputs a step selected.
    inputs: Vec<Value>,
    tree: Tree,
    /// A recording before and after a flow, by bytes and by lines.
    before: (Vec<u8>, Vec<Value>),
    after: (Vec<u8>, Vec<Value>),
    measurements: Vec<Measurement>,
    timeouts: Value,
    cleanup: (Vec<measured::Aged>, Vec<measured::Aged>),
    cleanup_doc: Value,
}

impl Version {
    fn name(&self) -> String {
        self.dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// The path the manifest gives at `key`, inside this version.
    fn path(&self, key: &str) -> Result<PathBuf, String> {
        let rel =
            recording::named(&self.manifest, key).map_err(|e| format!("{}: {e}", self.name()))?;
        Ok(self.dir.join(rel))
    }

    fn read_transcripts(&mut self) -> Result<(), String> {
        self.lines.clear();
        for path in &self.transcripts {
            self.lines.extend(recording::lines(path)?);
        }
        Ok(())
    }
}

fn asm(w: &mut World) -> &mut Assumptions {
    w.section::<Assumptions>()
}

/// Runs `check` on every recorded version, naming the version in each
/// problem.
fn each(
    w: &mut World,
    mut check: impl FnMut(&mut Version) -> Result<Vec<String>, String>,
) -> Result<(), String> {
    let mut problems = Vec::new();
    for v in &mut asm(w).versions {
        let name = v.name();
        problems.extend(check(v)?.into_iter().map(|p| format!("{name}: {p}")));
    }
    outcome(problems)
}

fn open_versions(w: &mut World, version: &str) -> Result<(), String> {
    if version != "supported" {
        return Err(format!(
            "only the \"supported\" versions are recorded, not {version:?}"
        ));
    }
    let root = testkit::repo_root().join("testdata/claude-code");
    asm(w).versions = recording::versions(&root)?
        .into_iter()
        .map(|dir| {
            let manifest = recording::json(&dir.join("manifest.json"))?;
            Ok(Version {
                dir,
                manifest,
                ..Version::default()
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(())
}

fn none_unless(found: bool, what: impl FnOnce() -> String) -> Vec<String> {
    if found { Vec::new() } else { vec![what()] }
}

// ASM-01: hook inputs.

#[given(regex = r#"^the hook inputs recorded from Claude Code "([^"]+)" in the "([^"]+)" flow$"#)]
fn hook_inputs_of_a_flow(w: &mut World, version: String, flow: String) -> Result<(), String> {
    open_versions(w, &version)?;
    asm(w).flow = flow;
    Ok(())
}

#[when(regex = r#"^the "([^"]+)" hook inputs are read$"#)]
fn hook_inputs_are_read(w: &mut World, event: String) -> Result<(), String> {
    let flow = asm(w).flow.clone();
    each(w, |v| {
        let all = recording::hook_inputs(&v.dir, &flow)?;
        v.inputs = hook_input::of_event(&all, &event)
            .into_iter()
            .cloned()
            .collect();
        Ok(none_unless(!v.inputs.is_empty(), || {
            format!("the {flow} flow recorded no {event} hook input")
        }))
    })
}

#[then(regex = r"^each carries (.+)$")]
fn each_carries(w: &mut World, fields: String) -> Result<(), String> {
    let fields = hook_input::names(&fields)?;
    let fields: Vec<&str> = fields.iter().map(String::as_str).collect();
    each(w, |v| {
        Ok(v.inputs
            .iter()
            .flat_map(|i| hook_input::missing(i, &fields))
            .collect())
    })
}

#[then(regex = r"^the last carries (.+)$")]
fn the_last_carries(w: &mut World, extra: String) -> Result<(), String> {
    let expects = hook_input::parse(&extra)?;
    each(w, |v| {
        Ok(v.inputs
            .last()
            .map(|i| hook_input::unmet(i, &expects))
            .unwrap_or_default())
    })
}

#[then(regex = r#"^the last one's "session_id" is (the same as|other than) the "([^"]+)" flow's$"#)]
fn the_last_ones_harness_session(
    w: &mut World,
    relation: String,
    flow: String,
) -> Result<(), String> {
    each(w, |v| {
        let first = recording::hook_inputs(&v.dir, &flow)?;
        let theirs = first
            .first()
            .map(|i| text(i, "session_id"))
            .unwrap_or_default();
        let ours = v
            .inputs
            .last()
            .map(|i| text(i, "session_id"))
            .unwrap_or_default();
        let same = ours == theirs;
        Ok(none_unless(same == (relation == "the same as"), || {
            format!("session_id {ours} is not {relation} the {flow} flow's {theirs}")
        }))
    })
}

// ASM-02: additional context.

#[given(
    regex = r#"^the transcripts recorded from Claude Code "([^"]+)" with every hook answering "additionalContext"$"#
)]
fn transcripts_answering_context(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        let list = v.manifest.get("hook_context").and_then(Value::as_array);
        v.transcripts = list
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|rel| v.dir.join(rel))
            .collect();
        Ok(none_unless(!v.transcripts.is_empty(), || {
            "the manifest names no hook_context".into()
        }))
    })
}

#[when(regex = r"^each line of the recordings? (?:is|are) read$")]
fn each_line_is_read(w: &mut World) -> Result<(), String> {
    each(w, |v| v.read_transcripts().map(|()| Vec::new()))
}

#[then(
    regex = r#"^the "([^"]+)" hook's additional context (shows|never shows) as a hook context attachment$"#
)]
fn context_shows(w: &mut World, event: String, shows: String) -> Result<(), String> {
    each(w, |v| {
        let found = transcript::hook_context_events(&v.lines).contains(&event);
        Ok(none_unless(found == (shows == "shows"), || {
            let seen = if found { "shows" } else { "never shows" };
            format!("the {event} hook's additional context {seen} as an attachment")
        }))
    })
}

#[then(
    regex = r#"^the "([^"]+)" hook's output (shows|never shows) in a compact command's output$"#
)]
fn output_echoed(w: &mut World, event: String, shows: String) -> Result<(), String> {
    each(w, |v| {
        let found = transcript::echoed_hooks(&v.lines).contains(&event);
        Ok(none_unless(found == (shows == "shows"), || {
            let seen = if found { "shows" } else { "never shows" };
            format!("the {event} hook's output {seen} in a compact command's output")
        }))
    })
}

// ASM-03 and ASM-16: the tree of transcripts.

#[given(regex = r#"^a recorded "~/.claude/projects" tree for Claude Code "([^"]+)"$"#)]
fn a_recorded_transcript_tree(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)
}

#[given(
    regex = r#"^a recorded "~/.claude/projects" tree for Claude Code "([^"]+)" after one harness session in each of (.+)$"#
)]
fn a_tree_after_harness_sessions(
    w: &mut World,
    version: String,
    cwds: String,
) -> Result<(), String> {
    open_versions(w, &version)?;
    asm(w).cwds = hook_input::names(&cwds)?;
    Ok(())
}

#[when(regex = r"^the recorded tree is walked$")]
fn the_tree_is_walked(w: &mut World) -> Result<(), String> {
    let by_cwd = !asm(w).cwds.is_empty();
    each(w, |v| {
        if by_cwd {
            let flow = recording::named(&v.manifest, "directories")?.to_owned();
            v.inputs = recording::hook_inputs(&v.dir, &flow)?;
        }
        v.tree = transcript_tree::walk(&v.path("projects")?)?;
        Ok(Vec::new())
    })
}

#[then(regex = r#"^each harness session yields one JSONL transcript under "[^"]+"$"#)]
fn one_transcript_per_harness_session(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let mut problems = none_unless(!v.tree.harness_sessions.is_empty(), || {
            "no harness session".into()
        });
        for s in &v.tree.harness_sessions {
            problems.extend(transcript_tree::harness_session_problems(s)?);
        }
        problems.extend(transcript_tree::duplicates(&v.tree));
        problems.extend(
            v.tree
                .strays
                .iter()
                .map(|p| format!("{} lies outside the layout", p.display())),
        );
        Ok(problems)
    })
}

#[then(regex = r#"^each subagent transcript lies under "[^"]+"$"#)]
fn subagents_nested(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let mut problems = none_unless(!v.tree.subagents.is_empty(), || {
            "no subagent transcript".into()
        });
        for s in &v.tree.subagents {
            problems.extend(transcript_tree::subagent_problems(s, &v.tree)?);
        }
        Ok(problems)
    })
}

#[then(
    regex = r#"^each subagent transcript has a "\.meta\.json" beside it naming its agent type$"#
)]
fn subagents_have_meta(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        Ok(v.tree
            .subagents
            .iter()
            .flat_map(transcript_tree::meta_problems)
            .collect())
    })
}

#[then(regex = r#"^the three harness sessions share the transcript directory "([^"]+)"$"#)]
fn harness_sessions_share_a_directory(w: &mut World, dir: String) -> Result<(), String> {
    let cwds = asm(w).cwds.clone();
    each(w, |v| {
        Ok(transcript_tree::shared_directory(
            &v.inputs, &cwds, &dir, &v.tree,
        ))
    })
}

// ASM-04: appends.

#[given(
    regex = r#"^a recorded transcript before and after a "([^"]+)" flow for Claude Code "([^"]+)"$"#
)]
fn before_and_after(w: &mut World, flow: String, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    asm(w).flow = flow;
    Ok(())
}

#[when(regex = r"^the two recordings are compared$")]
fn recordings_compared(w: &mut World) -> Result<(), String> {
    let flow = asm(w).flow.clone();
    each(w, |v| {
        let flow = v
            .manifest
            .get("appends")
            .and_then(|a| a.get(&flow))
            .cloned()
            .ok_or_else(|| format!("the manifest names no {flow} recording"))?;
        v.before = read_both(&v.dir.join(text(&flow, "before")))?;
        v.after = read_both(&v.dir.join(text(&flow, "after")))?;
        Ok(Vec::new())
    })
}

fn read_both(path: &Path) -> Result<(Vec<u8>, Vec<Value>), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok((bytes, recording::lines(path)?))
}

#[then(regex = r"^the earlier recording is a byte prefix of the later one$")]
fn byte_prefix(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let grew = v.after.0.len() > v.before.0.len();
        Ok(none_unless(
            grew && v.after.0.starts_with(&v.before.0),
            || "the earlier recording is not a byte prefix of the later one".into(),
        ))
    })
}

#[then(regex = r"^the first line appended continues from a line before the earlier one's last$")]
fn appended_branches(w: &mut World) -> Result<(), String> {
    each(w, |v| Ok(transcript::branched(&v.before.1, &v.after.1)))
}

#[then(regex = r#"^a "([^"]+)" line is appended$"#)]
fn line_appended(w: &mut World, kind: String) -> Result<(), String> {
    each(w, |v| {
        let found = transcript::appended_types(&v.before.1, &v.after.1).contains(&kind);
        Ok(none_unless(found, || {
            format!("no {kind} line was appended")
        }))
    })
}

// ASM-05: cleanup.

#[given(
    regex = r#"^a recorded "~/.claude/projects" tree before and after harness starts with "cleanupPeriodDays" of (\d+)$"#
)]
fn tree_before_after_cleanup(w: &mut World, days: u32) -> Result<(), String> {
    open_versions(w, "supported")?;
    each(w, |v| {
        v.cleanup_doc = recording::json(&v.path("cleanup")?)?;
        let period = v
            .cleanup_doc
            .get("cleanup_period_days")
            .and_then(Value::as_u64);
        Ok(none_unless(period == Some(u64::from(days)), || {
            format!("the cleanup was recorded with a period of {period:?} days, not {days}")
        }))
    })
}

#[when(regex = r"^the two trees are compared$")]
fn trees_compared(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        v.cleanup = measured::cleanup(&v.cleanup_doc)?;
        Ok(Vec::new())
    })
}

#[then(regex = r"^a transcript older than (\d+) days was (deleted|kept)$")]
fn old_transcript(w: &mut World, days: f64, fate: String) -> Result<(), String> {
    each(w, |v| {
        let list = if fate == "deleted" {
            &v.cleanup.0
        } else {
            &v.cleanup.1
        };
        let found = list.iter().any(|a| a.age_days > days);
        Ok(none_unless(found, || {
            format!("no transcript older than {days} days was {fate}")
        }))
    })
}

#[then(regex = r"^no transcript younger than (\d+) days was deleted$")]
fn young_kept(w: &mut World, days: f64) -> Result<(), String> {
    each(w, |v| {
        Ok(v.cleanup
            .0
            .iter()
            .filter(|a| a.age_days < days)
            .map(|a| format!("{} was deleted at {} days", a.path, a.age_days))
            .collect())
    })
}

// ASM-06: subagent compaction.

#[given(regex = r#"^a recorded hook input for Claude Code "([^"]+)"$"#)]
fn a_recorded_hook_input(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)
}

#[given(regex = r"^the hook input was captured during a subagent's compaction$")]
fn during_subagent_compaction(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let flow = recording::named(&v.manifest, "subagent_compaction.hooks")?.to_owned();
        v.inputs = recording::hook_inputs(&v.dir, &flow)?;
        Ok(none_unless(
            hook_input::ordered(&v.inputs, "SubagentStart", "PreCompact"),
            || format!("the {flow} flow holds no PreCompact after a SubagentStart"),
        ))
    })
}

#[when(regex = r#"^the hook "([^"]+)" runs with the recorded hook input$"#)]
fn the_hook_runs(w: &mut World, event: String) -> Result<(), String> {
    each(w, |v| {
        let all = std::mem::take(&mut v.inputs);
        let parent = all
            .iter()
            .find(|i| text(i, "hook_event_name") == "SessionStart")
            .cloned()
            .ok_or("the flow holds no SessionStart of the main agent")?;
        v.inputs = std::iter::once(parent)
            .chain(hook_input::of_event(&all, &event).into_iter().cloned())
            .collect();
        Ok(none_unless(v.inputs.len() > 1, || {
            format!("the flow holds no {event} hook input")
        }))
    })
}

#[then(
    regex = r#"^its "session_id" and "transcript_path" are the main agent's harness session's$"#
)]
fn the_main_agents_identity(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let Some((main, rest)) = v.inputs.split_first() else {
            return Ok(vec!["no hook input".into()]);
        };
        let rest: Vec<&Value> = rest.iter().collect();
        Ok(hook_input::identity_mismatches(
            main,
            &rest,
            &["session_id", "transcript_path"],
        ))
    })
}

#[then(regex = r"^it carries no subagent-specific field$")]
fn no_subagent_field(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        Ok(v.inputs
            .iter()
            .skip(1)
            .flat_map(hook_input::subagent_fields)
            .collect())
    })
}

#[then(
    regex = r#"^the additional context of the "SessionStart" hook that follows it lands in the subagent's transcript, where the model reads it$"#
)]
fn session_start_context_lands_in_subagent(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let main = recording::lines(&v.path("subagent_compaction.main_transcript")?)?;
        let subagent = recording::lines(&v.path("subagent_compaction.subagent_transcript")?)?;
        let mut problems = Vec::new();
        if transcript::has_compaction(&main) {
            problems.push("the main agent's transcript holds a compaction".to_owned());
        }
        if !transcript::context_after_compaction(&subagent, "SessionStart") {
            problems.push(
                "no SessionStart context follows the subagent's compaction in its transcript"
                    .into(),
            );
        }
        Ok(problems)
    })
}

// ASM-10: harness timeouts.

#[given(regex = r#"^the hook timeouts measured on Claude Code "([^"]+)"$"#)]
fn timeouts_measured(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        v.timeouts = recording::json(&v.path("timeouts")?)?;
        v.measurements = measured::measurements(&v.timeouts)?;
        Ok(Vec::new())
    })
}

#[when(
    regex = r#"^the "([^"]+)" handler sleeps past its harness timeout in (print|interactive) mode with (?:a (\d+) s timeout|no timeout)$"#
)]
fn handler_sleeps(
    w: &mut World,
    event: String,
    mode: String,
    timeout: String,
) -> Result<(), String> {
    let timeout = if timeout.is_empty() {
        None
    } else {
        Some(timeout.parse::<f64>().map_err(|e| e.to_string())?)
    };
    asm(w).flow = format!("{event}/{mode}");
    each(w, |v| {
        let picked: Vec<Measurement> = measured::matching(&v.measurements, &event, &mode, timeout)
            .into_iter()
            .cloned()
            .collect();
        let found = !picked.is_empty();
        v.measurements = picked;
        Ok(none_unless(found, || {
            format!("no {event} measurement in {mode} mode with {timeout:?} s")
        }))
    })
}

#[then(regex = r"^the harness terminated the handler after about ([\d.]+) s$")]
fn terminated_after(w: &mut World, limit: f64) -> Result<(), String> {
    each(w, |v| {
        Ok(v.measurements
            .iter()
            .filter(|m| m.finished || !measured::about(m.ran_ms, limit))
            .map(|m| {
                format!(
                    "run {}: {} ran {} ms, not about {limit} s",
                    m.trial, m.event, m.ran_ms
                )
            })
            .collect())
    })
}

#[then(
    regex = r#"^no "SessionEnd" handler started in a print-mode harness session whose "Stop" handler reached its harness timeout$"#
)]
fn session_end_skipped(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let (checked, mut problems) = measured::session_end_after_stop_timeout(&v.timeouts)?;
        if checked == 0 {
            problems.push("no print-mode trial timed out its Stop handler".into());
        }
        Ok(problems)
    })
}

// ASM-11 to ASM-15: transcript lines.

#[given(
    regex = r#"^a recorded transcript for Claude Code "([^"]+)" (?:from a worktree with a CLAUDE\.md file, an MCP server and a skill|with an assistant reply carrying two tool calls|with a Bash tool call and its tool result)$"#
)]
fn the_harness_context_transcript(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        v.transcripts = vec![v.path("harness_context.transcript")?];
        Ok(Vec::new())
    })
}

#[given(
    regex = r#"^a recorded transcript for Claude Code "([^"]+)"(?: with extended thinking enabled)?$"#
)]
fn every_transcript(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        v.transcripts = recording::files(&v.dir.join("transcripts"), ".jsonl")?;
        Ok(none_unless(!v.transcripts.is_empty(), || {
            "no transcript".into()
        }))
    })
}

#[then(
    regex = r#"^the CLAUDE\.md file, the MCP server instructions and the skill listing appear as "attachment" lines$"#
)]
fn harness_context_attached(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let markers = harness_context_markers(&v.manifest)?;
        Ok(markers
            .iter()
            .filter(|(_, m)| transcript::attachment_with(&v.lines, m).is_none())
            .map(|(what, _)| format!("the {what} appears in no attachment line"))
            .collect())
    })
}

#[then(regex = r#"^each of those lines carries "attachment\.type" and "renderedRole"$"#)]
fn attachments_carry_type_and_role(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let markers = harness_context_markers(&v.manifest)?;
        Ok(markers
            .iter()
            .filter_map(|(what, m)| transcript::attachment_with(&v.lines, m).map(|l| (what, l)))
            .filter(|(_, l)| {
                text(l, "attachment.type").is_empty() || text(l, "renderedRole").is_empty()
            })
            .map(|(what, _)| format!("the {what} line lacks attachment.type or renderedRole"))
            .collect())
    })
}

fn harness_context_markers(manifest: &Value) -> Result<Vec<(String, String)>, String> {
    recording::pairs(manifest, "harness_context.markers")
}

#[then(regex = r"^each assistant line carries exactly one content block$")]
fn one_block_per_line(w: &mut World) -> Result<(), String> {
    each(w, |v| Ok(transcript::multi_block(&v.lines)))
}

#[then(regex = r#"^the reply's lines share one "message\.id" and repeat its "usage"$"#)]
fn lines_share_message(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let mut problems = none_unless(transcript::spans_tool_calls(&v.lines, 2), || {
            "no message spans two tool-call lines".into()
        });
        problems.extend(transcript::usage_spread(&v.lines));
        Ok(problems)
    })
}

#[then(
    regex = r#"^the tool-result line carries the output in "message\.content" and again in "toolUseResult"$"#
)]
fn result_repeats_output(w: &mut World) -> Result<(), String> {
    tool_result_problems(w, false)
}

#[then(regex = r#"^its "sourceToolAssistantUUID" names the line carrying the Bash tool call$"#)]
fn result_names_its_call(w: &mut World) -> Result<(), String> {
    tool_result_problems(w, true)
}

fn tool_result_problems(w: &mut World, source: bool) -> Result<(), String> {
    each(w, |v| {
        let found = transcript::tool_results(&v.lines, "Bash");
        let mut problems = if source { found.source } else { found.output };
        if found.checked == 0 {
            problems.push("no Bash tool result".into());
        }
        Ok(problems)
    })
}

#[then(
    regex = r#"^every line of type "([^"]+)" carries (neither "uuid" nor "timestamp"|no "uuid")$"#
)]
fn line_type_lacks_keys(w: &mut World, kind: String, missing: String) -> Result<(), String> {
    let both = missing.starts_with("neither");
    each(w, |v| {
        let (n, uuid, timestamp) = transcript::keys_by_type(&v.lines, &kind);
        let mut problems = none_unless(n > 0, || format!("no line of type {kind}"));
        if uuid > 0 {
            problems.push(format!("{uuid} of {n} {kind} lines carry a uuid"));
        }
        if both && timestamp > 0 {
            problems.push(format!("{timestamp} of {n} {kind} lines carry a timestamp"));
        }
        Ok(problems)
    })
}

#[then(regex = r"^every thinking block carries a signature and empty thinking text$")]
fn thinking_is_signed_and_empty(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let (n, mut problems) = transcript::thinking(&v.lines);
        if n == 0 {
            problems.push("no thinking block".into());
        }
        Ok(problems)
    })
}
