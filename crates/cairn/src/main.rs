//! The `cairn` executable. Everything it does is [`cairn::run`]; this
//! entry point only hands it the process's arguments, environment and
//! streams.

use std::io::{self, BufRead, IsTerminal};
use std::process::ExitCode;
use std::sync::mpsc::{self, Receiver};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    if args.first().is_some_and(|a| a == "hook") {
        // A hook handler writes nothing to stderr, the default panic
        // message included (§9.1, ENG-04); its entry point catches the
        // panic and audits it.
        std::panic::set_hook(Box::new(|_| {}));
    }
    let ctx = cairn::Context {
        build: &cairn::BUILD,
        cairn_home: std::env::var_os("CAIRN_HOME"),
        home: std::env::var_os("HOME"),
        home_id: std::env::var_os("CAIRN_HOME_ID"),
        // The development build's seam for the `@ENG-04` scenario; a
        // release build never reads it.
        hook_panics: cfg!(debug_assertions)
            && std::env::var_os("CAIRN_TEST_FAULT").is_some_and(|v| v == "hook-panic"),
        terminal: terminal(),
        working_dir: std::env::current_dir().ok(),
        // A development build's seam for the tests; a release build
        // reads only the documented path.
        managed_policy: cairn::managed_policy(
            cfg!(debug_assertions)
                .then(|| std::env::var_os("CAIRN_TEST_MANAGED_POLICY"))
                .flatten(),
        ),
        os_user: os_user(),
        input: &stdin_lines,
        clock: &now,
    };

    ExitCode::from(cairn::run(args, &ctx, &mut io::stdout(), &mut io::stderr()))
}

/// The wall clock, in seconds since the Unix epoch, for audit entries.
#[expect(
    clippy::disallowed_methods,
    reason = "an audit entry records when it was made; no derived artifact reads it (I10)"
)]
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The running OS user, on an OS whose file modes Cairn checks; unknown
/// there when it cannot be learnt, which refuses managed policy.
fn os_user() -> configuration::source::OsUser {
    use configuration::source::OsUser;
    #[cfg(unix)]
    return administration::home::running_uid().map_or(OsUser::Unknown, OsUser::Known);
    #[cfg(not(unix))]
    OsUser::Unchecked
}

/// Whether the process's standard input and output are both a terminal
/// (OWN-12).
fn terminal() -> bool {
    #[expect(
        clippy::disallowed_methods,
        reason = "asking whether stdin is a terminal reads nothing, so it waits for nothing"
    )]
    let stdin = io::stdin();

    stdin.is_terminal() && io::stdout().is_terminal()
}

/// The process's input, one line at a time, read on a thread of its
/// own so its reader waits for each line with a deadline (ENG-03).
fn stdin_lines() -> Receiver<io::Result<String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        #[expect(
            clippy::disallowed_methods,
            reason = "the reading thread blocks on stdin; its reader waits with recv_timeout"
        )]
        let stdin = io::stdin();
        // Reading stops once the reader is gone.
        stdin
            .lock()
            .lines()
            .map_while(|line| tx.send(line).ok())
            .for_each(drop);
    });

    rx
}
