use super::*;

#[test]
fn an_acceptance_names_its_benchmark_ceiling_and_reason() {
    let text = "# A comment.\n\ncore_start_up 30 the core links SQLite\n";

    let got = parse(text).unwrap();

    assert_eq!(
        got,
        [Acceptance {
            benchmark: "core_start_up".into(),
            up_to_percent: 30,
            reason: "the core links SQLite".into(),
        }]
    );
    assert!(parse("").unwrap().is_empty());
}

#[test]
fn a_malformed_acceptance_is_refused_naming_its_line() {
    for (text, want) in [
        (
            "core_start_up",
            "line 1: want <benchmark> <percent> <reason>",
        ),
        (
            "core_start_up 30",
            "line 1: want <benchmark> <percent> <reason>",
        ),
        (
            "# c\ncore_start_up x why",
            "line 2: \"x\" is not a whole percent",
        ),
        (
            "core_start_up 10 why",
            "line 1: 10% is no more than the gate's 10%, so it accepts nothing",
        ),
    ] {
        assert_eq!(parse(text), Err(want.to_owned()), "{text}");
    }
}

#[test]
fn only_an_acceptance_the_base_does_not_carry_is_fresh() {
    let a = parse("core_start_up 30 why\nother 20 because\n").unwrap();
    let base = parse("other 20 because\n").unwrap();

    assert_eq!(fresh(&a, &base), &a[..1]);
    assert!(fresh(&a, &a).is_empty());
}

#[test]
fn an_acceptance_of_a_benchmark_the_base_names_is_not_fresh_however_its_line_reads() {
    let base = parse("core_start_up 30 why\n").unwrap();
    let reworded = parse("core_start_up 30 another reason\n").unwrap();
    let raised = parse("core_start_up 40 why\n").unwrap();

    assert!(fresh(&reworded, &base).is_empty());
    assert!(fresh(&raised, &base).is_empty());
}
