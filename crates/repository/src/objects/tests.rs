use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::{MAX_CHAIN, MAX_PACKS, Objects};
use crate::fixture::pack::{PackBuilder, delta, delta_copy, delta_insert};
use crate::fixture::{Prng, zlib_stored};
use crate::object::{Kind, Object};
use crate::{CommitId, Error};

fn raw(n: u8) -> [u8; 20] {
    [n; 20]
}

fn id(n: u8) -> CommitId {
    CommitId::from_bytes(&raw(n))
}

/// Lays `pack` and its index down as the pack `name` of `objects`.
fn put_pack(objects: &Path, name: &str, pack: &[u8], index: &[u8]) {
    let dir = objects.join("pack");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("pack-{name}.pack")), pack).unwrap();
    fs::write(dir.join(format!("pack-{name}.idx")), index).unwrap();
}

fn put_loose(objects: &Path, id: &CommitId, kind: &str, body: &[u8]) {
    let (fan, rest) = id.as_str().split_at(2);
    fs::create_dir_all(objects.join(fan)).unwrap();
    let header = format!("{kind} {}\0", body.len());
    let content = zlib_stored(&[header.as_bytes(), body].concat());
    fs::write(objects.join(fan).join(rest), content).unwrap();
}

fn object(kind: Kind, data: &[u8]) -> Option<Object> {
    Some(Object {
        kind,
        data: data.to_vec(),
    })
}

fn read(objects: &Path, id: &CommitId) -> Result<Option<Object>, Error> {
    Objects::open(objects)?.read(id)
}

#[test]
fn a_loose_object_is_read() {
    let dir = TempDir::new().unwrap();
    put_loose(dir.path(), &id(1), "commit", b"tree x\n");
    let objects = Objects::open(dir.path()).unwrap();
    assert_eq!(
        objects.read(&id(1)).unwrap(),
        object(Kind::Commit, b"tree x\n")
    );
}

#[test]
fn a_packed_object_is_read() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    pack.whole(raw(1), 1, b"first");
    pack.whole(raw(2), 3, b"second");
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    let objects = Objects::open(dir.path()).unwrap();
    assert_eq!(
        objects.read(&id(1)).unwrap(),
        object(Kind::Commit, b"first")
    );
    assert_eq!(objects.read(&id(2)).unwrap(), object(Kind::Blob, b"second"));
}

#[test]
fn an_object_the_clone_lacks_is_none() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    pack.whole(raw(1), 1, b"first");
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    let objects = Objects::open(dir.path()).unwrap();
    assert_eq!(objects.read(&id(9)).unwrap(), None);
}

#[test]
fn a_clone_with_no_object_directory_holds_nothing() {
    let dir = TempDir::new().unwrap();
    let objects = Objects::open(&dir.path().join("objects")).unwrap();
    assert_eq!(objects.read(&id(1)).unwrap(), None);
}

#[test]
fn offset_deltas_are_applied_down_the_chain() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    let base = pack.whole(raw(1), 1, b"hello world");
    let ops = [delta_copy(0, 6), delta_insert(b"brave "), delta_copy(6, 5)];
    let first = pack.offset_delta(raw(2), base, &delta(11, 17, &ops));
    let ops = [delta_copy(0, 17), delta_insert(b"!")];
    pack.offset_delta(raw(3), first, &delta(17, 18, &ops));
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    let objects = Objects::open(dir.path()).unwrap();
    assert_eq!(
        objects.read(&id(2)).unwrap(),
        object(Kind::Commit, b"hello brave world")
    );
    assert_eq!(
        objects.read(&id(3)).unwrap(),
        object(Kind::Commit, b"hello brave world!")
    );
}

#[test]
fn a_reference_delta_finds_its_base_wherever_it_is() {
    let dir = TempDir::new().unwrap();
    let mut base_pack = PackBuilder::new();
    base_pack.whole(raw(1), 2, b"packed base");
    let (bytes, index) = base_pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    put_loose(dir.path(), &id(5), "blob", b"loose base");

    let mut pack = PackBuilder::new();
    pack.whole(raw(6), 1, b"same pack");
    pack.reference_delta(raw(2), raw(1), &delta(11, 3, &[delta_copy(0, 3)]));
    pack.reference_delta(raw(3), raw(5), &delta(10, 4, &[delta_copy(0, 4)]));
    pack.reference_delta(raw(4), raw(6), &delta(9, 4, &[delta_copy(5, 4)]));
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "b", &bytes, &index);

    let objects = Objects::open(dir.path()).unwrap();
    assert_eq!(objects.read(&id(2)).unwrap(), object(Kind::Tree, b"pac"));
    assert_eq!(objects.read(&id(3)).unwrap(), object(Kind::Blob, b"loos"));
    assert_eq!(objects.read(&id(4)).unwrap(), object(Kind::Commit, b"pack"));
}

