//! ENG-01: every toolchain the repository builds with is fixed to an
//! exact version.

/// Whether `go_mod` fixes the Go toolchain with a `toolchain goX.Y.Z`
/// line.
#[must_use]
pub fn go_toolchain_pinned(go_mod: &str) -> bool {
    go_mod.lines().any(|line| {
        let fields: Vec<&str> = line.split_whitespace().collect();
        fields.len() == 2 && fields[0] == "toolchain" && fields[1].starts_with("go1.")
    })
}

#[cfg(test)]
mod tests;
