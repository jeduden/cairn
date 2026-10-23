use super::*;

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT;")
        .unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn
}

#[test]
fn a_secret_is_kept_when_first_asked_and_never_replaced() {
    let conn = conn();

    let first = secret(&conn, "repository", [1; 32]).unwrap();
    let second = secret(&conn, "repository", [2; 32]).unwrap();
    let other = secret(&conn, "other", [3; 32]).unwrap();

    assert_eq!((first, second, other), ([1; 32], [1; 32], [3; 32]));
}

#[test]
fn a_secret_that_does_not_read_is_corrupt() {
    let conn = conn();
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, 'short')",
        [key("repository")],
    )
    .unwrap();

    assert!(matches!(
        secret(&conn, "repository", [1; 32]),
        Err(StoreError::Corrupt { .. })
    ));
}

#[test]
fn a_directory_keeps_the_identity_it_last_resolved_to() {
    let conn = conn();
    assert_eq!(identity(&conn, "d").unwrap(), None);

    set_identity(&conn, "d", "one").unwrap();
    assert_eq!(identity(&conn, "d").unwrap().as_deref(), Some("one"));
    set_identity(&conn, "d", "two").unwrap();
    assert_eq!(identity(&conn, "d").unwrap().as_deref(), Some("two"));
    assert_eq!(identity(&conn, "e").unwrap(), None);
}

#[test]
fn a_store_that_never_kept_a_directory_has_no_table_and_none_resolved() {
    let conn = Connection::open_in_memory().unwrap();

    assert_eq!(identity(&conn, "d").unwrap(), None);
    let _ = conn.query_row("SELECT 1", [], |r| r.get::<_, i64>(0)).optional();
}
