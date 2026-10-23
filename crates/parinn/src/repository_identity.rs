//! `parinn repository show|bind` (LANE-02): what identity a directory's
//! repository has, and the widening principal act that binds it by hand
//! or rebinds it after a change.

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ::repository::CommitId;
use std::collections::BTreeSet;
use std::path::Path;

use administration::act::Act;
use administration::home::Home;
use administration::node::Node;
use administration::repository::{self, Outcome, Shown};
use audit::entry::{Counter, Kind, Occurrence};
use key::random::OsEntropy;
use serde_json::json;
use store::Store;

use crate::act::{Taking, at_terminal};
use crate::{Context, EXIT_USAGE, failed, written};

/// How long a look at a clone's git data may take (ENG-03).
const WITHIN: Duration = Duration::from_secs(10);

/// What a command line asked for.
struct Asked {
    directory: PathBuf,
    commit: Option<CommitId>,
    json: bool,
}

/// Reads `args`, the directory against `ctx`'s working directory.
fn parse(args: impl Iterator<Item = String>, bind: bool, ctx: &Context) -> Result<Asked, String> {
    let (mut directory, mut commit, mut json) = (None, None, false);
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--commit" if bind => {
                let text = args.next().ok_or("--commit wants a commit id")?;
                commit = Some(
                    CommitId::parse(&text)
                        .ok_or("the commit must be 40 lowercase hexadecimal digits")?,
                );
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag:?}")),
            _ if directory.is_none() => directory = Some(PathBuf::from(arg)),
            _ => return Err("want at most one directory".to_owned()),
        }
    }
    let directory = directory.unwrap_or_default();
    let base = ctx.working_dir.as_deref().unwrap_or_else(|| Path::new(""));
    let directory = base.join(directory);

    Ok(Asked {
        directory,
        commit,
        json,
    })
}

/// A deadline `WITHIN` from now.
#[expect(
    clippy::disallowed_methods,
    reason = "a deadline is wall-clock time (ENG-03); no derived artifact reads it (I10)"
)]
fn deadline() -> impl Fn() -> bool {
    let end = Instant::now() + WITHIN;

    move || Instant::now() >= end
}

/// Binds the identity of every repository the `SessionStart` hook
/// observations not yet in the record showed, once each (LANE-02): a change in one is audited
/// and counted, and a clone that cannot be read counted, each once (I6).
/// Nothing is relinked, and a failure leaves the record as it was (I9).
pub(crate) fn bind_observed(ctx: &Context, home: &Home, store: &mut Store) {
    let (Ok(node), Ok(held)) = (Node::find(home, store), store.hook_observations()) else {
        return;
    };
    let directories: BTreeSet<String> = held
        .into_iter()
        .filter(|h| !h.appended)
        .map(|h| h.observation)
        .filter(|o| o.hook == "SessionStart" && !o.cwd.is_empty())
        .map(|o| o.cwd)
        .collect();
    // One budget for every directory this call looks at (ENG-03, I9).
    let deadline = deadline();
    for directory in directories {
        let outcome = repository::bind(
            store,
            &node,
            &mut OsEntropy,
            Path::new(&directory),
            ctx.home.as_deref().map(Path::new),
            &deadline,
        );
        let occurrence = match outcome {
            Ok(Outcome::Changed) => Occurrence::Degraded {
                kind: Kind::Rejected,
                counter: Counter::RepositoryIdentityChanged,
                operation: "repository bind",
                detail: "repository identity changed: no branch link is relinked until a widening bind",
            },
            Ok(Outcome::Unreadable) | Err(_) => Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::RepositoryReadFailed,
                operation: "repository bind",
                detail: "repository could not be read: its identity stays as it was",
            },
            Ok(_) => continue,
        };
        let _ = ctx.audit(&occurrence);
    }
}

