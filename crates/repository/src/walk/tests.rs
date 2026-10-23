use std::cell::Cell;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::{MAX_COMMITS, lowest_root, parents, read_shallow};
use crate::fixture::zlib_stored;
use crate::objects::Objects;
use crate::repo::Repo;
use crate::{CommitId, Error};

fn id(n: u8) -> CommitId {
    CommitId::from_bytes(&[n; 20])
}

fn put(objects: &Path, id: &CommitId, content: &[u8]) {
    let (fan, rest) = id.as_str().split_at(2);
    fs::create_dir_all(objects.join(fan)).unwrap();
    fs::write(objects.join(fan).join(rest), zlib_stored(content)).unwrap();
}

/// Stores a commit of `id` with `parents`.
fn commit(objects: &Path, id: &CommitId, parents: &[&CommitId]) {
    let mut body = String::from("tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\n");
    for parent in parents {
        body.push_str(&format!("parent {}\n", parent.as_str()));
    }
    body.push_str(
        "author A <a@example.com> 1 +0000\ncommitter A <a@example.com> 1 +0000\n\nmessage\n",
    );
    let header = format!("commit {}\0", body.len());
    put(objects, id, &[header.as_bytes(), body.as_bytes()].concat());
}

fn root(dir: &TempDir, tip: &CommitId, shallow: &[&CommitId]) -> Result<Option<CommitId>, Error> {
    let objects = Objects::open(dir.path()).unwrap();
    let shallow: BTreeSet<CommitId> = shallow.iter().map(|id| (*id).clone()).collect();
    lowest_root(&objects, &shallow, tip.clone(), &|| false, MAX_COMMITS)
}

fn corrupt<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Corrupt(_)))
}

#[test]
fn a_commit_without_parents_has_none() {
    let data = b"tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\nauthor A 1 +0000\n\nmessage\n";
    assert!(parents(data).unwrap().is_empty());
}

#[test]
fn each_parent_header_names_a_parent() {
    let one = format!("tree t\nparent {}\nauthor A\n\nm", id(1).as_str());
    assert_eq!(parents(one.as_bytes()).unwrap(), [id(1)]);
    let octopus = format!(
        "tree t\nparent {}\nparent {}\nparent {}\nauthor A\n\nm",
        id(1).as_str(),
        id(2).as_str(),
        id(3).as_str()
    );
    assert_eq!(parents(octopus.as_bytes()).unwrap(), [id(1), id(2), id(3)]);
}

#[test]
fn a_parent_line_in_the_message_is_not_a_parent() {
    let text = format!("tree t\nauthor A\n\nparent {}\n", id(1).as_str());
    assert!(parents(text.as_bytes()).unwrap().is_empty());
    let signed = format!(
        "tree t\nauthor A\ngpgsig -----BEGIN\n parent {}\n -----END\n\nm",
        id(1).as_str()
    );
    assert!(parents(signed.as_bytes()).unwrap().is_empty());
}

#[test]
fn a_parent_header_that_is_not_an_id_is_corrupt() {
    for text in [
        "parent \n\n",
        "parent abc\n\n",
        "parent zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz\n\n",
        "parent 1111111111111111111111111111111111111111 \n\n",
    ] {
        assert!(corrupt(parents(text.as_bytes())), "{text:?}");
    }
}

#[test]
fn a_commit_without_a_blank_line_still_gives_its_parents() {
    let text = format!("tree t\nparent {}", id(1).as_str());
    assert_eq!(parents(text.as_bytes()).unwrap(), [id(1)]);
}

#[test]
fn a_linear_history_ends_at_its_one_root() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(1), &[]);
    commit(dir.path(), &id(2), &[&id(1)]);
    commit(dir.path(), &id(3), &[&id(2)]);
    assert_eq!(root(&dir, &id(3), &[]).unwrap(), Some(id(1)));
    assert_eq!(root(&dir, &id(1), &[]).unwrap(), Some(id(1)));
}

#[test]
fn the_lowest_of_several_roots_wins_whatever_the_order() {
    for (first, second) in [(1, 9), (9, 1)] {
        let dir = TempDir::new().unwrap();
        commit(dir.path(), &id(1), &[]);
        commit(dir.path(), &id(9), &[]);
        commit(dir.path(), &id(4), &[&id(first)]);
        commit(dir.path(), &id(5), &[&id(second)]);
        commit(dir.path(), &id(6), &[&id(4), &id(5)]);
        assert_eq!(root(&dir, &id(6), &[]).unwrap(), Some(id(1)), "{first}");
        commit(dir.path(), &id(7), &[&id(5), &id(4)]);
        assert_eq!(root(&dir, &id(7), &[]).unwrap(), Some(id(1)), "{second}");
    }
}

#[test]
fn a_shared_ancestor_is_visited_once_and_a_cycle_ends() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(1), &[]);
    commit(dir.path(), &id(2), &[&id(1)]);
    commit(dir.path(), &id(3), &[&id(1)]);
    commit(dir.path(), &id(4), &[&id(2), &id(3)]);
    assert_eq!(root(&dir, &id(4), &[]).unwrap(), Some(id(1)));
    commit(dir.path(), &id(8), &[&id(9)]);
    commit(dir.path(), &id(9), &[&id(8)]);
    assert_eq!(root(&dir, &id(8), &[]).unwrap(), None);
}

