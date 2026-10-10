use super::*;

const REPORT: &str = "SF:/w/a/src/lib.rs
FN:1,f
DA:1,3
DA:2,0
LF:2
end_of_record
SF:/w/b/src/lib.rs
DA:5,1
end_of_record
";

fn lines(entries: &[(&str, &[(u32, u64)])]) -> Lines {
    entries
        .iter()
        .map(|(f, l)| (PathBuf::from(f), l.iter().copied().collect()))
        .collect()
}

#[test]
fn parse_reads_the_hits_of_each_line_per_file() {
    assert_eq!(
        parse(REPORT).unwrap(),
        lines(&[
            ("/w/a/src/lib.rs", &[(1, 3), (2, 0)]),
            ("/w/b/src/lib.rs", &[(5, 1)])
        ])
    );
}

#[test]
fn parse_adds_up_a_line_reported_twice() {
    assert_eq!(
        parse("SF:a\nDA:1,2\nDA:1,3\n").unwrap(),
        lines(&[("a", &[(1, 5)])])
    );
}

#[test]
fn parse_refuses_a_record_it_cannot_read() {
    assert_eq!(
        parse("SF:a\nDA:x,1\n").unwrap_err(),
        r#"lcov line 2: malformed record "DA:x,1""#
    );
    assert_eq!(
        parse("SF:a\nDA:1\n").unwrap_err(),
        r#"lcov line 2: malformed record "DA:1""#
    );
    assert_eq!(
        parse("DA:1,1\n").unwrap_err(),
        "lcov line 1: a DA record before any SF record"
    );
}

#[test]
fn merge_unions_lines_and_adds_hits() {
    let unit = lines(&[("a", &[(1, 0), (2, 1)])]);
    let e2e = lines(&[("a", &[(1, 2)]), ("b", &[(7, 0)])]);

    assert_eq!(
        merge([&unit, &e2e]),
        lines(&[("a", &[(1, 2), (2, 1)]), ("b", &[(7, 0)])])
    );
}
