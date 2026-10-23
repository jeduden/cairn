use super::*;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn
}

#[test]
fn a_store_with_no_origins_table_holds_none() {
    let conn = Connection::open_in_memory().unwrap();

    assert_eq!(all(&conn).unwrap(), BTreeMap::new());
}

#[test]
fn origins_are_read_back_by_writer() {
    let conn = conn();
    let kept: BTreeMap<_, _> = [(writer(1), "witnessed"), (writer(2), "peer")]
        .into_iter()
        .map(|(w, o)| (w, o.to_owned()))
        .collect();

    replace(&conn, &kept).unwrap();

    assert_eq!(all(&conn).unwrap(), kept);
}

#[test]
fn replacing_drops_the_origins_not_offered() {
    let conn = conn();
    let first: BTreeMap<_, _> = [(writer(1), "witnessed".to_owned())].into();
    let second: BTreeMap<_, _> = [(writer(2), "ingested".to_owned())].into();

    replace(&conn, &first).unwrap();
    replace(&conn, &second).unwrap();

    assert_eq!(all(&conn).unwrap(), second);
}
