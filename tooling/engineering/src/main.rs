//! The `protection-check` command compares the ruleset the repository
//! commits for main with the one GitHub holds (ENG-21). The nightly
//! workflow fetches the live ruleset and runs it; it exits non-zero
//! when the live ruleset is weaker. It reads files and writes the
//! terminal only. It is CI tooling and never ships.

use std::fs;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let code = engineering::branch_protection::cli::run(
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
        &|path| fs::read_to_string(path),
    );

    ExitCode::from(code)
}
