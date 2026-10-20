//! Reports the events the search index leaves out (REC-11, I6): a writer
//! whose slot a smaller writer id holds, and a writer from seq 2^28 on.
//! Each is audited once per writer and class, as the counters
//! `search_index_writer_unindexed` and `search_index_seq_beyond_limit`,
//! and then noted in the index, so a later ingest does not audit it
//! again. A gap whose entry the audit log refused stays unreported until
//! an ingest audits it.

use administration::home::Home;
use audit::entry::{Counter, Kind, Occurrence};
use store::{GapClass, IndexGap, Store, StoreError};

/// The entry for `gap`, the writer named by `writer`, its hex id.
fn occurrence<'a>(gap: IndexGap, writer: &'a str, operation: &'static str) -> Occurrence<'a> {
    let (counter, detail) = match gap.class {
        GapClass::WriterUnindexed => (
            Counter::SearchIndexWriterUnindexed,
            "search index leaves out the events of writer",
        ),
        GapClass::SeqBeyondLimit => (
            Counter::SearchIndexSeqBeyondLimit,
            "search index leaves out the events from seq 2^28 on of writer",
        ),
    };

    Occurrence::Keyed {
        kind: Kind::Dropped,
        counter,
        operation,
        detail,
        key: writer,
    }
}

/// Audits each of `gaps` through `audit`, which says whether the audit
/// log took the entry, and notes the gaps it took in the index of
/// `home`'s store. Opens no store when there is no gap.
///
/// # Errors
///
/// Fails when the store cannot be opened or the note cannot be made.
pub(crate) fn report(
    home: &Home,
    gaps: &[IndexGap],
    operation: &'static str,
    audit: &mut dyn FnMut(&Occurrence<'_>) -> bool,
) -> Result<(), StoreError> {
    if gaps.is_empty() {
        return Ok(());
    }
    let taken: Vec<IndexGap> = gaps
        .iter()
        .copied()
        .filter(|gap| audit(&occurrence(*gap, &gap.writer.to_string(), operation)))
        .collect();

    Store::open(&home.store())?.mark_search_index_gaps_reported(&taken)
}

#[cfg(test)]
pub(crate) mod tests;