#[test]
fn a_reference_delta_whose_base_is_absent_names_it() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    pack.reference_delta(raw(2), raw(9), &delta(1, 1, &[delta_copy(0, 1)]));
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    let result = read(dir.path(), &id(2));
    assert!(matches!(result, Err(Error::Missing(missing)) if missing == id(9)));
}

/// A pack holding a chain of `length` deltas on an empty base, each
/// adding one byte; `chain_id(n)` is the id of the nth.
fn chain(dir: &Path, length: usize) {
    let mut pack = PackBuilder::new();
    let mut at = pack.whole(raw(0), 3, b"");
    for n in 1..=length {
        let ops = if n == 1 {
            vec![delta_insert(b"x")]
        } else {
            vec![delta_copy(0, n - 1), delta_insert(b"x")]
        };
        let mut id = [0; 20];
        id[..8].copy_from_slice(&(n as u64).to_be_bytes());
        at = pack.offset_delta(id, at, &delta(n - 1, n, &ops));
    }
    let (bytes, index) = pack.finish();
    put_pack(dir, "chain", &bytes, &index);
}

fn chain_id(n: usize) -> CommitId {
    let mut id = [0; 20];
    id[..8].copy_from_slice(&(n as u64).to_be_bytes());
    CommitId::from_bytes(&id)
}

#[test]
fn a_long_chain_is_followed_to_its_limit() {
    let dir = TempDir::new().unwrap();
    chain(dir.path(), MAX_CHAIN + 10);
    let objects = Objects::open(dir.path()).unwrap();
    let top = objects.read(&chain_id(MAX_CHAIN - 1)).unwrap().unwrap();
    assert_eq!(top.data, vec![b'x'; MAX_CHAIN - 1]);
    let result = objects.read(&chain_id(MAX_CHAIN + 5));
    assert!(matches!(result, Err(Error::TooLarge(_))));
}

#[test]
fn a_delta_that_is_its_own_base_is_cut_off() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    pack.reference_delta(raw(2), raw(2), &delta(1, 1, &[delta_copy(0, 1)]));
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    let result = read(dir.path(), &id(2));
    assert!(matches!(result, Err(Error::TooLarge(_))));
}

#[test]
fn a_delta_that_does_not_fit_its_base_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    let base = pack.whole(raw(1), 1, b"abc");
    pack.offset_delta(raw(2), base, &delta(99, 1, &[delta_insert(b"x")]));
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    let result = read(dir.path(), &id(2));
    assert!(matches!(result, Err(Error::Corrupt(_))));
}

#[test]
fn an_offset_delta_that_is_its_own_base_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    let at = pack.whole(raw(1), 1, b"abc");
    pack.offset_delta(raw(2), at, &delta(3, 1, &[delta_insert(b"x")]));
    let (mut bytes, index) = pack.finish();
    // Point the base of the delta at itself: distance zero.
    let delta_at = at as usize + 1 + zlib_stored(b"abc").len();
    bytes[delta_at + 1] = 0;
    put_pack(dir.path(), "a", &bytes, &index);
    assert!(matches!(read(dir.path(), &id(2)), Err(Error::Corrupt(_))));
}

#[test]
fn alternates_make_an_absent_object_unknowable() {
    let dir = TempDir::new().unwrap();
    put_loose(dir.path(), &id(1), "blob", b"here");
    fs::create_dir_all(dir.path().join("info")).unwrap();
    fs::write(dir.path().join("info/alternates"), "/elsewhere/objects\n").unwrap();
    let objects = Objects::open(dir.path()).unwrap();
    assert_eq!(objects.read(&id(1)).unwrap(), object(Kind::Blob, b"here"));
    assert!(matches!(objects.read(&id(2)), Err(Error::Unsupported(_))));
}

#[test]
fn an_alternates_file_that_names_nothing_changes_nothing() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("info")).unwrap();
    fs::write(dir.path().join("info/alternates"), "# none\n\n  \n").unwrap();
    let objects = Objects::open(dir.path()).unwrap();
    assert_eq!(objects.read(&id(2)).unwrap(), None);
}

#[test]
fn files_that_are_not_packs_are_ignored() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    pack.whole(raw(1), 1, b"first");
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    let packs = dir.path().join("pack");
    fs::write(packs.join("pack-a.keep"), "").unwrap();
    fs::write(packs.join("pack-a.rev"), "junk").unwrap();
    fs::write(packs.join("tmp_pack_x"), "junk").unwrap();
    fs::write(packs.join("pack-orphan.idx"), "no pack beside it").unwrap();
    let objects = Objects::open(dir.path()).unwrap();
    assert_eq!(
        objects.read(&id(1)).unwrap(),
        object(Kind::Commit, b"first")
    );
}

#[test]
fn a_loose_object_that_hashes_to_another_id_is_corrupt() {
    let dir = TempDir::new().unwrap();
    put_loose(dir.path(), &id(1), "commit", b"tree x\n");
    assert!(matches!(read(dir.path(), &id(1)), Err(Error::Corrupt(_))));
}

