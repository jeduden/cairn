//! The measurements: load, size, search, expand, ingest, hook, cancel
//! and optimize, each the Go harness's command of the same name.

use std::collections::{BTreeMap, HashMap};
use std::thread;
use std::time::{Duration, Instant};

use rusqlite::{Connection, ErrorCode};

use crate::Opts;
use crate::corpus::{BANDS, Corpus, Event, Rng};
use crate::store::{self, Result};

/// cmd_load builds a store of o.n synthetic events.
pub fn load(o: &Opts) -> Result<()> {
    let mut db = store::open(&o.db, o.key, false)?;
    store::create_schema(&db)?;
    let mut c = Corpus::new(o.seed);
    let mut text_bytes: i64 = 0;
    let mut prev = vec![0u8; 32];
    let start = Instant::now();
    let mut lap = start;
    const BATCH: i64 = 10_000;
    let mut seq: i64 = 1;
    while seq <= o.n {
        let tx = store::begin(&mut db)?;
        let mut j = 0;
        while j < BATCH && seq <= o.n {
            let e = c.next();
            text_bytes += e.body.len() as i64;
            prev = store::append(&tx, seq, &e, &prev)?;
            seq += 1;
            j += 1;
        }
        tx.commit()?;
        if (seq - 1) % 500_000 == 0 {
            let now = Instant::now();
            println!(
                "{} events  {:.0} ev/s (lap)  {}s elapsed",
                seq - 1,
                500_000.0 / (now - lap).as_secs_f64(),
                (now - start).as_secs()
            );
            lap = now;
        }
    }
    let el = start.elapsed();
    store::set_meta(&db, "events", o.n)?;
    store::set_meta(&db, "text_bytes", text_bytes)?;
    db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
    println!(
        "RESULT load events={} text_bytes={} elapsed={:?} rate={:.0} ev/s",
        o.n,
        text_bytes,
        el,
        o.n as f64 / el.as_secs_f64()
    );
    report_size(&db, o)
}

/// The store's on-disk footprint against the text it holds (NFR-09).
pub fn report_size(db: &Connection, o: &Opts) -> Result<()> {
    let tb = store::get_meta(db, "text_bytes")?;
    let mut size = 0u64;
    for suf in ["", "-wal", "-shm"] {
        let mut p = o.db.clone().into_os_string();
        p.push(suf);
        if let Ok(m) = std::fs::metadata(&p) {
            size += m.len();
        }
    }
    println!(
        "RESULT size file_bytes={size} text_bytes={tb} ratio={:.3}",
        size as f64 / tb as f64
    );
    let mut st = db.prepare("SELECT name, SUM(pgsize) FROM dbstat GROUP BY name ORDER BY 2 DESC")?;
    let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    for row in rows {
        let (name, bytes) = row?;
        println!("RESULT table name={name} bytes={bytes}");
    }
    Ok(())
}

/// Ranks every match by BM25; the rowid floor, above zero, bounds the
/// candidates to the most recent events.
const SEARCH_SQL: &str = "SELECT e.seq, e.session, snippet(events_fts, 0, '[', ']', '…', 16)
    FROM events_fts JOIN events e ON e.seq = events_fts.rowid
    WHERE events_fts MATCH ? AND events_fts.rowid > ? ORDER BY rank LIMIT 20";

/// Ranks by BM25 only the most recent k matches, then keeps the best 20.
const RECENT_SQL: &str = "SELECT r FROM (
        SELECT rowid AS r, bm25(events_fts) AS s FROM events_fts
        WHERE events_fts MATCH ? ORDER BY rowid DESC LIMIT ?)
    ORDER BY s LIMIT 20";

const SNIPPET_SQL: &str = "SELECT e.seq, e.session, snippet(events_fts, 0, '[', ']', '…', 16)
    FROM events_fts JOIN events e ON e.seq = events_fts.rowid
    WHERE events_fts MATCH ? AND events_fts.rowid = ?";

