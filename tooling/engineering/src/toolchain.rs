//! ENG-01: every toolchain the repository builds with is fixed to an
//! exact version: Go's by `go.mod`'s toolchain line, Rust's by
//! `rust-toolchain.toml`.

/// Whether `go_mod` fixes the Go toolchain with a `toolchain goX.Y.Z`
/// line.
#[must_use]
pub fn go_toolchain_pinned(go_mod: &str) -> bool {
    go_mod.lines().any(|line| {
        let fields: Vec<&str> = line.split_whitespace().collect();
        fields.len() == 2 && fields[0] == "toolchain" && fields[1].starts_with("go1.")
    })
}

/// Checks that `toml`, a `rust-toolchain.toml`, fixes the Rust toolchain
/// to an exact release such as `channel = "1.99.0"`.
///
/// # Errors
///
/// Fails when the file names no channel, or one that is not
/// `MAJOR.MINOR.PATCH`.
pub fn rust_toolchain_pinned(toml: &str) -> Result<(), String> {
    let channel = toml
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("channel")?
                .trim()
                .strip_prefix('=')
                .map(str::trim)
        })
        .ok_or("rust-toolchain.toml names no channel")?
        .trim_matches('"');
    let parts: Vec<&str> = channel.split('.').collect();
    let exact = parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));

    if exact {
        Ok(())
    } else {
        Err(format!(
            "rust-toolchain.toml does not pin an exact release: {channel:?}"
        ))
    }
}

#[cfg(test)]
mod tests;
