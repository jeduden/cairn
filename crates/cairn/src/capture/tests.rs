use std::path::PathBuf;

use audit::log::Log;
use mcp::tool_call::Runs;
use serde_json::json;
use store::Observation;
use testkit::TempDir;

use super::*;

const SESSION: &str = "5e55a0de-0000-4000-8000-0000000000d1";
const HARNESS: u32 = 77;

struct Fixture {
    dir: TempDir,
    home: Home,
    shared: Arc<Shared>,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let home = Home::locate(Some(dir.path().join(".cairn").as_os_str()), None).unwrap();
        home.create().unwrap();
        std::fs::create_dir_all(dir.path().join("projects").join("p")).unwrap();
        Self {
            dir,
            home,
            shared: Arc::new(Shared::default()),
        }
    }

    fn projects(&self) -> PathBuf {
        self.dir.path().join("projects").join("p")
    }

    fn main(&self) -> PathBuf {
        self.projects().join(format!("{SESSION}.jsonl"))
    }

    fn subagent(&self, id: &str) -> PathBuf {
        let dir = self.projects().join(SESSION).join("subagents");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(format!("agent-{id}.jsonl"))
    }

    fn running(&self, harness: Option<u32>, limit: Option<usize>) -> Running {
        let roots = AllowedRoots::default().with(&self.dir.path().join("projects"), Access::Read);
        Running {
            capture: Capture {
                home: self.home.clone(),
                roots,
                redactor: Redactor::new(&[]).unwrap().0,
                inputs: ingest::Inputs::default(),
                harness,
                limit,
                clock: Box::new(|| 1_760_000_000),
                shared: self.shared.clone(),
            },
            seats: RunSeats::default(),
            sessions: Vec::new(),
            transcripts: BTreeMap::new(),
            refused: BTreeSet::new(),
            lock: None,
        }
    }

    /// Holds a `SessionStart` hook observation of `session` naming
    /// `path`, of the harness process `harness`.
    fn bind(&self, session: &str, path: &Path, harness: u32) {
        let mut store = Store::open(&self.home.store()).unwrap();
        let observation = Observation {
            hook: "SessionStart".to_owned(),
            harness_session: session.to_owned(),
            transcript_path: path.to_string_lossy().into_owned(),
            cwd: "/w".to_owned(),
            offset: Some(0),
            source: Some("startup".to_owned()),
            prompt_id: None,
            attended: false,
            ancestry: Some((1, false)),
            harness: Some(harness),
            at: 1,
        };
        store.hold_observation(&observation, None).unwrap();
    }

    /// The counter of each audit entry.
    fn counted(&self) -> Vec<String> {
        Log::new(self.home.audit())
            .entries()
            .unwrap()
            .into_iter()
            .filter_map(|e| e.counter)
            .collect()
    }

    fn locks(&self) -> Vec<PathBuf> {
        std::fs::read_dir(self.home.root().join(LOCKS))
            .map(|l| l.flatten().map(|e| e.path()).collect())
            .unwrap_or_default()
    }
}

/// A transcript line of the main run, or of the subagent `agent`,
/// carrying the tool call `tool_use_id` when given.
fn line(n: u64, tool_use_id: Option<&str>) -> String {
    let value = match tool_use_id {
        Some(id) => json!({
            "type": "assistant",
            "n": n,
            "message": {"content": [{"type": "tool_use", "id": id, "name": "event_search"}]},
        }),
        // In Claude Code 2.1.296's recorded shape, so nothing drifts.
        None => json!({"type": "atis-latch", "atis": n.to_string(), "sessionId": "s"}),
    };
    format!("{value}\n")
}

#[test]
fn a_run_is_named_by_its_harness_session_and_a_subagent_by_its_agent_id_too() {
    let main = RunKey {
        agent_id: None,
        harness_session: "s".to_owned(),
    };
    let agent = RunKey {
        agent_id: Some("a1".to_owned()),
        ..main.clone()
    };

    assert_eq!(run_name(&main), "s");
    assert_eq!(run_name(&agent), "s/a1");
}

#[test]
fn only_an_assistant_line_s_tool_use_blocks_carry_tool_use_ids() {
    let call = json!({"type": "assistant", "message": {"content": [
        {"type": "text", "text": "x"},
        {"type": "tool_use", "id": "toolu_1"},
        {"type": "tool_use"},
    ]}});
    let result = json!({"type": "user", "message": {"content": [
        {"type": "tool_result", "tool_use_id": "toolu_1"},
    ]}});

    assert_eq!(tool_use_ids(&call), ["toolu_1"]);
    assert!(tool_use_ids(&result).is_empty());
    assert!(tool_use_ids(&json!({"type": "assistant"})).is_empty());
}

