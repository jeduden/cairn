use std::collections::BTreeSet;

use super::*;

#[test]
fn every_case_has_a_unique_name_a_guard_and_a_message() {
    let cases = cases();
    let names: BTreeSet<&str> = cases.iter().map(|c| c.name).collect();

    assert_eq!(names.len(), cases.len(), "two cases share a name");
    for c in &cases {
        assert!(!c.guards.is_empty() && !c.want.is_empty(), "{}", c.name);
    }
}

#[test]
fn only_the_srs_citing_a_missing_adr_is_a_known_gap() {
    let gaps: Vec<&str> = cases()
        .iter()
        .filter(|c| c.known_gap)
        .map(|c| c.name)
        .collect();

    assert_eq!(gaps, ["the SRS citing an ADR with no file"]);
}
