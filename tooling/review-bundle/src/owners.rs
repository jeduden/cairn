//! Reads `.github/CODEOWNERS` for the paths it assigns, so the review's
//! route follows the same list the stakeholder's consent does (ENG-21).
//!
//! It reads the subset of the format the repository uses: a pattern
//! with a leading `/` is anchored at the root, one without matches at
//! any depth, a trailing `/` names a directory's contents, and `*`
//! matches within one path segment.

/// The patterns `codeowners` assigns, in file order: each line's first
/// field, comments and blank lines left out.
#[must_use]
pub fn patterns(codeowners: &str) -> Vec<String> {
    codeowners
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// Whether `path`, relative to the root, is one `pattern` assigns.
#[must_use]
pub fn owns(pattern: &str, path: &str) -> bool {
    let anchored = pattern.starts_with('/');
    let dir = pattern.ends_with('/');
    let want: Vec<&str> = pattern
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let have: Vec<&str> = path.split('/').collect();
    if want.is_empty() {
        return false;
    }
    let starts = if anchored {
        0..1
    } else {
        0..have.len().saturating_sub(want.len()) + 1
    };

    starts.into_iter().any(|at| {
        let rest = &have[at..];
        rest.len() >= want.len()
            && !(dir && rest.len() == want.len())
            && want.iter().zip(rest).all(|(w, h)| glob(w, h))
    })
}

/// Whether `pattern`, where `*` matches any run of characters, matches
/// all of `text`.
fn glob(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, tail)) => text.strip_prefix(head).is_some_and(|rest| {
            (0..=rest.len())
                .filter(|&i| rest.is_char_boundary(i))
                .any(|i| glob(tail, &rest[i..]))
        }),
    }
}

#[cfg(test)]
mod tests;