#[test]
fn a_shallow_boundary_is_not_a_root() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(2), &[&id(1)]);
    commit(dir.path(), &id(3), &[&id(2)]);
    assert_eq!(root(&dir, &id(3), &[&id(2)]).unwrap(), None);
    assert_eq!(root(&dir, &id(2), &[&id(2)]).unwrap(), None);
}

#[test]
fn a_boundary_that_holds_its_parentless_commit_is_still_not_a_root() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(2), &[]);
    assert_eq!(root(&dir, &id(2), &[&id(2)]).unwrap(), None);
}

#[test]
fn a_root_behind_one_side_of_a_shallow_history_is_still_found() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(1), &[]);
    commit(dir.path(), &id(2), &[&id(1)]);
    commit(dir.path(), &id(5), &[&id(4)]);
    commit(dir.path(), &id(6), &[&id(2), &id(5)]);
    assert_eq!(root(&dir, &id(6), &[&id(5)]).unwrap(), Some(id(1)));
}

#[test]
fn a_parent_the_clone_lacks_is_missing() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(2), &[&id(1)]);
    let result = root(&dir, &id(2), &[]);
    assert!(matches!(result, Err(Error::Missing(missing)) if missing == id(1)));
    let result = root(&dir, &id(7), &[]);
    assert!(matches!(result, Err(Error::Missing(missing)) if missing == id(7)));
}

#[test]
fn a_tip_that_is_not_a_commit_is_corrupt() {
    let dir = TempDir::new().unwrap();
    put(dir.path(), &id(1), b"blob 3\0abc");
    assert!(corrupt(root(&dir, &id(1), &[])));
}

#[test]
fn a_commit_with_a_bad_parent_header_is_corrupt() {
    let dir = TempDir::new().unwrap();
    put(dir.path(), &id(1), b"commit 17\0parent nonsense\n\n");
    assert!(corrupt(root(&dir, &id(1), &[])));
}

#[test]
fn the_walk_stops_when_the_deadline_passes() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(1), &[]);
    for n in 2..=20 {
        commit(dir.path(), &id(n), &[&id(n - 1)]);
    }
    let objects = Objects::open(dir.path()).unwrap();
    let asked = Cell::new(0);
    let deadline = || {
        asked.set(asked.get() + 1);
        asked.get() > 5
    };
    let result = lowest_root(&objects, &BTreeSet::new(), id(20), &deadline, MAX_COMMITS);
    assert!(matches!(result, Err(Error::Deadline)));
    assert_eq!(asked.get(), 6);
}

#[test]
fn a_walk_past_the_commit_bound_is_too_large() {
    let dir = TempDir::new().unwrap();
    commit(dir.path(), &id(1), &[]);
    for n in 2..=10 {
        commit(dir.path(), &id(n), &[&id(n - 1)]);
    }
    let objects = Objects::open(dir.path()).unwrap();
    let shallow = BTreeSet::new();
    let result = lowest_root(&objects, &shallow, id(10), &|| false, 5);
    assert!(matches!(result, Err(Error::TooLarge(_))));
    let result = lowest_root(&objects, &shallow, id(10), &|| false, 10);
    assert_eq!(result.unwrap(), Some(id(1)));
}

fn repo(dir: &TempDir) -> Repo {
    Repo {
        head_dir: dir.path().to_path_buf(),
        common_dir: dir.path().to_path_buf(),
    }
}

#[test]
fn a_clone_that_is_not_shallow_lists_no_boundary() {
    let dir = TempDir::new().unwrap();
    assert!(read_shallow(&repo(&dir), 1000).unwrap().is_empty());
}

#[test]
fn the_shallow_file_lists_the_boundary_commits() {
    let dir = TempDir::new().unwrap();
    let text = format!("{}\n\n{}\r\n", id(1).as_str(), id(2).as_str());
    fs::write(dir.path().join("shallow"), text).unwrap();
    let expected: BTreeSet<CommitId> = [id(1), id(2)].into();
    assert_eq!(read_shallow(&repo(&dir), 1000).unwrap(), expected);
}

#[test]
fn a_shallow_file_that_lists_something_else_is_corrupt() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("shallow"), "not an id\n").unwrap();
    assert!(corrupt(read_shallow(&repo(&dir), 1000)));
}

#[test]
fn a_shallow_file_past_the_limit_is_too_large() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("shallow"), format!("{}\n", id(1).as_str())).unwrap();
    assert!(matches!(
        read_shallow(&repo(&dir), 10),
        Err(Error::TooLarge(_))
    ));
}

#[test]
fn a_shallow_file_in_a_linked_worktree_is_the_repositorys() {
    let dir = TempDir::new().unwrap();
    let common = dir.path().join("common");
    fs::create_dir_all(&common).unwrap();
    fs::write(common.join("shallow"), format!("{}\n", id(3).as_str())).unwrap();
    let repo = Repo {
        head_dir: dir.path().join("linked"),
        common_dir: common,
    };
    assert_eq!(read_shallow(&repo, 1000).unwrap(), [id(3)].into());
}
