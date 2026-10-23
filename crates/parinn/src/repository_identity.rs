//! `parinn repository show|bind` (LANE-02): what identity a directory's
//! repository has, and the widening principal act that binds it by hand
//! or rebinds it after a change.

use std::io::Write;

use crate::Context;

/// `parinn repository show|bind [<directory>] [--commit <id>] [--json]`.
pub(crate) fn repository(
    _args: impl Iterator<Item = String>,
    _ctx: &Context,
    _stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> u8 {
    crate::EXIT_FAILURE
}

#[cfg(test)]
mod tests;
