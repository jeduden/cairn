use record::digest::Digest;

use super::*;
use crate::entry::{Counter, Entry, Kind, Occurrence};

fn degraded(counter: Counter) -> Occurrence<'static> {
    Occurrence::Degraded {
        kind: Kind::Rejected,
        counter,
        operation: "hook UserPromptSubmit",
        detail: "not a JSON object",
    }
}

fn entries(occurrences: &[Occurrence]) -> Vec<Entry> {
    let mut prev = Digest::ZERO;
    let mut out = Vec::new();
    for (n, o) in (1..).zip(occurrences) {
        let e = Entry::next(o, n, 0, prev).unwrap();
        prev = e.hash;
        out.push(e);
    }
    out
}

#[test]
fn no_entry_counts_nothing() {
    let c = counts(&[], None);
    assert_eq!(c, Counts::default());
    assert!(!c.risen());
}

#[test]
fn each_counter_counts_the_entries_that_name_it() {
    let c = counts(
        &entries(&[
            degraded(Counter::HookRejected),
            degraded(Counter::HookPanicked),
            degraded(Counter::HookRejected),
        ]),
        None,
    );
    let want: BTreeMap<String, u64> =
        [("hook_panicked".into(), 1), ("hook_rejected".into(), 2)].into();
    assert_eq!(c.total, want);
    assert_eq!(c.since_ack, want);
    assert!(c.risen());
}

#[test]
fn an_acknowledgement_starts_the_rise_again_and_keeps_the_totals() {
    let both = entries(&[
        degraded(Counter::HookRejected),
        degraded(Counter::McpPanicked),
    ]);
    let c = counts(&both[..1], Some(1));
    assert_eq!(c.total, [("hook_rejected".into(), 1)].into());
    assert!(c.since_ack.is_empty());
    assert!(!c.risen());

    let c = counts(&both, Some(1));
    assert_eq!(c.total.len(), 2);
    assert_eq!(c.since_ack, [("mcp_panicked".into(), 1)].into());
    assert!(c.risen());

    let c = counts(&both, Some(0));
    assert_eq!(c.since_ack.len(), 2);
}
