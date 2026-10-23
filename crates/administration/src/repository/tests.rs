use std::fs;
use std::path::{Path, PathBuf};

use super::*;
use crate::home::Home;
use crate::node_identity::IdentitySource;
use key::random::OsEntropy;
use record::event::{EventKind, Provenance};
use testkit::TempDir;

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const C: &str = "3333333333333333333333333333333333333333";

/// A zlib stream of `data` in stored blocks.
fn zlib(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut rest = data;
    loop {
        let (chunk, tail) = rest.split_at(rest.len().min(65_535));
        out.push(u8::from(tail.is_empty()));
        out.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        out.extend_from_slice(&(!(chunk.len() as u16)).to_le_bytes());
        out.extend_from_slice(chunk);
        rest = tail;
        if rest.is_empty() {
            break;
        }
    }
    let (mut a, mut b) = (1_u32, 0_u32);
    for byte in data {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    out.extend_from_slice(&((b << 16) | a).to_be_bytes());
    out
}

/// A repository at `dir` whose commits are `commits`, each with its
/// parents, the newest on `main`.
fn repository(dir: &Path, commits: &[(&str, &[&str])], tip: Option<&str>) {
    let git = dir.join(".git");
    fs::create_dir_all(git.join("refs/heads")).unwrap();
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    for (id, parents) in commits {
        let mut body = format!("tree {}\n", "0".repeat(40));
        for parent in *parents {
            body.push_str(&format!("parent {parent}\n"));
        }
        body.push_str("\nmessage\n");
        let object = format!("commit {}\0{body}", body.len());
        let path = git.join("objects").join(&id[..2]).join(&id[2..]);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, zlib(object.as_bytes())).unwrap();
    }
    if let Some(tip) = tip {
        fs::write(git.join("refs/heads/main"), format!("{tip}\n")).unwrap();
    }
}

struct Fixture {
    dir: TempDir,
    store: Store,
    node: Node,
}

fn fixture() -> Fixture {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("machine-id"), "first").unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    Fixture { dir, store, node }
}

impl Fixture {
    fn path(&self, name: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn bind(&mut self, directory: &Path) -> Outcome {
        bind(
            &mut self.store,
            &self.node,
            &mut OsEntropy,
            directory,
            &|| false,
        )
        .unwrap()
    }

    /// The repository bind events the device seat holds.
    fn binds(&self) -> Vec<serde_json::Value> {
        self.store
            .log(self.node.device_writer())
            .unwrap()
            .into_iter()
            .filter(|s| s.link.header.kind == EventKind::RepositoryBind)
            .map(|s| {
                assert_eq!(s.link.header.provenance, Provenance::Structural);
                s.content.unwrap()
            })
            .collect()
    }
}

#[test]
fn a_directory_in_no_repository_binds_nothing() {
    let mut f = fixture();
    let plain = f.path("plain");

    assert_eq!(f.bind(&plain), Outcome::NoRepository);
    assert!(f.binds().is_empty());
}

#[test]
fn the_first_look_binds_the_lowest_root_commit_once_by_a_structural_event() {
    let mut f = fixture();
    let app = f.path("app");
    repository(&app, &[(B, &[]), (A, &[]), (C, &[A, B])], Some(C));

    assert_eq!(f.bind(&app), Outcome::Bound { provisional: false });
    assert_eq!(f.bind(&app), Outcome::Unchanged);

    assert_eq!(f.binds(), [serde_json::json!({"repository": A})]);
}

#[test]
fn a_second_clone_of_a_bound_repository_binds_no_second_event() {
    let mut f = fixture();
    let (one, two) = (f.path("one"), f.path("two"));
    repository(&one, &[(A, &[])], Some(A));
    repository(&two, &[(A, &[]), (B, &[A])], Some(B));

    assert_eq!(f.bind(&one), Outcome::Bound { provisional: false });
    assert_eq!(f.bind(&two), Outcome::Bound { provisional: false });

    assert_eq!(f.binds().len(), 1);
}

#[test]
fn a_clone_holding_a_bound_commit_takes_that_identity() {
    let mut f = fixture();
    let (one, two) = (f.path("one"), f.path("two"));
    repository(&one, &[(B, &[])], Some(B));
    // A lower root joined this clone's history; the bound commit stays.
    repository(&two, &[(A, &[]), (B, &[]), (C, &[A, B])], Some(C));

    f.bind(&one);
    assert_eq!(f.bind(&two), Outcome::Bound { provisional: false });

    assert_eq!(f.binds(), [serde_json::json!({"repository": B})]);
}

#[test]
fn a_repository_with_no_commit_gets_a_provisional_identity_and_binds_later() {
    let mut f = fixture();
    let empty = f.path("empty");
    repository(&empty, &[], None);

    assert_eq!(f.bind(&empty), Outcome::Bound { provisional: true });
    assert_eq!(f.bind(&empty), Outcome::Unchanged);
    let binds = f.binds();
    assert_eq!(binds.len(), 1);
    assert_eq!(
        binds[0]["repository_provisional"].as_str().unwrap().len(),
        32
    );

    // A commit makes it a change; nothing is rebound or rewritten.
    repository(&empty, &[(A, &[])], Some(A));
    assert_eq!(f.bind(&empty), Outcome::Changed);
    assert_eq!(f.binds().len(), 1);
}

#[test]
fn a_change_in_the_identity_a_directory_resolves_to_is_reported_every_time_and_relinks_nothing() {
    let mut f = fixture();
    let app = f.path("app");
    repository(&app, &[(A, &[])], Some(A));
    f.bind(&app);
    fs::remove_dir_all(app.join(".git")).unwrap();
    repository(&app, &[(B, &[])], Some(B));

    assert_eq!(f.bind(&app), Outcome::Changed);
    assert_eq!(f.bind(&app), Outcome::Changed);
    assert_eq!(f.binds().len(), 1);
}

#[test]
fn a_clone_whose_git_data_cannot_be_read_is_unreadable() {
    let mut f = fixture();
    let app = f.path("app");
    repository(&app, &[(A, &[])], Some(A));
    fs::write(
        app.join(".git/config"),
        "[extensions]\n\tobjectformat = sha256\n",
    )
    .unwrap();

    assert_eq!(f.bind(&app), Outcome::Unreadable);
    assert!(f.binds().is_empty());
}

#[test]
fn no_name_this_node_keeps_reveals_the_directory_or_the_identity() {
    let mut f = fixture();
    let app = f.path("app");
    repository(&app, &[(A, &[])], Some(A));
    f.bind(&app);

    let conn = rusqlite::Connection::open(f.dir.path().join(".parinn/store.db")).unwrap();
    let mut statement = conn
        .prepare("SELECT directory, identity FROM repository_directories")
        .unwrap();
    let rows: Vec<(String, String)> = statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();

    assert_eq!(rows.len(), 1);
    for (directory, identity) in rows {
        assert!(!identity.contains(A) && !directory.contains("app"));
        assert_eq!((directory.len(), identity.len()), (64, 64));
    }
}
