use std::fs;

use testkit::TempDir;

use super::read_limited;
use crate::Error;

#[test]
fn a_file_is_read_whole() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("f");
    fs::write(&path, b"contents").unwrap();
    assert_eq!(read_limited(&path, 100).unwrap().unwrap(), b"contents");
}

#[test]
fn a_file_of_exactly_the_limit_is_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("f");
    fs::write(&path, b"12345").unwrap();
    assert_eq!(read_limited(&path, 5).unwrap().unwrap(), b"12345");
}

#[test]
fn a_missing_file_is_none() {
    let dir = TempDir::new().unwrap();
    assert!(read_limited(&dir.path().join("none"), 5).unwrap().is_none());
}

#[test]
fn a_file_past_the_limit_is_too_large() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("f");
    fs::write(&path, b"123456").unwrap();
    assert!(matches!(read_limited(&path, 5), Err(Error::TooLarge(_))));
}

#[test]
fn a_directory_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    assert!(matches!(read_limited(dir.path(), 5), Err(Error::Io { .. })));
}
