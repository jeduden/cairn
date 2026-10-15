//! The store's SQLite has no extension loading (ADR-2610110553, I4):
//! `.cargo/config.toml` compiles it out, so no code that loads foreign
//! native code links into the core. The reach test allows crates by
//! name; this proves the build flag behind `libsqlite3-sys`.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use store::Store;
use testkit::TempDir;

#[test]
fn sqlite_is_built_without_extension_loading() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.db");
    drop(Store::open(&path).unwrap());
    let conn = rusqlite::Connection::open(&path).unwrap();

    let omitted: i64 = conn
        .query_row(
            "SELECT sqlite_compileoption_used('OMIT_LOAD_EXTENSION')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let call = conn.query_row("SELECT load_extension('x')", [], |r| r.get::<_, i64>(0));

    assert_eq!(
        omitted, 1,
        "SQLite was built with extension loading: OMIT_LOAD_EXTENSION is not set"
    );
    assert!(call.unwrap_err().to_string().contains("no such function"));
}
