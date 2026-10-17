use super::*;
use crate::{NewEvent, SeatId, Store, StoredSeat};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use search::Request;
use serde_json::json;
use testkit::TempDir;

fn room() -> RoomId {
    RoomId::derive(&[1; 32], &[2; 16])
}

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn open(dir: &TempDir) -> Store {
    Store::open(&dir.path().join("store.db")).unwrap()
}

fn seat(store: &mut Store, writer: WriterId, room: RoomId) {
    store
        .add_seat(&StoredSeat {
            writer,
            seat: SeatId::derive(&room, writer.digest().as_bytes()),
            room,
            certificate: json!({}),
        })
        .unwrap();
}

fn line(n: u64, text: &str, provenance: Provenance, time: Option<&str>) -> NewEvent {
    NewEvent {
        kind: EventKind::TranscriptLine,
        provenance,
        content: json!({"line": n, "text": text}),
        commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
        transcript_timestamp: time.map(str::to_owned),
    }
}

fn texts(texts: &[&str]) -> Vec<NewEvent> {
    texts
        .iter()
        .zip(1..)
        .map(|(t, n)| line(n, t, Provenance::Unparsed, None))
        .collect()
}

fn compiled(text: &str) -> Compiled {
    search::compile(&Request {
        text,
        ..Request::default()
    })
    .unwrap()
}

fn find(store: &Store, text: &str) -> Vec<Hit> {
    let query = compiled(text);
    store.search(&base(&query)).unwrap()
}

fn base(query: &Compiled) -> Search<'_> {
    Search {
        query,
        room: room(),
        run: None,
        provenance: &[],
        kinds: &[],
        since: None,
        until: None,
        k: 10,
        deadline: Duration::from_secs(60),
    }
}

/// A store with a seated writer 1 holding `events`.
fn with(dir: &TempDir, events: &[NewEvent]) -> Store {
    let mut store = open(dir);
    seat(&mut store, writer(1), room());
    store.append(writer(1), events).unwrap();
    store
}

fn seqs(hits: &[Hit]) -> Vec<u64> {
    hits.iter().map(|h| h.address.seq.get()).collect()
}

#[test]
fn text_is_the_contents_text_cut_at_the_cap_on_a_character_boundary() {
    let content = json!({"text": "aé"});

    assert_eq!(text_of(&content, 10), Some("aé"));
    assert_eq!(text_of(&content, 2), Some("a"));
    assert_eq!(text_of(&content, 0), Some(""));
    assert_eq!(text_of(&json!({"line": 1}), 10), None);
    assert_eq!(text_of(&json!({"text": 3}), 10), None);
    assert_eq!(text_of(&json!("bare"), 10), None);
}

#[test]
fn a_row_id_follows_the_address_and_a_newer_event_has_the_larger_one() {
    let slot = slot_of(writer(1));
    let (one, two) = (Seq::new(1).unwrap(), Seq::new(2).unwrap());

    assert_eq!(row_id(slot, one), row_id(slot_of(writer(1)), one));
    assert!(row_id(slot, two) > row_id(slot, one));
    assert!(row_id(slot_of(writer(2)), one) < row_id(slot, two));
    assert_eq!(row_id(slot, two) & SLOT_MASK, slot);
    assert!((0..=SLOT_MASK).contains(&slot));
}

#[test]
fn the_row_id_of_an_address_is_what_the_index_uses_and_none_past_the_limit() {
    let at = |seq| Address::new(writer(1), Seq::new(seq).unwrap());

    assert_eq!(
        row_id_of(at(7)),
        Some(row_id(slot_of(writer(1)), Seq::new(7).unwrap()))
    );
    assert_eq!(
        row_id_of(at(SEQ_LIMIT - 1)).unwrap() >> SLOT_BITS,
        (SEQ_LIMIT - 1) as i64
    );
    assert_eq!(row_id_of(at(SEQ_LIMIT)), None);
}

#[test]
fn a_new_store_has_its_index_and_records_how_it_is_built() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    assert!(exists(&store.conn).unwrap());
    assert_eq!(
        store.meta("search_tokenizer").unwrap().as_deref(),
        Some(TOKENIZER)
    );
    assert_eq!(
        store.meta("search_index_cap").unwrap(),
        Some(DEFAULT_CAP.to_string())
    );
}

#[test]
fn an_appended_event_is_found_by_the_words_of_its_text() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy the staging cluster", "lunch plans"]));

    assert_eq!(seqs(&find(&store, "staging")), [1]);
    assert_eq!(seqs(&find(&store, "STAGING deploy")), [1]);
    assert!(find(&store, "production").is_empty());
}

#[test]
fn a_hyphenated_term_matches_its_words_next_to_each_other() {
    let dir = TempDir::new().unwrap();
    let store = with(
        &dir,
        &texts(&["a quokka-early tool result", "early quokka"]),
    );

    assert_eq!(seqs(&find(&store, "quokka-early")), [1]);
}

