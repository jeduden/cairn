//! The `cairn` executable. Everything it does is [`cairn::run`]; this
//! entry point only hands it the process's arguments and streams.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned());

    ExitCode::from(cairn::run(
        args,
        &cairn::BUILD,
        &mut io::stdout(),
        &mut io::stderr(),
    ))
}
