use super::*;
use crate::suite::SUITE;
use std::env;

/// This test executable, which runs no test and exits 0 when its filter
/// matches none: a program every platform the suite runs on has.
fn quiet() -> (std::path::PathBuf, [&'static str; 2]) {
    (
        env::current_exe().unwrap(),
        ["--exact", "no_test_has_this_name"],
    )
}

#[test]
fn time_program_times_a_program_that_exits_successfully() {
    let (program, args) = quiet();

    let took = time_program(&program, &args, DEADLINE).unwrap();

    assert!(took > Duration::ZERO);
}

#[test]
fn time_program_refuses_a_program_that_exits_unsuccessfully() {
    let (program, _) = quiet();

    let err = time_program(&program, &["--no-such-flag"], DEADLINE).unwrap_err();

    assert!(err.contains(" --no-such-flag exited "), "{err}");
}

#[test]
fn time_program_names_a_program_it_cannot_start() {
    let err = time_program(Path::new("/no/such/program"), &[], DEADLINE).unwrap_err();

    assert!(err.starts_with("start /no/such/program: "), "{err}");
}

#[test]
fn time_program_gives_up_at_its_deadline() {
    let (program, args) = quiet();

    let err = time_program(&program, &args, Duration::ZERO).unwrap_err();

    assert!(err.ends_with(" did not exit within 0ns"), "{err}");
}

#[test]
fn sample_discards_one_warm_up_and_keeps_the_rest_in_nanoseconds() {
    let mut calls = 0_u64;
    let mut time = |_: &Benchmark| {
        calls += 1;
        Ok(Duration::from_micros(calls))
    };

    let samples = sample(&SUITE[0], 3, &mut time).unwrap();

    assert_eq!(samples, [2000, 3000, 4000]);
}

/// A timer whose first `n` samples succeed and whose others fail.
fn failing_after(n: u32) -> impl FnMut(&Benchmark) -> Result<Duration, String> {
    let mut calls = 0;
    move |_| {
        calls += 1;
        if calls > n {
            Err("it broke".to_owned())
        } else {
            Ok(Duration::from_nanos(1))
        }
    }
}

#[test]
fn sample_names_the_benchmark_when_a_sample_fails() {
    // The warm-up failing, and a kept sample failing.
    for n in [0, 2] {
        assert_eq!(
            sample(&SUITE[0], 3, &mut failing_after(n)),
            Err("core_start_up: it broke".to_owned())
        );
    }
}

#[test]
fn sample_saturates_a_time_too_long_for_nanoseconds() {
    let mut time = |_: &Benchmark| Ok(Duration::MAX);

    assert_eq!(sample(&SUITE[0], 1, &mut time), Ok(vec![u64::MAX]));
}
