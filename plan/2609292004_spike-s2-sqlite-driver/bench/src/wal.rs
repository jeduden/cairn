//! WAL correctness under concurrent writer processes (NFR-05: ≥ 50),
//! ADR-01's daemonless model: each writer is its own OS process
//! opening the store directly, while readers search throughout.

use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

use crate::Opts;
use crate::cmds::{search_from, stats};
use crate::corpus::{Corpus, Rng};
use crate::store::{self, Result};

pub fn wal(o: &Opts) -> Result<()> {
    for suf in ["", "-wal", "-shm"] {
        let mut p = o.db.clone().into_os_string();
        p.push(suf);
        let _ = std::fs::remove_file(p);
    }
    {
        let db = store::open(&o.db, o.key, false)?;
        store::create_schema(&db)?;
        store::set_meta(&db, "events", 0)?;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let (reads, read_errs) = (Arc::new(AtomicI64::new(0)), Arc::new(AtomicI64::new(0)));
    let words = Arc::new(Corpus::new(o.seed).words);
    let readers: Vec<_> = (0..4u64)
        .map(|r| {
            let (stop, reads, read_errs, words, o) =
                (stop.clone(), reads.clone(), read_errs.clone(), words.clone(), o.clone());
            thread::spawn(move || {
                let Ok(rdb) = store::open(&o.db, o.key, true) else {
                    read_errs.fetch_add(1, Ordering::Relaxed);
                    return;
                };
                let mut rng = Rng::new(r, 3);
                while !stop.load(Ordering::Relaxed) {
                    let q = format!("\"{}\"", words[20 + rng.int_n(2000)]);
                    if search_from(&rdb, &q, 0).is_err() {
                        read_errs.fetch_add(1, Ordering::Relaxed);
                    }
                    reads.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();

    let start = Instant::now();
    let exe = std::env::current_exe()?;
    let workers: Vec<_> = (0..o.workers)
        .map(|w| {
            let mut cmd = Command::new(&exe);
            cmd.args(["walworker", "-db"])
                .arg(&o.db)
                .args(["-n", &o.n.to_string(), "-id", &w.to_string(), "-seed", &o.seed.to_string()]);
            if o.key {
                cmd.arg("-key");
            }
            thread::spawn(move || cmd.output())
        })
        .collect();
    let mut outs = Vec::new();
    for w in workers {
        outs.push(w.join().map_err(|_| "worker thread panicked")?);
    }
    let el = start.elapsed();
    stop.store(true, Ordering::Relaxed);
    for r in readers {
        let _ = r.join();
    }

    let (mut failed, mut retries, mut worst) = (0, 0i64, 0f64);
    for (w, out) in outs.into_iter().enumerate() {
        let out = out?;
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        if !out.status.success() {
            failed += 1;
            println!("worker {w} failed: {}{}", text, String::from_utf8_lossy(&out.stderr));
            continue;
        }
        for f in text.split_whitespace() {
            if let Some(v) = f.strip_prefix("retries=") {
                retries += v.parse::<i64>().unwrap_or(0);
            }
            if let Some(v) = f.strip_prefix("max=").and_then(|v| v.strip_suffix("ms")) {
                worst = worst.max(v.parse::<f64>().unwrap_or(0.0));
            }
        }
    }
    println!(
        "RESULT wal writers={} tx_per_writer={} elapsed={el:.1?} failed_writers={failed} busy_retries={retries} worst_commit={worst:.2}ms reads={} read_errors={}",
        o.workers,
        o.n,
        reads.load(Ordering::Relaxed),
        read_errs.load(Ordering::Relaxed)
    );
    let verified = verify(o, o.workers * o.n);
    if failed > 0 || read_errs.load(Ordering::Relaxed) > 0 {
        return Err(format!("concurrency probe failed: {failed} writers, {} reads", read_errs.load(Ordering::Relaxed)).into());
    }
    verified
}

/// Checks the record after the concurrent run: every transaction
/// present, seq gap-free, the hash chain recomputed end to end, and
/// both integrity checks clean.
fn verify(o: &Opts, want_tx: i64) -> Result<()> {
    let db = store::open(&o.db, o.key, false)?;
    let (n, lo, hi): (i64, i64, i64) =
        db.query_row("SELECT count(*), min(seq), max(seq) FROM events", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
    let txs: i64 = db.query_row(
        "SELECT count(DISTINCT source * 1000000 + line / 3) FROM events",
        [],
        |r| r.get(0),
    )?;
    let mut st = db.prepare("SELECT seq, body, prev_hash, hash FROM events ORDER BY seq")?;
    let mut rows = st.query([])?;
    let mut prev = vec![0u8; 32];
    let (mut chain_ok, mut walked) = (true, 0i64);
    while let Some(row) = rows.next()? {
        walked += 1;
        let (seq, body, ph, h): (i64, String, Vec<u8>, Vec<u8>) =
            (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);
        let mut s = Sha256::new();
        s.update(&ph);
        s.update(body.as_bytes());
        if ph != prev || s.finalize().as_slice() != h.as_slice() {
            if chain_ok {
                println!("chain broken at seq {seq}");
            }
            chain_ok = false;
        }
        prev = h;
    }
    drop(rows);
    drop(st);
    if walked != n {
        chain_ok = false;
    }
    let integ: String = db.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    let fts = db.execute("INSERT INTO events_fts(events_fts, rank) VALUES('integrity-check', 1)", []);
    let gap_free = lo == 1 && hi == n;
    println!(
        "RESULT walverify events={n} transactions={txs}/{want_tx} seq_min={lo} seq_max={hi} gap_free={gap_free} chain_ok={chain_ok} integrity={integ} fts_integrity_err={:?}",
        fts.as_ref().err().map(|e| e.to_string()).unwrap_or_default()
    );
    let mut errs = Vec::new();
    if txs != want_tx {
        errs.push("transactions missing".to_owned());
    }
    if !gap_free {
        errs.push("seq not gap-free".to_owned());
    }
    if !chain_ok {
        errs.push("hash chain broken".to_owned());
    }
    if integ != "ok" {
        errs.push(format!("integrity_check: {integ}"));
    }
    if let Err(e) = fts {
        errs.push(format!("fts integrity-check: {e}"));
    }
    if errs.is_empty() { Ok(()) } else { Err(errs.join("; ").into()) }
}

/// One writer process: o.n transactions of 1–3 events, each allocating
/// seq and chaining onto the tail inside BEGIN IMMEDIATE, on a fresh
/// connection per transaction as a hook process opens one.
pub fn worker(o: &Opts) -> Result<()> {
    let mut c = Corpus::from_seeds(o.seed, o.seed + o.id as u64 * 7919);
    let mut r = Rng::new(o.id as u64, 5);
    let (mut retries, mut lat) = (0, Vec::new());
    for i in 0..o.n {
        let k = 1 + r.int_n(3);
        let evs: Vec<_> = (0..k)
            .map(|j| {
                let mut e = c.next();
                e.source = 1000 + o.id;
                e.line = i * 3 + j as i64;
                e
            })
            .collect();
        let mut attempt = 0;
        loop {
            let t = Instant::now();
            match write_tx(o, &evs) {
                Ok(()) => {
                    lat.push(t.elapsed());
                    break;
                }
                Err(e) => {
                    let msg = e.to_string().to_lowercase();
                    if attempt >= 20 || !(msg.contains("busy") || msg.contains("locked")) {
                        return Err(format!("worker {} tx {i}: {e}", o.id).into());
                    }
                    retries += 1;
                    attempt += 1;
                    thread::sleep(Duration::from_millis(r.int_n(20) as u64));
                }
            }
        }
    }
    println!("worker={} retries={retries} {}", o.id, stats(&lat));
    Ok(())
}

fn write_tx(o: &Opts, evs: &[crate::corpus::Event]) -> Result<()> {
    let mut db = store::open(&o.db, o.key, false)?;
    let tx = store::begin(&mut db)?;
    let (mut seq, mut prev) = store::last_hash(&tx)?;
    for e in evs {
        seq += 1;
        prev = store::append(&tx, seq, e, &prev)?;
    }
    tx.commit()?;
    Ok(())
}