#[test]
fn operators_in_caller_text_run_nothing() {
    let dir = TempDir::new().unwrap();
    let store = with(
        &dir,
        &texts(&["foo only", "foo OR bar together", "bar only"]),
    );

    assert_eq!(seqs(&find(&store, "foo OR bar")), [2]);
    assert!(find(&store, "foo NEAR(bar)").is_empty());
    assert_eq!(
        seqs(&find(&store, "bar -foo")),
        [2],
        "a minus is no negation"
    );
}

#[test]
fn text_beyond_the_indexing_cap_is_not_indexed() {
    let dir = TempDir::new().unwrap();
    let long = format!("quokka-early {} quokka-late", "x".repeat(DEFAULT_CAP));
    let store = with(&dir, &texts(&[&long]));

    assert_eq!(seqs(&find(&store, "quokka-early")), [1]);
    assert!(find(&store, "quokka-late").is_empty());
}

#[test]
fn payload_text_is_indexed_too() {
    let dir = TempDir::new().unwrap();
    let big = format!("{} needle", "padding ".repeat(2000));
    let store = with(&dir, &texts(&[&big]));

    let stored = store.log(writer(1)).unwrap();
    assert!(stored[0].payload.is_some());
    assert_eq!(seqs(&find(&store, "needle")), [1]);
}

#[test]
fn an_event_without_text_is_left_out_without_a_word_of_it() {
    let dir = TempDir::new().unwrap();
    let mut events = texts(&["one"]);
    events.push(NewEvent {
        kind: EventKind::KeyEvent,
        provenance: Provenance::Unparsed,
        content: json!({"key": "needle"}),
        commitment_key: CommitmentKey::from_bytes([9; 32]),
        transcript_timestamp: None,
    });
    let store = with(&dir, &events);

    assert!(find(&store, "needle").is_empty());
    assert_eq!(seqs(&find(&store, "one")), [1]);
}

#[test]
fn a_text_naming_a_word_five_times_ranks_above_one_naming_it_once() {
    let dir = TempDir::new().unwrap();
    let store = with(
        &dir,
        &texts(&[
            "wombat and four other words here",
            "wombat wombat wombat wombat wombat",
            "nothing relevant",
        ]),
    );

    let hits = find(&store, "wombat");

    assert_eq!(seqs(&hits), [2, 1]);
    assert!(hits[0].score > hits[1].score);
}

fn mixed(dir: &TempDir) -> Store {
    let mut store = open(dir);
    seat(&mut store, writer(1), room());
    seat(&mut store, writer(2), RoomId::derive(&[3; 32], &[4; 16]));
    store
        .append(
            writer(1),
            &[
                line(
                    1,
                    "deploy web",
                    Provenance::parse("web").unwrap(),
                    Some("2026-03-01T10:00:00Z"),
                ),
                line(
                    2,
                    "deploy file",
                    Provenance::parse("file").unwrap(),
                    Some("2026-07-01T10:00:00Z"),
                ),
                line(3, "deploy none", Provenance::parse("file").unwrap(), None),
            ],
        )
        .unwrap();
    store
        .append(
            writer(2),
            &[line(1, "deploy elsewhere", Provenance::Unparsed, None)],
        )
        .unwrap();
    store
}

#[test]
fn a_search_reaches_only_the_writers_of_its_room() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);

    let hits = find(&store, "deploy");

    assert!(hits.iter().all(|h| h.address.writer == writer(1)));
    assert_eq!(hits.len(), 3);
}

#[test]
fn a_run_filter_keeps_one_writer() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    let query = compiled("deploy");

    let none = store
        .search(&Search {
            run: Some(writer(2)),
            ..base(&query)
        })
        .unwrap();
    let one = store
        .search(&Search {
            run: Some(writer(1)),
            ..base(&query)
        })
        .unwrap();

    assert!(none.is_empty(), "writer 2 is in another room");
    assert_eq!(one.len(), 3);
}

#[test]
fn provenance_and_kind_filters_keep_the_events_they_name() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    let query = compiled("deploy");
    let with = |provenance: &[String], kinds: &[String]| {
        seqs(
            &store
                .search(&Search {
                    provenance,
                    kinds,
                    ..base(&query)
                })
                .unwrap(),
        )
    };
    let mut got = with(&["file".to_owned()], &[]);
    got.sort_unstable();

    assert_eq!(with(&["web".to_owned()], &[]), [1]);
    assert_eq!(got, [2, 3]);
    assert_eq!(with(&[], &["transcript_line".to_owned()]).len(), 3);
    assert!(with(&[], &["unparsed_line".to_owned()]).is_empty());
}

