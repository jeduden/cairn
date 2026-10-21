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

/// A test the deadline test starts as a program of its own, and which
/// holds still for longer than that test's deadline: ignored, so no
/// run of the suite starts it.
#[test]
#[ignore = "run only as the program the deadline test times"]
fn holds_still_past_the_deadline() {
    thread::sleep(Duration::from_secs(5));
}

#[test]
fn time_program_gives_up_at_its_deadline() {
    let program = env::current_exe().unwrap();
    let args = [
        "--exact",
        "measure::tests::holds_still_past_the_deadline",
        "--ignored",
    ];

    let err = time_program(&program, &args, Duration::from_millis(50)).unwrap_err();

    assert!(err.ends_with(" did not exit within 50ms"), "{err}");
}

#[test]
fn time_benchmark_times_each_benchmark_over_its_fixture() {
    const QUIET: [&str; 2] = ["--exact", "no_test_has_this_name"];
    let (program, _) = quiet();
    for b in SUITE {
        let fixture = match b.fixture {
            Fixture::Search { lines, .. } => Fixture::Search {
                lines,
                prepare: &QUIET,
            },
            other => other,
        };
        let quiet_benchmark = Benchmark {
            args: &QUIET,
            fixture,
            ..b
        };

        let took = time_benchmark(&program, &quiet_benchmark, DEADLINE).unwrap();

        assert!(took > Duration::ZERO, "{}", b.name);
    }
}

#[test]
fn time_program_fed_hands_its_input_and_fails_as_time_program_does() {
    let (program, args) = quiet();

    let took = time_program_fed(&program, &args, &[], b"{}".to_vec(), DEADLINE).unwrap();
    assert!(took > Duration::ZERO);
    let err =
        time_program_fed(&program, &["--no-such-flag"], &[], Vec::new(), DEADLINE).unwrap_err();
    assert!(err.contains(" --no-such-flag exited "), "{err}");
    let err = time_program_fed(
        Path::new("/no/such/program"),
        &[],
        &[],
        Vec::new(),
        DEADLINE,
    )
    .unwrap_err();
    assert!(err.starts_with("start /no/such/program: "), "{err}");
}

#[test]
fn time_program_in_sets_the_variables_it_is_given() {
    let (program, _) = quiet();
    let env = [("PARINN_BENCHMARK_PROBE", std::path::PathBuf::from("x"))];

    let err = time_program_in(&program, &["--no-such-flag"], &env, DEADLINE).unwrap_err();

    assert!(err.contains(" --no-such-flag exited "), "{err}");
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

#[test]
fn a_hook_sample_whose_handler_counted_an_overrun_is_an_error() {
    let base = testkit::TempDir::new().unwrap();
    let home = fixture::Home::with_transcript(base.path(), "x\n").unwrap();
    assert_eq!(unexceeded(&home, "hook_session_start"), Ok(()));

    let audit = home.env()[1].1.join("audit");
    std::fs::create_dir_all(&audit).unwrap();
    std::fs::write(
        audit.join("audit-000001.jsonl"),
        "{\"counter\":\"hook_budget_exceeded\"}\n",
    )
    .unwrap();

    assert_eq!(
        unexceeded(&home, "hook_session_start"),
        Err(Attempt::Overran("hook_session_start: the handler counted hook_budget_exceeded: it overran its hook budget".to_owned()))
    );
}

#[test]
fn a_sample_whose_handler_overran_once_is_taken_again() {
    let mut calls = 0;
    let took = twice(|| {
        calls += 1;
        if calls == 1 {
            Err(Attempt::Overran("overran".to_owned()))
        } else {
            Ok(Duration::from_millis(7))
        }
    });

    assert_eq!((took, calls), (Ok(Duration::from_millis(7)), 2));
}

#[test]
fn a_handler_that_overruns_twice_in_a_row_is_an_error() {
    let mut calls = 0;
    let took = twice(|| {
        calls += 1;
        Err(Attempt::Overran(format!("overran {calls}")))
    });

    assert_eq!((took, calls), (Err("overran 2".to_owned()), 2));
}

#[test]
fn a_sample_that_cannot_be_taken_is_not_taken_again() {
    let mut calls = 0;
    let took = twice(|| {
        calls += 1;
        Err(Attempt::Failed("no fixture".to_owned()))
    });

    assert_eq!((took, calls), (Err("no fixture".to_owned()), 1));
}

#[test]
fn a_sample_that_gives_a_time_is_taken_once() {
    let mut calls = 0;
    let took = twice(|| {
        calls += 1;
        Ok(Duration::from_millis(3))
    });

    assert_eq!((took, calls), (Ok(Duration::from_millis(3)), 1));
}

#[test]
fn time_program_fed_hands_the_program_its_input() {
    let (program, args) = quiet();

    let took = time_program_fed(&program, &args, &[], b"input\n".to_vec(), DEADLINE).unwrap();

    assert!(took > Duration::ZERO);
}

#[test]
fn time_program_fed_reports_a_program_it_cannot_start() {
    let err = time_program_fed(
        Path::new("/no/such/program"),
        &[],
        &[],
        b"input".to_vec(),
        DEADLINE,
    )
    .unwrap_err();

    assert!(err.starts_with("start /no/such/program"), "{err}");
}

#[test]
fn a_search_fixture_that_cannot_be_prepared_names_the_benchmark() {
    let (program, _) = quiet();
    let failing = Benchmark {
        fixture: Fixture::Search {
            lines: 1,
            prepare: &["--no-such-flag"],
        },
        ..SUITE[4]
    };

    let err = time_benchmark(&program, &failing, DEADLINE).unwrap_err();

    assert!(
        err.starts_with("prepare the fixture of search_events: "),
        "{err}"
    );
}
