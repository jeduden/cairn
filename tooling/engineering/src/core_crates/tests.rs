use super::*;
use testkit::TempDir;

/// A clippy.toml carrying every ban.
fn full() -> String {
    let mut body = String::from("disallowed-types = [\n");
    for b in REACH_BANS.iter().chain(&DETERMINISM_BANS) {
        body.push_str(&format!("  {{ path = \"{b}\", reason = \"r\" }},\n"));
    }
    body.push_str("]\n");
    body
}

/// A checkout with one crate, `core`, holding `clippy` (none when
/// `None`) and the source files `src`.
fn checkout(clippy: Option<&str>, src: &[(&str, &str)]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::write(root.join("clippy.toml"), "allow-unwrap-in-tests = true\n").unwrap();
    fs::create_dir_all(root.join("crates/core/src")).unwrap();
    fs::create_dir_all(root.join("crates/notes")).unwrap();
    fs::write(root.join("crates/core/Cargo.toml"), "[package]\n").unwrap();
    if let Some(body) = clippy {
        fs::write(root.join("crates/core/clippy.toml"), body).unwrap();
    }
    for (file, body) in src {
        let path = root.join("crates/core/src").join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
    let c = Checkout::new(root);
    (dir, c)
}

#[test]
fn banned_reads_every_path() {
    let got = banned("x = [\n { path = \"a::B\", reason = \"r\" },\n{path=\"c::d\"}, { path = 3 }\n]\n# path is\n");

    assert_eq!(got, BTreeSet::from(["a::B".to_owned(), "c::d".to_owned()]));
}

#[test]
fn missing_bans_names_each_absent_one() {
    assert!(missing_bans(&full()).is_empty());
    let without = full().replace("std::time::Instant::now", "x");

    assert_eq!(missing_bans(&without), ["std::time::Instant::now"]);
    assert_eq!(missing_bans("").len(), REACH_BANS.len() + DETERMINISM_BANS.len());
}

#[test]
fn mutable_globals_finds_interior_mutability_and_thread_locals() {
    let body = "\
static NAME: &str = \"cairn\";
static CONFIG: std::sync::LazyLock<Vec<u8>> = std::sync::LazyLock::new(Vec::new);
pub static COUNT: std::sync::atomic::AtomicU64 = AtomicU64::new(0);
  pub(crate) static CACHE: Mutex<Vec<u8>>
      = Mutex::new(Vec::new());
static mut RAW: u8 = 0;
thread_local! { static T: RefCell<u8> = RefCell::new(0); }
let staticky = 1;
";

    let got = mutable_globals("src/lib.rs", body);

    assert_eq!(
        got,
        [
            "src/lib.rs:3: mutable global state: pub static COUNT: std::sync::atomic::AtomicU64 = AtomicU64::new(0);",
            "src/lib.rs:4: mutable global state: pub(crate) static CACHE: Mutex<Vec<u8>>",
            "src/lib.rs:6: mutable global state: static mut RAW: u8 = 0;",
            "src/lib.rs:7: mutable global state: thread_local! { static T: RefCell<u8> = RefCell::new(0); }",
        ]
    );
}

#[test]
fn crates_keep_their_bans_reads_each_crate_or_the_workspace() {
    let (_d, c) = checkout(Some(&full()), &[]);
    assert!(crates_keep_their_bans(&c).is_empty());

    let (_d, c) = checkout(Some(&full().replace("std::net::UdpSocket", "x")), &[]);
    assert_eq!(
        crates_keep_their_bans(&c),
        ["crate core: crates/core/clippy.toml does not ban std::net::UdpSocket"]
    );

    let (_d, c) = checkout(None, &[]);
    let got = crates_keep_their_bans(&c);
    assert_eq!(got.len(), REACH_BANS.len() + DETERMINISM_BANS.len());
    assert!(got[0].starts_with("crate core: clippy.toml does not ban"), "{got:?}");
}

#[test]
fn crates_keep_their_bans_names_what_it_cannot_read() {
    let (d, c) = checkout(None, &[]);
    fs::remove_file(d.path().join("clippy.toml")).unwrap();
    assert!(crates_keep_their_bans(&c)[0].starts_with("read clippy.toml"));

    let empty = TempDir::new().unwrap();
    let got = crates_keep_their_bans(&Checkout::new(empty.path()));
    assert!(got[0].starts_with("list crates"), "{got:?}");
    assert!(no_mutable_globals(&Checkout::new(empty.path()))[0].starts_with("list crates"));
}

#[test]
fn no_mutable_globals_reads_shipped_code_only() {
    let (_d, c) = checkout(
        None,
        &[
            ("lib.rs", "static OK: u8 = 1;\n"),
            ("store/log.rs", "static SEQ: AtomicU64 = AtomicU64::new(0);\n"),
            ("store/tests.rs", "static T: Mutex<u8> = Mutex::new(0);\n"),
            ("notes.md", "static X: Mutex<u8>\n"),
        ],
    );

    assert_eq!(
        no_mutable_globals(&c),
        ["crates/core/src/store/log.rs:1: mutable global state: static SEQ: AtomicU64 = AtomicU64::new(0);"]
    );
}

#[test]
fn no_mutable_globals_names_what_it_cannot_read() {
    let (d, c) = checkout(None, &[("lib.rs", "")]);
    fs::write(d.path().join("crates/core/src/bad.rs"), [0xff, 0xfe]).unwrap();
    assert!(no_mutable_globals(&c)[0].starts_with("read crates/core/src/bad.rs"));

    fs::remove_dir_all(d.path().join("crates/core/src")).unwrap();
    assert!(no_mutable_globals(&c)[0].starts_with("list "), "{:?}", no_mutable_globals(&c));
}