#[test]
fn a_packed_object_that_hashes_to_another_id_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    pack.whole(raw(1), 1, b"first");
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    assert!(matches!(read(dir.path(), &id(1)), Err(Error::Corrupt(_))));
}

#[test]
fn a_delta_result_that_hashes_to_another_id_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    let real = crate::sha1::object_id("commit", b"hello").to_bytes();
    let base = pack.whole(real, 1, b"hello");
    pack.offset_delta(raw(2), base, &delta(5, 2, &[delta_copy(0, 2)]));
    let (bytes, index) = pack.finish();
    put_pack(dir.path(), "a", &bytes, &index);
    assert!(matches!(read(dir.path(), &id(2)), Err(Error::Corrupt(_))));
}

#[test]
fn an_object_under_its_real_id_is_read() {
    let dir = TempDir::new().unwrap();
    let real = crate::sha1::object_id("commit", b"tree x\n");
    put_loose(dir.path(), &real, "commit", b"tree x\n");
    assert_eq!(
        read(dir.path(), &real).unwrap(),
        object(Kind::Commit, b"tree x\n")
    );
}

#[test]
fn a_pack_index_that_is_a_directory_is_refused() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("pack/pack-dir.idx")).unwrap();
    fs::write(dir.path().join("pack/pack-dir.pack"), "").unwrap();
    assert!(matches!(
        Objects::open(dir.path()),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn a_pack_that_is_not_a_pack_is_corrupt() {
    let dir = TempDir::new().unwrap();
    put_pack(dir.path(), "a", b"definitely not a pack", &[0; 8]);
    assert!(matches!(Objects::open(dir.path()), Err(Error::Corrupt(_))));
}

#[test]
fn more_packs_than_the_limit_are_refused() {
    let dir = TempDir::new().unwrap();
    let header = b"PACK\0\0\0\x02\0\0\0\0";
    for n in 0..=MAX_PACKS {
        put_pack(dir.path(), &format!("{n:04}"), header, b"");
    }
    assert!(matches!(Objects::open(dir.path()), Err(Error::TooLarge(_))));
}

#[test]
fn a_pack_directory_that_cannot_be_listed_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("pack"), "a file, not a directory").unwrap();
    assert!(matches!(Objects::open(dir.path()), Err(Error::Io { .. })));
}

#[test]
fn corrupted_packs_and_indexes_never_panic() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    let base = pack.whole(raw(1), 1, b"hello world");
    pack.offset_delta(raw(2), base, &delta(11, 3, &[delta_copy(0, 3)]));
    pack.reference_delta(raw(3), raw(1), &delta(11, 3, &[delta_copy(0, 3)]));
    let (bytes, index) = pack.finish();
    let mut prng = Prng::new(41);
    for round in 0..400 {
        let (pack_bytes, index_bytes) = match round % 3 {
            0 => (prng.corrupt(&bytes), index.clone()),
            1 => (bytes.clone(), prng.corrupt(&index)),
            _ => (prng.corrupt(&bytes), prng.corrupt(&index)),
        };
        put_pack(dir.path(), "a", &pack_bytes, &index_bytes);
        if let Ok(objects) = Objects::open(dir.path()) {
            for n in 1..=4 {
                let _ = objects.read(&id(n));
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn a_pack_that_is_a_link_is_refused() {
    let dir = TempDir::new().unwrap();
    let packs = dir.path().join("pack");
    fs::create_dir_all(&packs).unwrap();
    fs::write(packs.join("pack-a.idx"), "").unwrap();
    let pack = packs.join("pack-a.pack");
    std::os::unix::fs::symlink(&pack, &pack).unwrap();
    assert!(matches!(
        Objects::open(dir.path()),
        Err(Error::Unsupported(_))
    ));
}

#[cfg(unix)]
#[test]
fn an_index_that_is_a_dangling_link_is_refused() {
    let dir = TempDir::new().unwrap();
    let mut pack = PackBuilder::new();
    pack.whole(raw(1), 1, b"first");
    let (bytes, _) = pack.finish();
    let packs = dir.path().join("pack");
    fs::create_dir_all(&packs).unwrap();
    fs::write(packs.join("pack-a.pack"), bytes).unwrap();
    std::os::unix::fs::symlink(packs.join("gone"), packs.join("pack-a.idx")).unwrap();
    assert!(matches!(
        Objects::open(dir.path()),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn a_symbolic_link_object_or_pack_directory_is_refused() {
    let dir = TempDir::new().unwrap();
    let elsewhere = dir.path().join("elsewhere");
    fs::create_dir_all(elsewhere.join("pack")).unwrap();
    let link = dir.path().join("objects");
    std::os::unix::fs::symlink(&elsewhere, &link).unwrap();
    assert!(matches!(Objects::open(&link), Err(Error::Unsupported(_))));
    let real = dir.path().join("real");
    fs::create_dir_all(&real).unwrap();
    std::os::unix::fs::symlink(&elsewhere, real.join("pack")).unwrap();
    assert!(matches!(Objects::open(&real), Err(Error::Unsupported(_))));
}
