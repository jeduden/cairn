use std::fs;

use testkit::TempDir;

use super::git_dir;
use crate::Error;

#[test]
fn the_git_directory_of_a_repository_is_its_dot_git() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    assert_eq!(git_dir(dir.path()).unwrap(), Some(dir.path().join(".git")));
}

#[test]
fn a_directory_inside_a_repository_finds_the_one_above() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let deep = dir.path().join("a/b/c");
    fs::create_dir_all(&deep).unwrap();
    assert_eq!(git_dir(&deep).unwrap(), Some(dir.path().join(".git")));
}

#[test]
fn the_nearest_repository_wins() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let inner = dir.path().join("vendor/lib");
    fs::create_dir_all(inner.join(".git")).unwrap();
    assert_eq!(git_dir(&inner).unwrap(), Some(inner.join(".git")));
}

#[test]
fn a_directory_outside_any_repository_finds_none() {
    let dir = TempDir::new().unwrap();
    let inside = dir.path().join("plain/dir");
    fs::create_dir_all(&inside).unwrap();
    assert_eq!(git_dir(&inside).unwrap(), None);
}

#[test]
fn a_path_that_is_not_there_is_looked_for_above() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let missing = dir.path().join("not/yet/made");
    assert_eq!(git_dir(&missing).unwrap(), Some(dir.path().join(".git")));
}

#[test]
fn a_dot_git_file_names_the_git_directory_relative_to_itself() {
    let dir = TempDir::new().unwrap();
    let real = dir.path().join("main/.git/worktrees/w");
    fs::create_dir_all(&real).unwrap();
    let work = dir.path().join("w");
    fs::create_dir(&work).unwrap();
    fs::write(work.join(".git"), "gitdir: ../main/.git/worktrees/w\n").unwrap();
    assert_eq!(
        git_dir(&work).unwrap(),
        Some(work.join("../main/.git/worktrees/w"))
    );
}

#[test]
fn a_dot_git_file_may_name_an_absolute_path() {
    let dir = TempDir::new().unwrap();
    let real = dir.path().join("modules/sub");
    fs::create_dir_all(&real).unwrap();
    let work = dir.path().join("sub");
    fs::create_dir(&work).unwrap();
    fs::write(work.join(".git"), format!("gitdir: {}\r\n", real.display())).unwrap();
    assert_eq!(git_dir(&work).unwrap(), Some(real));
}

#[test]
fn a_dot_git_file_that_names_no_directory_is_corrupt() {
    let dir = TempDir::new().unwrap();
    for text in [
        "",
        "not a gitdir line\n",
        "gitdir:\n",
        "gitdir: missing/dir\n",
        "gitdir: plain\n",
    ] {
        let work = dir.path().join("work");
        fs::create_dir_all(&work).unwrap();
        fs::write(work.join("plain"), "a file").unwrap();
        fs::write(work.join(".git"), text).unwrap();
        let result = git_dir(&work);
        assert!(matches!(result, Err(Error::Corrupt(_))), "{text:?}");
        fs::remove_dir_all(&work).unwrap();
    }
}

#[test]
fn a_dot_git_file_too_large_to_be_one_is_refused() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".git"), vec![b'x'; 1 << 20]).unwrap();
    assert!(matches!(git_dir(dir.path()), Err(Error::TooLarge(_))));
}

#[cfg(unix)]
#[test]
fn a_dot_git_that_is_a_symbolic_link_is_refused() {
    let dir = TempDir::new().unwrap();
    let outside = dir.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let work = dir.path().join("work");
    fs::create_dir(&work).unwrap();
    std::os::unix::fs::symlink(&outside, work.join(".git")).unwrap();
    assert!(matches!(git_dir(&work), Err(Error::Unsupported(_))));
}

#[cfg(unix)]
#[test]
fn a_gitdir_that_is_a_symbolic_link_is_refused() {
    let dir = TempDir::new().unwrap();
    let outside = dir.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let link = dir.path().join("link");
    std::os::unix::fs::symlink(&outside, &link).unwrap();
    let work = dir.path().join("work");
    fs::create_dir(&work).unwrap();
    fs::write(work.join(".git"), "gitdir: ../link\n").unwrap();
    assert!(matches!(git_dir(&work), Err(Error::Unsupported(_))));
}

#[test]
fn a_path_through_a_file_is_looked_for_above_the_file() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join("file"), "x").unwrap();
    let through = dir.path().join("file/below");
    assert_eq!(git_dir(&through).unwrap(), Some(dir.path().join(".git")));
}

#[test]
fn a_gitdir_through_a_file_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("plain"), "a file").unwrap();
    fs::write(dir.path().join(".git"), "gitdir: plain/below\n").unwrap();
    assert!(matches!(git_dir(dir.path()), Err(Error::Io { .. })));
}

#[cfg(unix)]
#[test]
fn a_path_through_a_symbolic_link_loop_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    let again = dir.path().join("again");
    std::os::unix::fs::symlink(&again, &again).unwrap();
    assert!(matches!(
        git_dir(&again.join("below")),
        Err(Error::Io { .. })
    ));
}
