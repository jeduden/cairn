//! Times hook handlers the way the harness sees them: from spawning
//! the process to its exit, hook input on stdin, n runs one after
//! another. Also the fleet load NFR-01 names: ten harnesses writing,
//! each one's MCP server ingesting, and the room view open.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::store::{self, Event, Result};

pub fn percentile(sorted: &[Duration], p: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let rank = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

pub fn lat(args: &[String], fixture: &Path, n: usize) -> Result<()> {
    let input = std::fs::read(fixture)?;
    let exe = std::env::current_exe()?;
    let mut ds = Vec::with_capacity(n);
    for _ in 0..n {
        let start = Instant::now();
        let mut child = Command::new(&exe)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.stdin.take().ok_or("no stdin")?.write_all(&input)?;
        let status = child.wait()?;
        ds.push(start.elapsed());
        if !status.success() {
            return Err(format!("hook handler exited {status}").into());
        }
    }
    ds.sort();
    println!(
        "RESULT lat {} n={n} p50={:.1?} p95={:.1?} p99={:.1?} max={:.1?}",
        args.join(" "),
        percentile(&ds, 50.0),
        percentile(&ds, 95.0),
        percentile(&ds, 99.0),
        ds[ds.len() - 1]
    );
    Ok(())
}

/// Ten writers append five events every 200 ms each, as ten harnesses'
/// MCP servers ingesting busy runs; one reader reads the newest 100
/// events every 250 ms, as the room view does. Runs for `secs`.
pub fn fleet(db: &Path, writers: i64, secs: u64) -> Result<()> {
    let stop = Arc::new(AtomicBool::new(false));
    let appended = Arc::new(AtomicU64::new(0));
    let reads = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::new();
    for w in 0..writers {
        let (db, stop, appended) = (db.to_owned(), stop.clone(), appended.clone());
        handles.push(std::thread::spawn(move || -> std::result::Result<(), String> {
            let mut c = store::open(&db).map_err(|e| e.to_string())?;
            let source = 3_000_000 + w;
            let mut line = store::next_line(&c, source).map_err(|e| e.to_string())?;
            while !stop.load(Ordering::Relaxed) {
                let tx = store::begin(&mut c).map_err(|e| e.to_string())?;
                for _ in 0..5 {
                    let body = format!("writer {w} line {line}: cargo test output ok passed");
                    store::append(&tx, &Event { source, line, kind: "tool_result", tool: Some("Bash"), body: &body })
                        .map_err(|e| e.to_string())?;
                    line += 1;
                }
                tx.commit().map_err(|e| e.to_string())?;
                appended.fetch_add(5, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(200));
            }
            Ok(())
        }));
    }
    {
        let (db, stop, reads) = (db.to_owned(), stop.clone(), reads.clone());
        handles.push(std::thread::spawn(move || -> std::result::Result<(), String> {
            let c = store::open(&db).map_err(|e| e.to_string())?;
            while !stop.load(Ordering::Relaxed) {
                let mut st = c
                    .prepare_cached("SELECT seq, kind, body FROM events ORDER BY seq DESC LIMIT 100")
                    .map_err(|e| e.to_string())?;
                let n = st.query_map([], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?.count();
                reads.fetch_add(n as u64 / 100, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(250));
            }
            Ok(())
        }));
    }
    std::thread::sleep(Duration::from_secs(secs));
    stop.store(true, Ordering::Relaxed);
    for h in handles {
        h.join().map_err(|_| "a fleet thread panicked")??;
    }
    println!(
        "RESULT fleet writers={writers} secs={secs} appended={} room_view_reads={}",
        appended.load(Ordering::Relaxed),
        reads.load(Ordering::Relaxed)
    );
    Ok(())
}
