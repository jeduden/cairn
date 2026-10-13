//! The crate against the real recordings under `testdata/claude-code`:
//! every version reads, every file its manifest names exists and
//! parses, and the tree holds nothing out of place.

use std::path::PathBuf;

use harness_contract::{measured, recording, transcript, transcript_tree};
use serde_json::Value;

fn root() -> PathBuf {
    testkit::repo_root().join("testdata/claude-code")
}

fn versions() -> Result<Vec<(PathBuf, Value)>, String> {
    recording::versions(&root())?
        .into_iter()
        .map(|dir| {
            let manifest = recording::json(&dir.join("manifest.json"))?;
            Ok((dir, manifest))
        })
        .collect()
}

/// Every path a manifest names: its strings, except its version, the
/// flow descriptions, the notes, the markers, and the probes' nonces and
/// answers.
fn named_paths(manifest: &Value) -> Vec<String> {
    let mut out = Vec::new();
    collect(manifest, "", &mut out);
    out
}

/// The manifest keys whose strings are not paths, matched as whole
/// segments of a key's dotted path.
const NOT_PATHS: [&str; 8] = [
    "claude_code",
    "hook_flows",
    "markers",
    "nonces",
    "answer",
    "answered_on",
    "sources",
    "notes",
];

fn collect(v: &Value, key: &str, out: &mut Vec<String>) {
    match v {
        Value::String(s) if !key.split('.').any(|segment| NOT_PATHS.contains(&segment)) => {
            out.push(s.clone());
        }
        Value::Array(a) => a.iter().for_each(|x| collect(x, key, out)),
        Value::Object(o) => o
            .iter()
            .for_each(|(k, x)| collect(x, &format!("{key}.{k}"), out)),
        _ => {}
    }
}

#[test]
fn a_supported_version_is_recorded() {
    let names: Vec<String> = versions()
        .unwrap()
        .iter()
        .map(|(d, _)| d.file_name().unwrap().to_string_lossy().into_owned())
        .collect();

    assert!(names.contains(&"2.1.296".to_owned()), "{names:?}");
}

#[test]
fn every_manifest_names_its_own_version() {
    for (dir, manifest) in versions().unwrap() {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(recording::named(&manifest, "claude_code").unwrap(), name);
    }
}

#[test]
fn every_path_a_manifest_names_exists() {
    for (dir, manifest) in versions().unwrap() {
        for rel in named_paths(&manifest) {
            let in_hooks = dir.join("hooks").join(&rel);
            assert!(
                dir.join(&rel).exists() || in_hooks.is_dir(),
                "{} names missing {rel}",
                dir.display()
            );
        }
    }
}

#[test]
fn every_described_flow_holds_hook_inputs() {
    for (dir, manifest) in versions().unwrap() {
        let flows = manifest["hook_flows"].as_object().unwrap();
        assert!(!flows.is_empty());
        for flow in flows.keys() {
            assert!(
                !recording::hook_inputs(&dir, flow).unwrap().is_empty(),
                "{flow}"
            );
        }
    }
}

#[test]
fn every_hook_input_is_named_for_its_hook() {
    for (dir, _) in versions().unwrap() {
        let inputs = recording::files(&dir.join("hooks"), ".json").unwrap();
        assert!(!inputs.is_empty());
        for path in inputs {
            let input = recording::json(&path).unwrap();
            let event = harness_contract::text(&input, "hook_event_name");
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            assert!(
                name.ends_with(&format!("-{event}.json")),
                "{name} holds {event}"
            );
        }
    }
}

#[test]
fn every_transcript_parses() {
    for (dir, _) in versions().unwrap() {
        let transcripts = recording::files(&dir.join("transcripts"), ".jsonl").unwrap();
        assert!(!transcripts.is_empty());
        for path in transcripts {
            assert!(
                !recording::lines(&path).unwrap().is_empty(),
                "{}",
                path.display()
            );
        }
    }
}

#[test]
fn the_tree_holds_nothing_out_of_place() {
    for (dir, manifest) in versions().unwrap() {
        let tree =
            transcript_tree::walk(&dir.join(recording::named(&manifest, "projects").unwrap()))
                .unwrap();

        assert!(tree.strays.is_empty(), "{:?}", tree.strays);
        assert!(transcript_tree::duplicates(&tree).is_empty());
        for s in &tree.harness_sessions {
            assert!(
                transcript_tree::harness_session_problems(s)
                    .unwrap()
                    .is_empty()
            );
        }
        for s in &tree.subagents {
            assert!(
                transcript_tree::subagent_problems(s, &tree)
                    .unwrap()
                    .is_empty()
            );
            assert!(transcript_tree::meta_problems(s).is_empty());
        }
    }
}

#[test]
fn the_measurements_parse() {
    for (dir, manifest) in versions().unwrap() {
        let timeouts =
            recording::json(&dir.join(recording::named(&manifest, "timeouts").unwrap())).unwrap();
        let cleanup =
            recording::json(&dir.join(recording::named(&manifest, "cleanup").unwrap())).unwrap();

        assert!(!measured::measurements(&timeouts).unwrap().is_empty());
        let (deleted, kept) = measured::cleanup(&cleanup).unwrap();
        assert!(!deleted.is_empty() && !kept.is_empty());
    }
}

#[test]
fn every_marker_appears_in_an_attachment() {
    for (dir, manifest) in versions().unwrap() {
        let path = dir.join(recording::named(&manifest, "harness_context.transcript").unwrap());
        let lines = recording::lines(&path).unwrap();
        for (what, marker) in recording::pairs(&manifest, "harness_context.markers").unwrap() {
            assert!(
                transcript::attachment_with(&lines, &marker).is_some(),
                "{what}"
            );
        }
    }
}

#[test]
fn the_hook_context_transcripts_hold_hook_context() {
    for (dir, manifest) in versions().unwrap() {
        let listed = manifest["hook_context"].as_array().unwrap();
        assert!(!listed.is_empty());
        for rel in listed {
            let lines = recording::lines(&dir.join(rel.as_str().unwrap())).unwrap();
            assert!(!transcript::hook_context_events(&lines).is_empty());
        }
    }
}

/// The lines of the blocked compaction the manifest names at `key`.
fn blocked(dir: &std::path::Path, manifest: &Value, key: &str) -> Result<Vec<Value>, String> {
    let rel = recording::named(manifest, &format!("compaction_blocked.{key}.transcript"))?;
    recording::lines(&dir.join(rel))
}

#[test]
fn a_blocked_manual_compaction_shows_as_command_output_and_is_answered() {
    for (dir, manifest) in versions().unwrap() {
        let lines = blocked(&dir, &manifest, "manual").unwrap();
        let shown = transcript::where_shown(&lines, "Compaction blocked by PreCompact hook");
        assert!(shown.contains(&transcript::Shown::CommandOutput));
        assert!(!transcript::has_compaction(&lines));
        assert!(transcript::refusals(&lines).is_empty());
        assert!(!transcript::model_replies_after(&lines, "Compaction blocked").is_empty());
    }
}

#[test]
fn the_full_context_window_ends_in_refusals_no_model_reply_follows() {
    for (dir, manifest) in versions().unwrap() {
        let lines = blocked(&dir, &manifest, "full_window").unwrap();
        assert!(!transcript::has_compaction(&lines));
        assert!(!transcript::refusals(&lines).is_empty());
        assert!(!transcript::model_replied_after_refusal(&lines));
    }
}