#[test]
fn a_time_range_keeps_events_whose_transcript_time_lies_in_it() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    let query = compiled("deploy");
    let range = |since, until| {
        let mut found = seqs(
            &store
                .search(&Search {
                    since,
                    until,
                    ..base(&query)
                })
                .unwrap(),
        );
        found.sort_unstable();
        found
    };

    assert_eq!(range(Some("2026-01-01"), None), [1, 2]);
    assert_eq!(range(None, Some("2026-06-30")), [1]);
    assert_eq!(range(Some("2026-03-01"), Some("2026-03-01")), [1]);
    assert_eq!(
        range(Some("2026-07-01T10:00:00Z"), Some("2026-07-01T10:00:00Z")),
        [2]
    );
    assert!(range(Some("2027-01-01"), None).is_empty());
}

#[test]
fn only_the_newest_window_of_matches_is_ranked() {
    let dir = TempDir::new().unwrap();
    let mut events = vec![line(
        1,
        "alpha alpha alpha alpha",
        Provenance::Unparsed,
        None,
    )];
    events
        .extend((2..=WINDOW as u64 + 1).map(|n| line(n, "alpha pad", Provenance::Unparsed, None)));
    let store = with(&dir, &events);
    let query = compiled("alpha");

    let all = store
        .search(&Search {
            k: rank::MAX_HITS,
            ..base(&query)
        })
        .unwrap();

    assert!(
        !seqs(&all).contains(&1),
        "the oldest match lies outside the window"
    );
    assert_eq!(all.len(), rank::MAX_HITS);
}

#[test]
fn a_search_past_its_deadline_is_cancelled_and_the_next_one_runs() {
    let dir = TempDir::new().unwrap();
    let events: Vec<NewEvent> = (1..=WINDOW as u64)
        .map(|n| line(n, "alpha pad", Provenance::Unparsed, None))
        .collect();
    let store = with(&dir, &events);
    let query = compiled("alpha");

    let late = store.search(&Search {
        deadline: Duration::ZERO,
        ..base(&query)
    });
    let after = store.search(&base(&query));

    assert!(matches!(late, Err(StoreError::Cancelled)), "{late:?}");
    assert_eq!(after.unwrap().len(), 10);
}

#[test]
fn the_index_kept_per_append_has_the_digest_of_a_rebuild() {
    let dir = TempDir::new().unwrap();
    let mut store = mixed(&dir);
    let kept = store.index_digest().unwrap();

    let read = store.rebuild_index().unwrap();

    assert_eq!(read, 4);
    assert_eq!(store.index_digest().unwrap(), kept);
}

#[test]
fn two_stores_holding_the_same_logs_share_a_digest_whatever_arrived_first() {
    let appends = |order: [u8; 2]| {
        let dir = TempDir::new().unwrap();
        let mut store = open(&dir);
        for n in order {
            seat(&mut store, writer(n), room());
            store
                .append(writer(n), &texts(&["shared word", &format!("only {n}")]))
                .unwrap();
        }
        store.index_digest().unwrap()
    };

    assert_eq!(appends([1, 2]), appends([2, 1]));
    assert_ne!(appends([1, 2]), appends([1, 1 + 2]));
}

#[test]
fn a_different_text_is_a_different_digest() {
    let (one, two) = (TempDir::new().unwrap(), TempDir::new().unwrap());

    assert_ne!(
        with(&one, &texts(&["same"])).index_digest().unwrap(),
        with(&two, &texts(&["other"])).index_digest().unwrap()
    );
}

#[test]
fn a_store_made_before_the_index_gets_one_built_from_its_record() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy the cluster", "lunch"]));
    let kept = store.index_digest().unwrap();
    for table in ["events_vocab", "events_fts", "search_writers"] {
        store
            .conn
            .execute_batch(&format!("DROP TABLE {table}"))
            .unwrap();
    }
    drop(store);

    let store = open(&dir);

    assert_eq!(seqs(&find(&store, "cluster")), [1]);
    assert_eq!(store.index_digest().unwrap(), kept);
}

#[test]
fn a_store_opened_for_reading_with_no_index_says_so() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    for table in ["events_vocab", "events_fts", "search_writers"] {
        store
            .conn
            .execute_batch(&format!("DROP TABLE {table}"))
            .unwrap();
    }
    drop(store);
    let mut read = Store::open_read_only(&dir.path().join("store.db")).unwrap();
    let query = compiled("deploy");

    assert!(matches!(
        read.search(&base(&query)),
        Err(StoreError::NoIndex)
    ));
    assert!(matches!(read.index_digest(), Err(StoreError::NoIndex)));
    assert!(matches!(read.rebuild_index(), Err(StoreError::NoIndex)));
}

