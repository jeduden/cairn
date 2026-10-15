use std::ffi::OsStr;

use super::*;

/// The probe a confined start runs in place of a component: this test
/// binary, filtered to [`probe_opens_a_socket`].
const PROBE_TEST: &str = "boundary::tests::probe_opens_a_socket";

/// What a confined start of this test binary runs: the probe alone.
fn probe_args() -> [&'static str; 3] {
    ["--exact", PROBE_TEST, "--ignored"]
}

/// The core's confinement on this machine, or `None` when the person
/// running the tests opted out of it with [`UNCONFINED`]; an opt-out
/// says so, and CI refuses it.
fn confined() -> Option<Confinement> {
    match Confinement::of(Component::Core) {
        Ok(c) if c.mode == Mode::Confined => Some(c),
        Ok(_) => None,
        Err(e) => panic!("{e}"),
    }
}

fn unconfined() -> Confinement {
    Confinement {
        component: Component::Core,
        mode: Mode::Unconfined,
    }
}

fn this_test_binary() -> PathBuf {
    std::env::current_exe().unwrap()
}

/// The probe itself: a test the confined starts select, never run on
/// its own.
#[test]
#[ignore = "the probe a confined start runs: it opens a socket"]
fn probe_opens_a_socket() {
    let _ = probe();
}

#[test]
fn the_core_is_named_core_and_denies_all_network() {
    assert_eq!(Component::named("core").unwrap(), Component::Core);
    assert_eq!(Component::Core.name(), "core");
    assert_eq!(Component::Core.confinement(), "denies all network");
}

#[test]
fn a_component_the_suite_cannot_confine_yet_is_named() {
    let err = Component::named("launcher").unwrap_err();

    assert_eq!(
        err.to_string(),
        "ENG-12: the end-to-end suite cannot confine the launcher yet: only the core exists"
    );
}

#[test]
fn opting_out_outside_ci_runs_unconfined() {
    let var = |k: &str| (k == UNCONFINED).then(|| OsString::from("1"));

    let got = mode(var, || panic!("an opt-out runs no preflight"));

    assert_eq!(got.unwrap(), Mode::Unconfined);
}

#[test]
fn ci_refuses_the_opt_out() {
    let var = |_: &str| Some(OsString::from("true"));

    let err = mode(var, || Ok(())).unwrap_err();

    assert!(matches!(err, Error::Refused));
    assert_eq!(
        err.to_string(),
        "ENG-12: CAIRN_E2E_UNCONFINED is set, but CI runs every component confined: unset it"
    );
}

#[test]
fn ci_set_to_false_is_no_ci() {
    let var = |k: &str| Some(OsString::from(if k == "CI" { "false" } else { "1" }));

    assert_eq!(mode(var, || Ok(())).unwrap(), Mode::Unconfined);
}

#[test]
fn without_an_opt_out_the_start_is_confined() {
    let none = |_: &str| None;
    let empty = |_: &str| Some(OsString::new());

    assert_eq!(mode(none, || Ok(())).unwrap(), Mode::Confined);
    assert_eq!(mode(empty, || Ok(())).unwrap(), Mode::Confined);
}

#[test]
fn a_machine_that_cannot_confine_fails_loudly() {
    let err = mode(|_: &str| None, || Err("unshare: not found".to_owned())).unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.starts_with(
            "ENG-12: the end-to-end suite cannot confine a component on this machine: unshare: not found"
        ),
        "{msg}"
    );
    assert!(msg.contains("set CAIRN_E2E_UNCONFINED=1"), "{msg}");
}

#[test]
fn preflight_passes_where_the_tools_run() {
    let Some(_) = confined() else { return };

    assert_eq!(preflight(), Ok(()));
}

#[test]
fn preflight_names_a_tool_that_will_not_start() {
    let err = preflight_with("cairn-no-such-unshare").unwrap_err();

    assert!(err.starts_with("start cairn-no-such-unshare: "), "{err}");
}

#[test]
fn preflight_names_a_tool_that_fails() {
    let err = preflight_with("false").unwrap_err();

    assert!(err.starts_with("false --map-current-user"), "{err}");
}

#[test]
fn a_confined_command_runs_the_program_under_the_namespace_and_the_trace() {
    let c = Confinement {
        component: Component::Core,
        mode: Mode::Confined,
    };

    let command = c.command(Path::new("/bin/cairn"), &["version"], Path::new("/t/trace"));

    assert_eq!(command.get_program(), "unshare");
    let args: Vec<&OsStr> = command.get_args().collect();
    assert_eq!(
        args,
        [
            "--map-current-user",
            "--net",
            "--",
            "strace",
            "-f",
            "-qq",
            "-e",
            "signal=none",
            "-e",
            "trace=%net,io_uring_setup",
            "-e",
            "inject=socket,socketpair,io_uring_setup:error=EACCES",
            "-o",
            "/t/trace",
            "--",
            "/bin/cairn",
            "version",
        ]
    );
}

