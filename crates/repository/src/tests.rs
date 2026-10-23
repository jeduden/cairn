use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use testkit::TempDir;

use super::{CommitId, Error, RepositoryIdentity, git_dir, has_commit, identity, root_commit};
use crate::fixture::zlib_stored;

fn id(n: u8) -> CommitId {
    CommitId::from_bytes(&[n; 20])
}

fn write(git: &Path, name: &str, content: &str) {
    let path = git.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn commit(git: &Path, id: &CommitId, parents: &[&CommitId]) {
    let mut body = String::from("tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\n");
    for parent in parents {
        body.push_str(&format!("parent {}\n", parent.as_str()));
    }
    body.push_str("author A <a@example.com> 1 +0000\n\nmessage\n");
    let (fan, rest) = id.as_str().split_at(2);
    let path = git.join("objects").join(fan).join(rest);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let header = format!("commit {}\0", body.len());
    fs::write(
        path,
        zlib_stored(&[header.as_bytes(), body.as_bytes()].concat()),
    )
    .unwrap();
}

fn checked_out(git: &Path, tip: &CommitId) {
    write(git, "HEAD", &format!("{}\n", tip.as_str()));
}

fn root(git: &Path) -> Result<Option<CommitId>, Error> {
    root_commit(git, &|| false)
}

/// Every file under `dir` with its bytes and modification time, and
/// every directory.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Option<(Vec<u8>, SystemTime)>> {
    let mut seen = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(path) = pending.pop() {
        let meta = fs::symlink_metadata(&path).unwrap();
        if meta.is_dir() {
            pending.extend(fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
            seen.insert(path, None);
        } else {
            seen.insert(
                path.clone(),
                Some((fs::read(&path).unwrap(), meta.modified().unwrap())),
            );
        }
    }
    seen
}

#[test]
fn the_root_of_a_linear_history_names_the_repository() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(1), &[]);
    commit(dir.path(), &id(2), &[&id(1)]);
    checked_out(dir.path(), &id(2));
    assert_eq!(root(dir.path()).unwrap(), Some(id(1)));
    assert_eq!(
        identity(dir.path(), &|| false).unwrap(),
        RepositoryIdentity::Bound(id(1))
    );
}

#[test]
fn the_lowest_of_two_roots_names_the_repository() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(8), &[]);
    commit(dir.path(), &id(3), &[]);
    commit(dir.path(), &id(9), &[&id(8), &id(3)]);
    checked_out(dir.path(), &id(9));
    assert_eq!(root(dir.path()).unwrap(), Some(id(3)));
}

#[test]
fn a_repository_with_no_commit_is_provisional() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    assert_eq!(root(dir.path()).unwrap(), None);
    assert_eq!(
        identity(dir.path(), &|| false).unwrap(),
        RepositoryIdentity::Provisional
    );
}

#[test]
fn a_shallow_clone_with_no_root_is_provisional() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(2), &[&id(1)]);
    checked_out(dir.path(), &id(2));
    write(dir.path(), "shallow", &format!("{}\n", id(2).as_str()));
    assert_eq!(root(dir.path()).unwrap(), None);
    assert_eq!(
        identity(dir.path(), &|| false).unwrap(),
        RepositoryIdentity::Provisional
    );
}

#[test]
fn the_remotes_default_branch_is_the_one_walked() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(1), &[]);
    commit(dir.path(), &id(5), &[]);
    checked_out(dir.path(), &id(5));
    write(
        dir.path(),
        "refs/remotes/origin/HEAD",
        "ref: refs/remotes/origin/main\n",
    );
    write(
        dir.path(),
        "refs/remotes/origin/main",
        &format!("{}\n", id(1).as_str()),
    );
    assert_eq!(root(dir.path()).unwrap(), Some(id(1)));
}

#[test]
fn a_linked_worktree_names_the_repository_it_belongs_to() {
    let dir = TempDir::new().unwrap();
    let common = dir.path().join("main.git");
    let linked = common.join("worktrees/w");
    commit(&common, &id(1), &[]);
    commit(&common, &id(2), &[&id(1)]);
    write(&linked, "commondir", "../..\n");
    checked_out(&linked, &id(2));
    assert_eq!(root(&linked).unwrap(), Some(id(1)));
    assert!(has_commit(&linked, &id(2)).unwrap());
}

#[test]
fn a_sha256_repository_is_not_read() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "config",
        "[extensions]\n\tobjectformat = sha256\n",
    );
    checked_out(dir.path(), &id(1));
    assert!(matches!(
        root(dir.path()),
        Err(Error::UnsupportedObjectFormat)
    ));
    assert!(matches!(
        has_commit(dir.path(), &id(1)),
        Err(Error::UnsupportedObjectFormat)
    ));
    assert!(matches!(
        identity(dir.path(), &|| false),
        Err(Error::UnsupportedObjectFormat)
    ));
}

#[test]
fn a_tip_the_clone_lacks_is_missing() {
    let dir = TempDir::new().unwrap();
    checked_out(dir.path(), &id(1));
    assert!(matches!(root(dir.path()), Err(Error::Missing(_))));
}

#[test]
fn a_directory_that_is_not_a_git_directory_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("none");
    assert!(matches!(root(&missing), Err(Error::Io { .. })));
    assert!(matches!(
        has_commit(&missing, &id(1)),
        Err(Error::Io { .. })
    ));
}

#[test]
fn a_directory_with_no_head_has_no_commit() {
    let dir = TempDir::new().unwrap();
    assert_eq!(root(dir.path()).unwrap(), None);
}

#[test]
fn the_deadline_is_passed_down() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    write(
        dir.path(),
        "packed-refs",
        &format!("{} refs/heads/main\n", id(1).as_str()),
    );
    let result = root_commit(dir.path(), &|| true);
    assert!(matches!(result, Err(Error::Deadline)));
    commit(dir.path(), &id(1), &[]);
    let result = root_commit(dir.path(), &|| true);
    assert!(matches!(result, Err(Error::Deadline)));
}

#[test]
fn has_commit_tells_a_held_commit_from_an_absent_one() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(1), &[]);
    assert!(has_commit(dir.path(), &id(1)).unwrap());
    assert!(!has_commit(dir.path(), &id(2)).unwrap());
}

#[test]
fn has_commit_cannot_tell_when_alternates_could_hold_it() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "objects/info/alternates",
        "/elsewhere/objects\n",
    );
    assert!(matches!(
        has_commit(dir.path(), &id(1)),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn reading_a_repository_changes_nothing_in_it() {
    let dir = TempDir::new().unwrap();
    let git = dir.path().join("work/.git");
    commit(&git, &id(1), &[]);
    commit(&git, &id(2), &[&id(1)]);
    checked_out(&git, &id(2));
    write(
        &git,
        "packed-refs",
        &format!("{} refs/heads/x\n", id(2).as_str()),
    );
    write(&git, "shallow", "");
    write(&git, "config", "[core]\n\tbare = false\n");
    let work = dir.path().join("work/sub");
    fs::create_dir_all(&work).unwrap();
    let before = snapshot(dir.path());
    let found = git_dir(&work).unwrap().unwrap();
    assert_eq!(found, git);
    root(&found).unwrap();
    has_commit(&found, &id(1)).unwrap();
    has_commit(&found, &id(7)).unwrap();
    identity(&found, &|| false).unwrap();
    assert_eq!(snapshot(dir.path()), before);
}