#[test]
fn a_finder_returns_the_runs_read_or_none_once_its_wait_passes() {
    let shared = Arc::new(Shared::default());
    shared.read("toolu_1".to_owned(), "s".to_owned());
    shared.read("toolu_1".to_owned(), "s".to_owned());
    let finder = Finder(shared.clone());

    assert!(!finder.serving());
    assert_eq!(finder.find("toolu_1", Duration::from_secs(5)).0, ["s"]);
    let (none, waited) = finder.find("toolu_2", Duration::from_millis(30));
    assert!(none.is_empty());
    assert!(waited >= Duration::from_millis(30));
    shared.stop();
    assert!(shared.stopping());
}

#[test]
fn live_ingest_takes_only_its_own_harness_process_s_binding_and_reads_its_tool_uses() {
    let f = Fixture::new();
    std::fs::write(
        f.main(),
        format!("{}{}", line(1, None), line(2, Some("toolu_m"))),
    )
    .unwrap();
    std::fs::write(f.subagent("a1"), line(1, Some("toolu_a"))).unwrap();
    f.bind("other", &f.projects().join("other.jsonl"), HARNESS + 1);
    let mut running = f.running(Some(HARNESS), None);

    running.bind();
    running.poll();
    assert!(!f.shared.serving.load(Ordering::SeqCst), "no binding yet");
    assert!(running.seats.runs().is_empty());

    f.bind(SESSION, &f.main(), HARNESS);
    running.bind();
    running.poll();

    assert!(f.shared.serving.load(Ordering::SeqCst));
    assert_eq!(
        running.seats.runs().len(),
        2,
        "the main run's and the subagent's"
    );
    assert_eq!(f.shared.runs_of("toolu_m"), [SESSION]);
    assert_eq!(f.shared.runs_of("toolu_a"), [format!("{SESSION}/a1")]);
    assert_eq!(f.locks().len(), 1, "its server lock");
    running.poll();
    running.stop();
    assert!(f.locks().is_empty(), "removed as it stopped");
    assert!(!f.counted().contains(&"ingest_remainder_left".to_owned()));
}

#[test]
fn a_stop_short_of_a_transcript_s_end_is_audited_by_the_server() {
    let f = Fixture::new();
    std::fs::write(f.main(), (1..=4).map(|n| line(n, None)).collect::<String>()).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), Some(2));

    running.bind();
    running.poll();
    running.stop();

    assert_eq!(f.counted(), ["ingest_remainder_left"]);
    let detail = Log::new(f.home.audit()).entries().unwrap()[0]
        .detail
        .clone();
    assert!(detail.ends_with(SESSION), "{detail}");
}

#[test]
fn a_killed_server_s_remainder_is_audited_once_by_the_next_reader() {
    let f = Fixture::new();
    std::fs::write(f.main(), (1..=4).map(|n| line(n, None)).collect::<String>()).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), Some(2));
    running.bind();
    running.poll();
    let audit = |o: &Occurrence| {
        Log::new(f.home.audit()).append(o, 1).unwrap();
    };

    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0, "it still runs");
    // Killed: its lock is released, its file left.
    drop(running);
    assert_eq!(count_stopped(&f.home, "ingest", &audit), 1);
    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0);
    assert_eq!(f.counted(), ["ingest_remainder_left"]);
    assert!(f.locks().is_empty());
}

#[test]
fn no_lock_directory_and_files_that_are_not_locks_are_passed_over() {
    let f = Fixture::new();
    let audit = |_: &Occurrence| panic!("nothing to audit");
    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0);

    std::fs::create_dir_all(f.home.root().join(LOCKS)).unwrap();
    std::fs::write(f.home.root().join(LOCKS).join("x.staged"), "").unwrap();
    std::fs::create_dir(f.home.root().join(LOCKS).join("d.lock")).unwrap();
    assert_eq!(count_stopped(&f.home, "ingest", &audit), 0);
}

