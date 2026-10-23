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
    let directory = match &ctx.working_dir {
        Some(base) => base.join(directory),
        None => directory,
    };

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
/// observations showed, once each (LANE-02): a change in one is audited
/// and counted, and a clone that cannot be read counted, each once (I6).
/// Nothing is relinked, and a failure leaves the record as it was (I9).
pub(crate) fn bind_observed(ctx: &Context, home: &Home, store: &mut Store) {
    let Ok(node) = Node::find(home, store) else {
        return;
    };
    let Ok(held) = store.hook_observations() else {
        return;
    };
    let directories: BTreeSet<String> = held
        .into_iter()
        .map(|h| h.observation)
        .filter(|o| o.hook == "SessionStart" && !o.cwd.is_empty())
        .map(|o| o.cwd)
        .collect();
    for directory in directories {
        let outcome = repository::bind(
            store,
            &node,
            &mut OsEntropy,
            Path::new(&directory),
            &deadline(),
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
    let mut taking = match Taking::open(ctx) {
        Ok(t) => t,
        Err(e) => return failed(OP, &e, stderr),
    };
    let shown = match repository::show(
        &mut taking.store,
        &taking.node,
        &mut OsEntropy,
        &asked.directory,
        &deadline(),
    ) {
        Ok(shown) => shown,
        Err(e) => return failed(OP, &e, stderr),
    };
    let (status, commit) = match shown {
        Shown::NoRepository => ("no repository", None),
        Shown::Unreadable => ("unreadable", None),
        Shown::Unbound(c) => ("unbound", Some(c)),
        Shown::Bound(c) => ("bound", Some(c)),
        Shown::Changed(c) => ("changed", Some(c)),
    };
    // A provisional identity has no commit to name.
    let named = commit.as_ref().map(|c| c.as_ref().map(CommitId::as_str));
    let text = match (asked.json, named) {
        (true, named) => json!({"status": status, "repository": named.flatten()}).to_string(),
        (false, Some(c)) => format!("{status}: {}", c.unwrap_or("provisional")),
        (false, None) => status.to_owned(),
    };

    written(writeln!(stdout, "{text}"), stderr)
}

/// `parinn repository bind`: the widening principal act.
fn binding(ctx: &Context, asked: &Asked, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    const OP: &str = "repository bind";
    if let Err(code) = at_terminal(ctx, OP, stderr) {
        return code;
    }
    let mut taking = match Taking::open(ctx) {
        Ok(t) => t,
        Err(e) => return failed(OP, &e, stderr),
    };
    let rebind = match repository::plan(
        &mut taking.store,
        &taking.node,
        &mut OsEntropy,
        &asked.directory,
        asked.commit.clone(),
        &deadline(),
    ) {
        Ok(rebind) => rebind,
        Err(e) => return failed(OP, &e, stderr),
    };
    let act = Act::RepositoryBind {
        identity: rebind.content(),
    };
    let address = match taking.take(ctx, OP, &act, stderr) {
        Ok(address) => address,
        Err(code) => return code,
    };
    if let Err(e) = repository::apply(&mut taking.store, &rebind) {
        return failed(OP, &e, stderr);
    }
    let noted = ctx.audit(&Occurrence::PrincipalAct {
        operation: OP,
        detail: "repository identity bound",
    });
    if let Err(e) = noted {
        let why = format!("the bind is recorded, but the audit log took no note of it: {e}");
        return failed(OP, &why, stderr);
    }
    let name = rebind
        .commit
        .as_ref()
        .map_or("provisional", CommitId::as_str);
    let line = if asked.json {
        json!({"repository": rebind.commit.as_ref().map(CommitId::as_str), "act": address.to_string()})
            .to_string()
    } else {
        format!("bound: {name}")
    };

    written(writeln!(stdout, "{line}"), stderr)
}

#[cfg(test)]
mod tests;
