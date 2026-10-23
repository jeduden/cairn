//! `parinn rebuild` (PRV-13, I10): derives every derived artifact again
//! from the record and the node's key set alone: the origins the store
//! keeps and the search index. It reads the record and appends no event.

use std::io::Write;

use administration::node::Node;
use administration::origin;
use key::signing::PublicKey;
use store::{Store, StoreError};

use crate::{Context, failed, json_flag, store_failed, written};

/// `parinn rebuild [--json]`.
pub(crate) fn rebuild(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("rebuild", args, stderr) else {
        return crate::EXIT_USAGE;
    };
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("rebuild", &e, stderr),
    };
    if !home.store().exists() {
        return failed(
            "rebuild",
            &format!("no store at {}", home.store().display()),
            stderr,
        );
    }
    let mut store = match Store::open(&home.store()) {
        Ok(store) => store,
        Err(e) => return store_failed("rebuild", &e, stderr),
    };
    let key = match Node::find(&home, &store) {
        Ok(node) => node.device().public(),
        Err(e) => return failed("rebuild", &e, stderr),
    };
    let (origins, indexed) = match derive_again(&mut store, &key) {
        Ok(counts) => counts,
        Err(e) => return failed("rebuild", &e, stderr),
    };
    let line = if as_json {
        format!("{{\"origins\":{origins},\"indexed\":{indexed}}}")
    } else {
        format!("rebuilt: {origins} origins, the search index of {indexed} events")
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// Derives the origins and the search index again; how many writers'
/// origins and how many events it read.
fn derive_again(store: &mut Store, key: &PublicKey) -> Result<(usize, u64), StoreError> {
    let derived = origin::derive(store, key)?;
    let kept = derived
        .iter()
        .map(|(writer, origin)| (*writer, origin.as_str().to_owned()))
        .collect();
    store.replace_origins(&kept)?;
    let indexed = match store.rebuild_search_index() {
        Ok(read) => read,
        Err(StoreError::NoIndex) => 0,
        Err(e) => return Err(e),
    };

    Ok((derived.len(), indexed))
}