#[test]
fn an_unconfined_command_runs_the_program_as_is() {
    let command = unconfined().command(Path::new("/bin/cairn"), &["version"], Path::new("/t"));

    assert_eq!(command.get_program(), "/bin/cairn");
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["version"]);
}

#[test]
fn every_line_of_the_trace_is_a_network_call() {
    let trace = "\n\
        4242  socket(AF_INET, SOCK_DGRAM|SOCK_CLOEXEC, IPPROTO_IP) = -1 EACCES (Permission denied) (INJECTED)\n\
        connect(3, {sa_family=AF_UNIX}, 110) = -1 ENOTSOCK\n  \n";

    assert_eq!(
        network_calls(trace),
        [
            "socket(AF_INET, SOCK_DGRAM|SOCK_CLOEXEC, IPPROTO_IP) = -1 EACCES (Permission denied) (INJECTED)",
            "connect(3, {sa_family=AF_UNIX}, 110) = -1 ENOTSOCK",
        ]
    );
    assert!(network_calls("").is_empty());
}

#[test]
fn the_caller_is_the_test_thread_and_the_place() {
    let here = Location::caller();

    let got = caller(Some("a_test"), here);

    assert_eq!(got, format!("a_test ({}:{})", here.file(), here.line()));
    assert!(caller(None, here).starts_with("an unnamed thread ("));
}

#[test]
fn a_quiet_trace_passes_the_output_through() {
    let c = unconfined();
    let output = Command::new("true").output().unwrap();

    let got = c.finish(output.clone(), Ok(String::new()), "t".to_owned());

    assert_eq!(got.unwrap(), output);
}

#[test]
fn a_network_call_in_the_trace_fails_naming_the_component_and_the_caller() {
    let c = unconfined();
    let output = Command::new("true").output().unwrap();

    let err = c
        .finish(
            output,
            Ok("1 socket(AF_INET) = -1\n".to_owned()),
            "my_test (x.rs:3)".to_owned(),
        )
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "ENG-12: the core opened a socket outside its boundary, which denies all network; \
         started by my_test (x.rs:3):\n  socket(AF_INET) = -1"
    );
}

