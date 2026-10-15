//! The `cairn` executable. Everything it does is [`cairn::run`]; this
//! entry point only hands it the process's arguments, environment and
//! streams.

use std::io::{self, BufRead};
use std::process::ExitCode;
use std::sync::mpsc::{self, Receiver};

fn main() -> ExitCode {
    let args = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned());
    let ctx = cairn::Context {
        build: &cairn::BUILD,
        cairn_home: std::env::var_os("CAIRN_HOME"),
        home: std::env::var_os("HOME"),
        home_id: std::env::var_os("CAIRN_HOME_ID"),
        hook_panics: false,
        input: &stdin_lines,
        clock: &|| 0,
    };

    ExitCode::from(cairn::run(args, &ctx, &mut io::stdout(), &mut io::stderr()))
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
