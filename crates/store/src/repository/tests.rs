use super::*;

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn
}

fn map(rows: &[(&str, &str)]) -> BTreeMap<String, String> {
    rows.iter()
        .map(|(w, i)| ((*w).to_owned(), (*i).to_owned()))
        .collect()
}

#[test]
fn a_worktree_keeps_the_identity_it_last_resolved_to() {
    let conn = conn();
    assert_eq!(identity(&conn, "w").unwrap(), None);

    set_identity(&conn, "w", "one").unwrap();
    assert_eq!(identity(&conn, "w").unwrap().as_deref(), Some("one"));
    set_identity(&conn, "w", "two").unwrap();
    assert_eq!(identity(&conn, "w").unwrap().as_deref(), Some("two"));
    assert_eq!(identity(&conn, "x").unwrap(), None);
}

#[test]
fn a_worktree_can_be_forgotten_and_forgetting_one_never_kept_is_nothing() {
    let conn = conn();
    set_identity(&conn, "w", "one").unwrap();

    forget_identity(&conn, "w").unwrap();
    forget_identity(&conn, "x").unwrap();

    assert_eq!(identity(&conn, "w").unwrap(), None);
}

#[test]
fn a_store_that_never_kept_a_worktree_has_no_table_none_resolved_and_no_rows() {
    let conn = Connection::open_in_memory().unwrap();

    assert_eq!(identity(&conn, "w").unwrap(), None);
    assert!(identities(&conn).unwrap().is_empty());
}

#[test]
fn a_store_that_takes_no_write_keeps_no_worktree() {
    let conn = conn();
    conn.execute_batch("PRAGMA query_only = ON").unwrap();

    assert!(set_identity(&conn, "w", "i").is_err());
}

#[test]
fn the_derived_rows_leave_out_the_markers() {
    let conn = conn();
    set_identity(&conn, "b", "two").unwrap();
    set_identity(&conn, "a", "one").unwrap();
    set_identity(&conn, "reported:a", "x").unwrap();
    set_identity(&conn, "looked:1", "1").unwrap();

    assert_eq!(
        identities(&conn).unwrap(),
        map(&[("a", "one"), ("b", "two")])
    );
}

#[test]
fn replacing_the_derived_rows_keeps_the_markers_and_drops_the_stale_rows() {
    let conn = conn();
    set_identity(&conn, "stale", "old").unwrap();
    set_identity(&conn, "a", "old").unwrap();
    set_identity(&conn, "unreadable:a", "unreadable").unwrap();

    replace(&conn, &map(&[("a", "new"), ("b", "two")])).unwrap();

    assert_eq!(
        identities(&conn).unwrap(),
        map(&[("a", "new"), ("b", "two")])
    );
    assert_eq!(
        identity(&conn, "unreadable:a").unwrap().as_deref(),
        Some("unreadable")
    );
}

#[test]
fn replacing_in_a_store_without_the_table_makes_it() {
    let conn = Connection::open_in_memory().unwrap();

    replace(&conn, &map(&[("a", "one")])).unwrap();

    assert_eq!(identities(&conn).unwrap(), map(&[("a", "one")]));
}