#[test]
fn a_missing_trace_fails() {
    let c = unconfined();
    let output = Command::new("true").output().unwrap();

    let err = c
        .finish(
            output,
            Err(io::Error::from(io::ErrorKind::NotFound)),
            "t".to_owned(),
        )
        .unwrap_err();

    assert!(
        err.to_string()
            .starts_with("ENG-12: make or read the trace of a confined start: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn an_unconfined_start_runs_the_program_without_a_trace() {
    let home = TempDir::new().unwrap();

    let out = unconfined()
        .run(Path::new("true"), &[], home.path())
        .unwrap();

    assert!(out.status.success());
}

#[test]
fn a_start_that_cannot_run_names_the_program() {
    let home = TempDir::new().unwrap();

    let err = unconfined()
        .run(Path::new("/cairn/no/such/program"), &[], home.path())
        .unwrap_err();

    assert!(
        err.to_string()
            .starts_with("ENG-12: start /cairn/no/such/program: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn a_confined_start_isolates_home_and_passes_a_quiet_program_through() {
    let Some(c) = confined() else { return };
    let home = TempDir::new().unwrap();

    let out = c
        .run(Path::new("sh"), &["-c", "echo \"$HOME\""], home.path())
        .unwrap();

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        home.path().display().to_string()
    );
}

#[test]
fn a_socket_the_confined_probe_opens_fails_naming_the_component_and_the_caller() {
    let Some(c) = confined() else { return };
    let home = TempDir::new().unwrap();

    let err = c
        .run(&this_test_binary(), &probe_args(), home.path())
        .unwrap_err();

    let Error::Socket {
        component,
        caller,
        calls,
    } = &err
    else {
        panic!("want a socket outside the boundary, got {err}");
    };
    assert_eq!(*component, Component::Core);
    assert!(
        caller.contains(
            "a_socket_the_confined_probe_opens_fails_naming_the_component_and_the_caller"
        ),
        "{caller}"
    );
    assert!(calls.iter().any(|c| c.starts_with("socket(")), "{calls:?}");
    assert!(
        err.to_string().contains("the core opened a socket"),
        "{err}"
    );
}

#[test]
fn the_unconfined_probe_opens_its_socket_unseen() {
    let home = TempDir::new().unwrap();

    let out = unconfined()
        .run(&this_test_binary(), &probe_args(), home.path())
        .unwrap();

    assert!(out.status.success());
}

#[test]
fn the_probe_opens_a_socket_on_loopback() {
    assert!(probe().is_ok());
}

#[test]
fn a_confinement_names_its_component() {
    assert_eq!(unconfined().component(), Component::Core);
}

#[test]
fn only_a_failed_start_or_trace_has_a_cause() {
    let causeless = [
        Error::NoConfinement {
            component: "launcher".to_owned(),
        },
        Error::Refused,
        Error::Unavailable {
            reason: "r".to_owned(),
        },
        Error::Socket {
            component: Component::Core,
            caller: "c".to_owned(),
            calls: Vec::new(),
        },
    ];

    for err in causeless {
        assert!(std::error::Error::source(&err).is_none(), "{err}");
    }
}

#[test]
fn an_unconfined_start_hands_its_input_to_the_program() {
    let home = TempDir::new().unwrap();

    let out = unconfined()
        .run_with_input(Path::new("cat"), &[], home.path(), b"line one\nline two\n")
        .unwrap();

    assert!(out.status.success());
    assert_eq!(out.stdout, b"line one\nline two\n");
}

#[test]
fn a_confined_start_hands_its_input_to_the_program() {
    let Some(c) = confined() else { return };
    let home = TempDir::new().unwrap();

    let out = c
        .run_with_input(
            Path::new("sh"),
            &["-c", "cat"],
            home.path(),
            b"to the core\n",
        )
        .unwrap();

    assert!(out.status.success());
    assert_eq!(out.stdout, b"to the core\n");
}

#[test]
fn a_start_with_input_that_cannot_run_names_the_program() {
    let home = TempDir::new().unwrap();

    let err = unconfined()
        .run_with_input(Path::new("/cairn/no/such/program"), &[], home.path(), b"x")
        .unwrap_err();

    assert!(matches!(err, Error::Start { .. }), "{err}");
}

#[test]
fn an_unconfined_start_hands_its_environment_and_input_to_the_program() {
    let home = TempDir::new().unwrap();

    let out = unconfined()
        .run_in(
            Path::new("sh"),
            &["-c", "printf '%s:' \"$CAIRN_HOME_ID\"; cat"],
            home.path(),
            &[("CAIRN_HOME_ID", "home-b")],
            Some(b"in"),
        )
        .unwrap();

    assert_eq!(out.stdout, b"home-b:in");
}

#[test]
fn a_confined_start_hands_its_environment_to_the_program() {
    let Some(c) = confined() else { return };
    let home = TempDir::new().unwrap();

    let out = c
        .run_in(
            Path::new("sh"),
            &["-c", "printf '%s' \"$CAIRN_TEST_FAULT\""],
            home.path(),
            &[("CAIRN_TEST_FAULT", "hook-panic")],
            None,
        )
        .unwrap();

    assert!(out.status.success());
    assert_eq!(out.stdout, b"hook-panic");
}

#[test]
fn a_start_s_environment_never_moves_it_out_of_its_isolated_home() {
    let home = TempDir::new().unwrap();

    let out = unconfined()
        .run_in(
            Path::new("sh"),
            &["-c", "printf '%s|%s' \"$HOME\" \"$CAIRN_HOME\""],
            home.path(),
            &[("HOME", "/elsewhere"), ("CAIRN_HOME", "/elsewhere/.cairn")],
            None,
        )
        .unwrap();

    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!(
            "{}|{}",
            home.path().display(),
            home.path().join(".cairn").display()
        )
    );
}

#[test]
fn a_shell_line_quotes_every_word() {
    assert_eq!(
        shell_line(Path::new("/bin/cairn"), &["counter", "it's"]),
        r"exec '/bin/cairn' 'counter' 'it'\''s'"
    );
}

#[test]
fn a_start_at_a_terminal_runs_the_program_under_script() {
    let line = terminal_args(Path::new("/bin/cairn"), &["counter", "ack"]);

    assert_eq!(
        line,
        ["-qefc", "exec '/bin/cairn' 'counter' 'ack'", "/dev/null"]
    );
}

#[test]
fn an_unconfined_start_at_a_terminal_gives_the_program_one() {
    let home = TempDir::new().unwrap();

    let out = unconfined()
        .run_at_terminal(
            Path::new("sh"),
            &["-c", "test -t 0 && test -t 1 && echo \"$CAIRN_HOME_ID\""],
            home.path(),
            &[("CAIRN_HOME_ID", "h1")],
        )
        .unwrap();

    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "h1");
}

#[test]
fn a_confined_start_at_a_terminal_gives_the_program_one() {
    let Some(c) = confined() else { return };
    let home = TempDir::new().unwrap();

    let out = c
        .run_at_terminal(
            Path::new("sh"),
            &["-c", "test -t 0 && test -t 1 && echo \"$HOME\""],
            home.path(),
            &[],
        )
        .unwrap();

    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        home.path().display().to_string()
    );
}

#[test]
fn a_start_at_a_terminal_passes_the_program_exit_code() {
    let home = TempDir::new().unwrap();

    let out = unconfined()
        .run_at_terminal(Path::new("sh"), &["-c", "exit 7"], home.path(), &[])
        .unwrap();

    assert_eq!(out.status.code(), Some(7));
}