#[test]
fn a_transcript_outside_the_roots_or_a_link_swapped_in_is_refused_once() {
    let f = Fixture::new();
    let outside = f.dir.path().join("elsewhere.jsonl");
    std::fs::write(&outside, line(1, None)).unwrap();
    f.bind(SESSION, &outside, HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    assert_eq!(f.counted(), ["path_outside_allowed_roots"]);

    let f = Fixture::new();
    std::fs::write(f.main(), line(1, None)).unwrap();
    let subagent = f.subagent("a2");
    std::fs::write(&subagent, line(1, None)).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    let secret = f.dir.path().join("id_ed25519");
    std::fs::write(&secret, "key\n").unwrap();
    std::fs::remove_file(&subagent).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&secret, &subagent).unwrap();
    running.poll();
    running.poll();

    assert_eq!(f.counted(), ["path_outside_allowed_roots"]);
    assert_eq!(running.transcripts.len(), 1, "only the main run's is read");
}

#[test]
fn a_transcript_not_written_yet_waits_and_one_that_fails_is_audited() {
    let f = Fixture::new();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    assert!(f.counted().is_empty(), "not written yet");

    std::fs::write(f.main(), [0xff, b'\n']).unwrap();
    running.poll();
    assert_eq!(f.counted(), ["live_ingest_failed"]);
}

#[test]
fn without_a_harness_process_nothing_is_taken() {
    let f = Fixture::new();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(None, None);

    running.bind();

    assert!(running.sessions.is_empty());
    assert!(!f.shared.serving.load(Ordering::SeqCst));
}

#[test]
fn what_live_ingest_found_is_audited_entry_by_entry() {
    let f = Fixture::new();
    std::fs::write(
        f.main(),
        format!(
            "not json\n{}\n",
            json!({"type": "atis-latch", "atis": "AKIAIOSFODNN7EXAMPLE", "sessionId": "s"})
        ),
    )
    .unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);
    running.bind();
    running.poll();
    let first = f.counted();
    assert!(first.contains(&"events_unparsed".to_owned()), "{first:?}");
    assert!(first.contains(&"redacted".to_owned()), "{first:?}");

    // The transcript is rewritten: a new generation.
    std::fs::write(f.main(), line(9, None)).unwrap();
    running.poll();
    assert!(
        f.counted()
            .contains(&"new_transcript_generation".to_owned())
    );
}

#[test]
fn a_witnessed_line_repeating_another_transcript_is_audited() {
    let f = Fixture::new();
    let uuid_line = format!(
        "{}\n",
        json!({"type": "mode", "mode": "normal", "uuid": "u-shared"})
    );
    let other = f.projects().join("other.jsonl");
    std::fs::write(&other, &uuid_line).unwrap();
    ingest::ingest_path(
        &f.home,
        &other,
        &Redactor::new(&[]).unwrap().0,
        &mut OsEntropy,
    )
    .unwrap();
    std::fs::write(f.main(), &uuid_line).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let mut running = f.running(Some(HARNESS), None);

    running.bind();
    running.poll();

    let counted: Vec<String> = f
        .counted()
        .into_iter()
        .filter(|c| c != "format_drift")
        .collect();
    assert_eq!(counted, ["witnessed_repeat_recorded"]);
}

#[test]
fn a_server_lock_rewritten_keeps_its_path_and_one_it_cannot_write_keeps_the_old() {
    let f = Fixture::new();
    let first = ServerLock::hold(&f.home, &[f.main()], None).unwrap();
    let path = first.path.clone();
    let second = ServerLock::hold(&f.home, &[f.main(), f.subagent("a")], Some(first)).unwrap();
    assert_eq!(second.path, path);
    assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 2);

    std::fs::remove_dir_all(f.home.root().join(LOCKS)).unwrap();
    let kept = ServerLock::hold(&f.home, &[f.main()], Some(second)).unwrap();
    assert_eq!(kept.path, path);
    assert!(ServerLock::hold(&f.home, &[], None).is_some());
}

#[test]
fn live_ingest_starts_bound_and_stops_when_told() {
    let f = Fixture::new();
    std::fs::write(f.main(), line(1, None)).unwrap();
    f.bind(SESSION, &f.main(), HARNESS);
    let running = f.running(Some(HARNESS), None);
    let capture = Capture {
        home: running.capture.home.clone(),
        roots: running.capture.roots.clone(),
        redactor: running.capture.redactor.clone(),
        inputs: ingest::Inputs::default(),
        harness: Some(HARNESS),
        limit: None,
        clock: Box::new(|| 1),
        shared: f.shared.clone(),
    };

    let started = capture.start();
    assert_eq!(started.seats.runs().len(), 1, "bound before the first call");
    f.shared.stop();
    started.run();

    assert!(f.locks().is_empty());
}