/// Realistic queries: terms lifted from a stored event, anchored on
/// one term in the requested band, so every query has a hit.
fn search_queries(
    db: &Connection,
    c: &Corpus,
    r: &mut Rng,
    n: usize,
    floor: i64,
    events: i64,
) -> Result<(Vec<String>, Vec<&'static str>)> {
    let rank: HashMap<String, usize> =
        c.words.iter().enumerate().map(|(i, w)| (w.to_lowercase(), i)).collect();
    if events <= floor {
        return Err(format!("no events to draw queries from: window ({floor}, {events}]").into());
    }
    let (mut qs, mut qb) = (Vec::new(), Vec::new());
    let mut tries = 0;
    while qs.len() < n {
        if tries >= 1000 * (n + 1) {
            return Err(format!("found {} of {n} queries in {tries} events", qs.len()).into());
        }
        tries += 1;
        let b = &BANDS[qs.len() % BANDS.len()];
        let seq = floor + 1 + r.below((events - floor) as u64) as i64;
        let body: String = db.query_row("SELECT body FROM events WHERE seq = ?", [seq], |x| x.get(0))?;
        let (mut anchor, mut other) = (Vec::new(), Vec::new());
        for w in body.replace('.', " ").split_whitespace() {
            match rank.get(&w.to_lowercase()) {
                Some(&k) if k >= b.lo && k < b.hi => anchor.push(w.to_owned()),
                Some(&k) if k >= BANDS[0].lo && k < b.lo => other.push(w.to_owned()),
                _ => {}
            }
        }
        if anchor.is_empty() {
            continue;
        }
        let mut terms = vec![format!("\"{}\"", anchor[r.int_n(anchor.len())])];
        let mut extra = r.int_n(3);
        while extra > 0 && !other.is_empty() {
            terms.push(format!("\"{}\"", other[r.int_n(other.len())]));
            extra -= 1;
        }
        qs.push(terms.join(" "));
        qb.push(b.name);
    }
    Ok((qs, qb))
}

/// FTS5 BM25 search latency (NFR-03: p95 ≤ 200 ms).
pub fn search(o: &Opts) -> Result<()> {
    if o.recent > 0 && o.window < crate::DEFAULT_WINDOW {
        return Err("-recent and -window do not combine".into());
    }
    let db = store::open(&o.db, o.key, true)?;
    let events = store::get_meta(&db, "events")?;
    let c = Corpus::new(o.seed);
    let mut r = Rng::new(o.seed + 7, 11);
    let floor = 0.max(events - o.window);
    let (qs, qb) = search_queries(&db, &c, &mut r, o.n as usize + 30, floor, events)?;
    let mut lat: BTreeMap<&str, Vec<Duration>> = BTreeMap::new();
    let mut hits = 0usize;
    for (i, q) in qs.iter().enumerate() {
        let t = Instant::now();
        let n = if o.recent > 0 {
            search_recent(&db, q, o.recent)?
        } else {
            search_from(&db, q, floor)?
        };
        let d = t.elapsed();
        if i < 30 {
            continue; // warm-up
        }
        hits += n;
        lat.entry(qb[i]).or_default().push(d);
        lat.entry("all").or_default().push(d);
    }
    for name in ["common", "mid", "rare", "all"] {
        println!(
            "RESULT search window={} recent={} band={name} {}",
            o.window,
            o.recent,
            stats(lat.get(name).map(Vec::as_slice).unwrap_or(&[]))
        );
    }
    println!("RESULT search mean_hits={:.1}", hits as f64 / lat["all"].len() as f64);
    Ok(())
}

pub fn search_from(db: &Connection, q: &str, floor: i64) -> Result<usize> {
    let mut st = db.prepare_cached(SEARCH_SQL)?;
    let mut rows = st.query(rusqlite::params![q, floor])?;
    let mut n = 0;
    while let Some(row) = rows.next()? {
        let _: (i64, String, String) = (row.get(0)?, row.get(1)?, row.get(2)?);
        n += 1;
    }
    Ok(n)
}

