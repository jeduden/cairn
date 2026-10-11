use super::*;

#[test]
fn parse_reads_each_transcript_line() {
    let text = "a.jsonl lines=30 boundaries=0 last=system\nagent-x.jsonl lines=24 boundaries=1 last=attachment\n";

    let s = parse(text).unwrap();

    assert_eq!(
        s["a.jsonl"],
        Counts {
            lines: 30,
            boundaries: 0
        }
    );
    assert_eq!(
        s["agent-x.jsonl"],
        Counts {
            lines: 24,
            boundaries: 1
        }
    );
}

#[test]
fn parse_refuses_a_line_without_counts() {
    assert!(parse("a.jsonl lines=3").unwrap_err().contains("a.jsonl"));
    assert!(parse("a.jsonl lines=x boundaries=0").is_err());
}

#[test]
fn boundaries_sum_every_transcript() {
    let s = parse("a lines=1 boundaries=2\nb lines=1 boundaries=1\n").unwrap();

    assert_eq!(boundaries(&s), 3);
    assert_eq!(boundaries(&Snapshot::new()), 0);
}

fn hook(event: &str, source: &str, seen: usize) -> (String, String, Snapshot) {
    let snapshot = parse(&format!("t lines=1 boundaries={seen}")).unwrap();
    (event.into(), source.into(), snapshot)
}

#[test]
fn early_boundaries_accepts_a_boundary_written_after_the_hooks() {
    let hooks = [
        hook("SessionStart", "startup", 0),
        hook("PreCompact", "auto", 0),
        hook("SessionStart", "compact", 0),
        hook("PostCompact", "auto", 0),
        hook("PreCompact", "auto", 1),
        hook("SessionStart", "compact", 1),
        hook("PostCompact", "auto", 1),
    ];

    assert!(early_boundaries(&hooks).is_empty());
}

#[test]
fn early_boundaries_names_a_hook_that_saw_its_own_boundary() {
    let hooks = [
        hook("SessionStart", "compact", 1),
        hook("PostCompact", "auto", 1),
        hook("PostCompact", "auto", 0),
    ];

    assert_eq!(
        early_boundaries(&hooks),
        [
            "the SessionStart hook of compaction 1 saw 1 boundaries",
            "the PostCompact hook of compaction 1 saw 1 boundaries"
        ]
    );
}

#[test]
fn early_boundaries_ignores_a_post_compact_before_any_compaction() {
    assert!(early_boundaries(&[hook("PostCompact", "auto", 5)]).is_empty());
}
