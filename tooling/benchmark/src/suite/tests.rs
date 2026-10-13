use super::*;

fn bench(name: &'static str) -> Benchmark {
    Benchmark {
        name,
        requirement: "NFR-01",
        coverage: Coverage::Target,
        measures: "something",
        args: &[],
    }
}

#[test]
fn the_suite_has_no_problems() {
    assert!(problems(&SUITE, &RETIRED).is_empty());
}

#[test]
fn problems_names_a_name_the_samples_file_cannot_carry() {
    assert_eq!(
        problems(&[bench("core start")], &[]),
        [r#"benchmark "core start" is not lowercase letters, digits and underscores"#]
    );
}

#[test]
fn problems_names_a_benchmark_registered_twice() {
    assert_eq!(
        problems(&[bench("a"), bench("b"), bench("a")], &[]),
        ["benchmark a is registered twice"]
    );
}

#[test]
fn problems_names_a_benchmark_registered_and_retired() {
    assert_eq!(
        problems(&[bench("a")], &["a"]),
        ["benchmark a is registered and retired"]
    );
}

#[test]
fn the_core_start_up_is_a_floor_not_a_hook_handler() {
    let start_up = SUITE.iter().find(|b| b.name == "core_start_up").unwrap();

    assert_eq!(start_up.coverage, Coverage::Floor);
    assert_eq!(start_up.args, ["version"]);
}

#[test]
fn every_nfr_id_eng_15_names_is_awaiting_a_benchmark_of_real_code() {
    let ids: Vec<&str> = AWAITING.iter().map(|a| a.requirement).collect();

    assert_eq!(ids, ["NFR-01", "NFR-02", "NFR-03", "NFR-04", "NFR-05"]);
    assert!(AWAITING.iter().all(|a| !a.on.is_empty()));
}