fn search_recent(db: &Connection, q: &str, k: i64) -> Result<usize> {
    let ids: Vec<i64> = {
        let mut st = db.prepare_cached(RECENT_SQL)?;
        let rows = st.query_map(rusqlite::params![q, k], |r| r.get(0))?;
        rows.collect::<std::result::Result<_, _>>()?
    };
    let mut st = db.prepare_cached(SNIPPET_SQL)?;
    for id in &ids {
        let _: (i64, String, String) =
            st.query_row(rusqlite::params![q, id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    }
    Ok(ids.len())
}

pub fn stats(ds: &[Duration]) -> String {
    if ds.is_empty() {
        return "n=0".into();
    }
    let mut s = ds.to_vec();
    s.sort();
    // Nearest rank: the p95 of 300 samples is the 285th, s[284].
    let p = |q: f64| s[((q * s.len() as f64).ceil() as usize).max(1) - 1];
    let mean = s.iter().sum::<Duration>() / s.len() as u32;
    let ms = |d: Duration| format!("{:.2}ms", d.as_secs_f64() * 1000.0);
    format!(
        "n={} mean={} p50={} p95={} p99={} max={}",
        s.len(),
        ms(mean),
        ms(p(0.5)),
        ms(p(0.95)),
        ms(p(0.99)),
        ms(s[s.len() - 1])
    )
}

/// Fetching a 50-event window by seq (NFR-03: expand p95 ≤ 100 ms).
pub fn expand(o: &Opts) -> Result<()> {
    let db = store::open(&o.db, o.key, true)?;
    let events = store::get_meta(&db, "events")?;
    if events <= 50 {
        return Err(format!("expand needs more than 50 events, store holds {events}").into());
    }
    let mut r = Rng::new(o.seed + 9, 13);
    let mut lat = Vec::new();
    let mut st = db.prepare(
        "SELECT seq, session, kind, trust, tool, body, tokens, ts, hash
         FROM events WHERE seq BETWEEN ? AND ? ORDER BY seq",
    )?;
    for i in 0..o.n + 30 {
        let lo = 1 + r.below((events - 50) as u64) as i64;
        let t = Instant::now();
        let mut rows = st.query([lo, lo + 49])?;
        let mut n = 0;
        while let Some(row) = rows.next()? {
            let _: (i64, String, String, i64, Option<String>, String, i64, i64, Vec<u8>) = (
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
            );
            n += 1;
        }
        if n != 50 {
            return Err(format!("expand got {n} rows").into());
        }
        if i >= 30 {
            lat.push(t.elapsed());
        }
    }
    println!("RESULT expand {}", stats(&lat));
    Ok(())
}

fn check_batch(batch: i64) -> Result<()> {
    if batch < 1 {
        return Err(format!("-batch must be at least 1, got {batch}").into());
    }
    Ok(())
}

/// Append throughput on one thread (NFR-04: ≥ 5,000 events/s),
/// o.batch events per transaction, the unit one hook commits.
pub fn ingest(o: &Opts) -> Result<()> {
    check_batch(o.batch)?;
    let mut db = store::open(&o.db, o.key, false)?;
    let mut c = Corpus::from_seeds(o.seed, o.seed + 1000);
    let mut evs: Vec<Event> = (0..o.n).map(|_| c.next()).collect();
    let (mut seq, mut prev) = store::last_hash(&db)?;
    let mut text = 0i64;
    let start = Instant::now();
    let mut i = 0usize;
    while i < evs.len() {
        let tx = store::begin(&mut db)?;
        let mut j = 0;
        while j < o.batch && i < evs.len() {
            seq += 1;
            evs[i].source = 1_000_000;
            evs[i].line = seq;
            text += evs[i].body.len() as i64;
            prev = store::append(&tx, seq, &evs[i], &prev)?;
            i += 1;
            j += 1;
        }
        tx.commit()?;
    }
    let el = start.elapsed();
    println!(
        "RESULT ingest events={} batch={} threads=1 elapsed={:?} rate={:.0} ev/s",
        o.n,
        o.batch,
        el,
        o.n as f64 / el.as_secs_f64()
    );
    store::add_meta(&db, o.n, text)
}

fn maxrss_mib() -> f64 {
    // SAFETY-free: getrusage through the libc crate's safe wrapper is
    // not offered, so read /proc instead.
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmHWM:"))
                .and_then(|l| l.split_whitespace().nth(1).and_then(|v| v.parse::<f64>().ok()))
        })
        .map_or(-1.0, |kb| kb / 1024.0)
}

/// What one hook process does: open, append one batch, search once,
/// close; reports peak RSS (NFR-09: ≤ 50 MiB) and wall time.
pub fn hook(o: &Opts) -> Result<()> {
    check_batch(o.batch)?;
    let start = Instant::now();
    let mut c = Corpus::from_seeds(o.seed, o.seed + 2000);
    let evs: Vec<Event> = (0..o.batch).map(|_| c.next()).collect();
    let before = maxrss_mib();
    let mut db = store::open(&o.db, o.key, false)?;
    let mut text = 0i64;
    {
        let tx = store::begin(&mut db)?;
        let (mut seq, mut prev) = store::last_hash(&tx)?;
        for e in &evs {
            seq += 1;
            let mut e = e.clone();
            e.source = 2_000_000;
            e.line = seq;
            text += e.body.len() as i64;
            prev = store::append(&tx, seq, &e, &prev)?;
        }
        tx.commit()?;
    }
    search_from(&db, &format!("\"{}\" \"{}\"", c.words[1200], c.words[80]), 0)?;
    db.close().map_err(|(_, e)| e)?;
    println!(
        "RESULT hook batch={} wall={:?} maxrss_before_open={before:.1}MiB maxrss={:.1}MiB",
        o.batch,
        start.elapsed(),
        maxrss_mib()
    );
    let mdb = store::open(&o.db, o.key, false)?;
    store::add_meta(&mdb, evs.len() as i64, text)
}

