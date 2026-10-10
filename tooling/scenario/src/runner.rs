//! What the `bdd` target needs beside cucumber-rs: its command line,
//! which scenarios it runs, and an executor to drive the run on the
//! current thread.

use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

use cucumber::gherkin::tagexpr::TagOperation;
use cucumber::tag::Ext as _;

use crate::PENDING;

/// Which scenarios a run selects.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// A tag expression such as `@ENG-24`, from `--tags`.
    pub tags: Option<TagOperation>,
    /// Name filters, as `cargo test <filter>` passes them: a scenario is
    /// selected when its name contains one, or one names its tag.
    pub filters: Vec<String>,
}

/// libtest's flags that take no value, accepted and ignored so that
/// `cargo test -- <flag>` across the workspace still runs the target.
const LIBTEST_FLAGS: [&str; 7] = [
    "--nocapture",
    "--show-output",
    "--quiet",
    "-q",
    "--ignored",
    "--include-ignored",
    "--exact",
];

/// libtest's flags that take a value, accepted and ignored likewise.
const LIBTEST_VALUED: [&str; 2] = ["--test-threads", "--color"];

/// Reads the target's command line: `--tags EXPR` (or `-t`), name
/// filters, and the libtest flags cargo may pass on.
///
/// # Errors
///
/// Fails on an unknown flag, a flag missing its value, or a tag
/// expression that does not parse.
pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, value)) if flag.starts_with('-') => (flag, Some(value.to_owned())),
            _ => (arg.as_str(), None),
        };
        let mut value = || {
            inline
                .clone()
                .or_else(|| args.next().cloned())
                .ok_or(format!("{flag} needs a value"))
        };
        match flag {
            "--tags" | "-t" => {
                let expr = value()?;
                options.tags = Some(expr.parse().map_err(|e| format!("--tags {expr:?}: {e}"))?);
            }
            f if LIBTEST_VALUED.contains(&f) => drop(value()?),
            f if LIBTEST_FLAGS.contains(&f) => {}
            f if f.starts_with('-') => return Err(format!("unknown flag {f:?}")),
            _ => options.filters.push(arg.clone()),
        }
    }

    Ok(options)
}

impl Options {
    /// Whether a run with these options runs the scenario `name`
    /// carrying `tags`, written without their `@`. A pending scenario is
    /// never run: it is declared, not written.
    #[must_use]
    pub fn selects<'a>(&self, name: &str, tags: impl Iterator<Item = &'a String> + Clone) -> bool {
        let matches_filter = |f: &String| {
            name.contains(f.as_str()) || tags.clone().any(|t| t == f.trim_start_matches('@'))
        };

        !tags.clone().any(|t| t == PENDING)
            && self
                .tags
                .as_ref()
                .is_none_or(|expr| expr.eval(tags.clone()))
            && (self.filters.is_empty() || self.filters.iter().any(matches_filter))
    }
}

/// Wakes the thread that drives a future.
struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

/// Drives `future` to completion on the current thread, parking it
/// while the future waits. cucumber-rs needs no more of an executor
/// than this.
pub fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        if let Poll::Ready(out) = future.as_mut().poll(&mut cx) {
            return out;
        }
        thread::park();
    }
}

#[cfg(test)]
mod tests;