/// A writer id that shares the slot of `writer(1)`: the same first five
/// bytes, then `tail`.
fn sharing_the_slot(tail: u8) -> WriterId {
    let mut bytes = *writer(1).digest().as_bytes();
    bytes[5..].fill(tail);

    WriterId::parse(&Digest::from_bytes(bytes).to_hex()).unwrap()
}

#[test]
fn of_two_writers_sharing_a_slot_the_smaller_id_is_indexed_whichever_arrived_first() {
    let (a, b) = (writer(1), sharing_the_slot(0));
    let (small, large) = if a < b { (a, b) } else { (b, a) };
    assert_eq!(slot_of(small), slot_of(large));
    let digests: Vec<_> = [[small, large], [large, small]]
        .into_iter()
        .map(|order| {
            let dir = TempDir::new().unwrap();
            let mut store = open(&dir);
            for w in order {
                seat(&mut store, w, room());
                store.append(w, &texts(&["shared word"])).unwrap();
            }
            let hits = find(&store, "shared");
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].address.writer, small);
            assert_eq!(store.index_displaced().unwrap(), [large]);
            let kept = store.index_digest().unwrap();
            store.rebuild_index().unwrap();
            assert_eq!(store.index_digest().unwrap(), kept);
            kept
        })
        .collect();

    assert_eq!(digests[0], digests[1]);
}

#[test]
fn a_store_with_no_displaced_writer_names_none() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);

    assert!(store.index_displaced().unwrap().is_empty());
}

#[test]
fn a_displaced_writer_id_that_does_not_parse_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let store = mixed(&dir);
    store
        .conn
        .execute(
            "UPDATE events SET writer = 'bad' WHERE writer = ?1",
            [writer(2).to_string()],
        )
        .unwrap();

    assert!(matches!(
        store.index_displaced(),
        Err(StoreError::Corrupt { .. })
    ));
}

#[test]
fn a_seq_past_the_limit_is_not_indexed() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    let mut indexing = Indexing::open(&store.conn, writer(1)).unwrap();

    indexing
        .add(Seq::new(SEQ_LIMIT).unwrap(), &json!({"text": "beyond"}))
        .unwrap();

    assert!(find(&store, "beyond").is_empty());
}

#[test]
fn a_hit_naming_a_writer_that_is_no_id_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let store = with(&dir, &texts(&["deploy"]));
    for table in ["search_writers", "seats", "events"] {
        store
            .conn
            .execute(&format!("UPDATE {table} SET writer = 'bad'"), [])
            .unwrap();
    }
    let query = compiled("deploy");

    assert!(matches!(
        store.search(&base(&query)),
        Err(StoreError::Corrupt { .. })
    ));
}

#[test]
fn a_rebuild_failing_midway_leaves_the_index_as_it_was() {
    let dir = TempDir::new().unwrap();
    let mut store = with(&dir, &texts(&["deploy", "lunch"]));
    let kept = store.index_digest().unwrap();
    store
        .conn
        .execute("UPDATE events SET kind = 'bad' WHERE seq = 2", [])
        .unwrap();

    assert!(store.rebuild_index().is_err());
    store
        .conn
        .execute(
            "UPDATE events SET kind = 'transcript_line' WHERE seq = 2",
            [],
        )
        .unwrap();

    assert_eq!(store.index_digest().unwrap(), kept);
}

/// A statement that runs for minutes.
const FOREVER: &str =
    "WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x + 1 FROM c WHERE x < 4000000000)
     SELECT count(*) FROM c";

#[expect(
    clippy::disallowed_methods,
    reason = "a test of time reads the clock to bound its own run"
)]
#[test]
fn work_that_outlasts_its_deadline_is_interrupted_and_cancelled() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    let started = std::time::Instant::now();

    let out = within(&store.conn, Duration::from_millis(50), || {
        Ok(store.conn.query_row(FOREVER, [], |r| r.get::<_, i64>(0))?)
    });

    assert!(matches!(out, Err(StoreError::Cancelled)), "{out:?}");
    assert!(started.elapsed() < Duration::from_secs(30));
}

#[test]
fn work_that_ends_in_time_returns_its_result_and_leaves_the_connection_alone() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    let out = within(&store.conn, Duration::from_secs(60), || Ok(7));

    assert_eq!(out.unwrap(), 7);
    assert_eq!(
        store
            .conn
            .query_row("SELECT 1", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn a_deadline_already_passed_cancels_the_work_before_it_starts() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    let mut ran = false;

    let out = within(&store.conn, Duration::ZERO, || {
        ran = true;
        Ok(())
    });

    assert!(matches!(out, Err(StoreError::Cancelled)));
    assert!(!ran);
}

#[test]
fn work_that_fails_for_another_reason_keeps_its_error() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);

    let out: Result<(), _> = within(&store.conn, Duration::from_secs(60), || {
        Err(StoreError::NoIndex)
    });

    assert!(matches!(out, Err(StoreError::NoIndex)));
}