/// Runs `f` on `db` with a deadline: a timer thread fires SQLite's
/// interrupt through rusqlite's InterruptHandle, the way the core
/// would carry ENG-03's deadline into a query.
fn with_deadline<T>(
    db: &Connection,
    deadline: Duration,
    f: impl FnOnce() -> rusqlite::Result<T>,
) -> (rusqlite::Result<T>, Duration) {
    let handle = db.get_interrupt_handle();
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    let timer = thread::spawn(move || {
        if done_rx.recv_timeout(deadline).is_err() {
            handle.interrupt();
        }
    });
    let t = Instant::now();
    let res = f();
    let el = t.elapsed();
    let _ = done_tx.send(());
    let _ = timer.join();
    (res, el)
}

fn is_interrupt(e: &rusqlite::Error) -> bool {
    e.sqlite_error_code() == Some(ErrorCode::OperationInterrupted)
}

fn err_string<T>(r: &rusqlite::Result<T>) -> String {
    r.as_ref().err().map(|e| e.to_string()).unwrap_or_default()
}

/// A deadline stops a running query promptly and leaves the connection
/// usable; an interrupted write transaction rolls back.
pub fn cancel(o: &Opts) -> Result<()> {
    let db = store::open(&o.db, o.key, false)?;
    let common = Corpus::new(o.seed).words[21].clone();
    let queries = [
        ("cte", "WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM c) SELECT count(*) FROM (SELECT x FROM c LIMIT 2000000000)".to_owned()),
        ("fts_common", format!("SELECT count(*) FROM (SELECT rowid AS s FROM events_fts WHERE events_fts MATCH '\"{common}\"' ORDER BY rank LIMIT 20)")),
        ("scan", "SELECT count(*) FROM events WHERE body LIKE '%zzqq%'".to_owned()),
    ];
    for (name, sql) in &queries {
        for deadline in [Duration::from_millis(50), Duration::from_millis(250)] {
            let (res, el) =
                with_deadline(&db, deadline, || db.query_row(sql, [], |r| r.get::<_, i64>(0)));
            let usable = db.query_row("SELECT count(*) FROM meta", [], |r| r.get::<_, i64>(0)).is_ok();
            println!(
                "RESULT cancel query={name} deadline={deadline:?} returned_after={el:.1?} overrun={:.1?} err={:?} is_interrupt={} conn_usable={usable}",
                el.saturating_sub(deadline),
                err_string(&res),
                res.as_ref().err().is_some_and(is_interrupt),
            );
        }
    }
    // An interrupted write: a sentinel row written first tells a
    // rolled-back transaction from one where only the statement failed.
    db.execute("DELETE FROM meta WHERE k = 'sentinel'", [])?;
    let count = || db.query_row("SELECT count(*) FROM meta", [], |r| r.get::<_, i64>(0));
    let before = count()?;
    db.execute_batch("BEGIN IMMEDIATE")?;
    db.execute("INSERT INTO meta(k, v) VALUES('sentinel', 1)", [])?;
    let (werr, _) = with_deadline(&db, Duration::from_millis(100), || {
        db.execute(
            "INSERT INTO meta(k, v) WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM c LIMIT 50000000) SELECT 'junk' || x, x FROM c",
            [],
        )
    });
    // Try to commit: if the interrupt left the transaction open, the
    // sentinel lands and the check below fails.
    let cerr = if werr.is_ok() {
        db.execute_batch("ROLLBACK").map(|_| ())
    } else {
        db.execute_batch("COMMIT")
    };
    let after = count()?;
    let sentinels: i64 =
        db.query_row("SELECT count(*) FROM meta WHERE k = 'sentinel'", [], |r| r.get(0))?;
    println!(
        "RESULT cancel query=write_tx exec_err={:?} commit_err={:?} autocommit={} rows_before={before} rows_after={after} sentinel_rows={sentinels} rolled_back={}",
        err_string(&werr),
        err_string(&cerr),
        db.is_autocommit(),
        werr.is_err() && before == after && sentinels == 0
    );
    Ok(())
}

/// Merges the FTS5 index into one segment and reports time and size.
pub fn optimize(o: &Opts) -> Result<()> {
    let db = store::open(&o.db, o.key, false)?;
    let segs = || {
        db.query_row("SELECT count(DISTINCT segid) FROM events_fts_idx", [], |r| r.get::<_, i64>(0))
            .unwrap_or(-1)
    };
    let before = segs();
    let t = Instant::now();
    db.execute("INSERT INTO events_fts(events_fts) VALUES('optimize')", [])?;
    let el = t.elapsed();
    db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
    println!(
        "RESULT optimize segments_before={before} segments_after={} elapsed={}s",
        segs(),
        el.as_secs()
    );
    report_size(&db, o)
}