/// `parinn repository show|bind [<directory>] [--commit <id>] [--json]`.
pub(crate) fn repository(
    mut args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let verb = args.next();
    let bind = match verb.as_deref() {
        Some("show") => false,
        Some("bind") => true,
        _ => {
            let _ = writeln!(stderr, "parinn repository: want show or bind");
            return EXIT_USAGE;
        }
    };
    let op = if bind {
        "repository bind"
    } else {
        "repository show"
    };
    let asked = match parse(args, bind, ctx) {
        Ok(asked) => asked,
        Err(why) => {
            let _ = writeln!(stderr, "parinn {op}: {why}");
            return EXIT_USAGE;
        }
    };
    if bind {
        binding(ctx, &asked, stdout, stderr)
    } else {
        showing(ctx, &asked, stdout, stderr)
    }
}

/// `parinn repository show`.
fn showing(ctx: &Context, asked: &Asked, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    const OP: &str = "repository show";
    match shown_line(ctx, asked) {
        Ok(text) => written(writeln!(stdout, "{text}"), stderr),
        Err(e) => failed(OP, &e, stderr),
    }
}

/// What `parinn repository show` prints.
fn shown_line(ctx: &Context, asked: &Asked) -> Result<String, String> {
    let mut taking = Taking::open(ctx)?;
    let shown = repository::show(
        &mut taking.store,
        &taking.node,
        &mut OsEntropy,
        &asked.directory,
        ctx.home.as_deref().map(Path::new),
        &deadline(),
    )
    .map_err(|e| e.to_string())?;
    let (status, commit) = match shown {
        Shown::NoRepository => ("no repository", None),
        Shown::Unreadable => ("unreadable", None),
        Shown::Unbound(c) => ("unbound", Some(c)),
        Shown::Bound(c) => ("bound", Some(c)),
        Shown::Changed(c) => ("changed", Some(c)),
    };
    // A provisional identity has no commit to name.
    let named = commit.as_ref().map(|c| c.as_ref().map(CommitId::as_str));

    Ok(match (asked.json, named) {
        (true, named) => json!({"status": status, "repository": named.flatten()}).to_string(),
        (false, Some(c)) => format!("{status}: {}", c.unwrap_or("provisional")),
        (false, None) => status.to_owned(),
    })
}

/// `parinn repository bind`: the widening principal act.
fn binding(ctx: &Context, asked: &Asked, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    const OP: &str = "repository bind";
    if let Err(code) = at_terminal(ctx, OP, stderr) {
        return code;
    }
    match bound_line(ctx, asked, stderr) {
        Ok(line) => written(writeln!(stdout, "{line}"), stderr),
        Err(code) => code,
    }
}

/// Takes the act and prints what it bound, or the exit code of the
/// failure it reported.
fn bound_line(ctx: &Context, asked: &Asked, stderr: &mut dyn Write) -> Result<String, u8> {
    const OP: &str = "repository bind";
    let mut taking = Taking::open(ctx).map_err(|e| failed(OP, &e, stderr))?;
    let rebind = repository::plan(
        &mut taking.store,
        &taking.node,
        &mut OsEntropy,
        &asked.directory,
        ctx.home.as_deref().map(Path::new),
        asked.commit.clone(),
        &deadline(),
    )
    .map_err(|e| failed(OP, &e, stderr))?;
    let act = Act::RepositoryBind {
        identity: rebind.content(),
    };
    let address = taking.take(ctx, OP, &act, stderr)?;
    repository::apply(&mut taking.store, &rebind).map_err(|e| failed(OP, &e, stderr))?;
    ctx.audit(&Occurrence::PrincipalAct {
        operation: OP,
        detail: "repository identity bound",
    })
    .map_err(|e| {
        let why = format!("the bind is recorded, but the audit log took no note of it: {e}");
        failed(OP, &why, stderr)
    })?;
    let commit = rebind.commit.as_ref().map(CommitId::as_str);

    Ok(if asked.json {
        json!({"repository": commit, "act": address.to_string()}).to_string()
    } else {
        format!("bound: {}", commit.unwrap_or("provisional"))
    })
}

#[cfg(test)]
mod tests;
