use super::*;
use testkit::TempDir;

/// A home in `dir` whose node identity is read from `dir/machine-id`.
fn home(dir: &TempDir) -> Home {
    let home = Home::locate(Some(dir.path().join(".cairn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    home
}

#[test]
fn this_os_reads_its_machine_id_or_nothing() {
    let source = IdentitySource::platform();

    if cfg!(target_os = "linux") {
        assert_eq!(
            source,
            IdentitySource::Files(vec![
                PathBuf::from("/etc/machine-id"),
                PathBuf::from("/var/lib/dbus/machine-id"),
            ])
        );
    } else {
        assert_eq!(source, IdentitySource::Unsupported);
    }
    assert_eq!(IdentitySource::Unsupported.read(), None);
}

#[test]
fn the_first_file_holding_a_value_names_the_node() {
    let dir = TempDir::new().unwrap();
    let (a, b) = (dir.path().join("a"), dir.path().join("b"));
    let source = IdentitySource::Files(vec![a.clone(), b.clone()]);
    assert_eq!(source.observe(), None);

    fs::write(&b, "beta\n").unwrap();
    assert_eq!(source.read().as_deref(), Some("beta"));
    fs::write(&a, "  \n").unwrap();
    assert_eq!(source.read().as_deref(), Some("beta"));
    fs::write(&a, "alpha\n").unwrap();
    assert_eq!(source.observe().as_deref(), Some("alpha"));
}

#[test]
fn a_digest_keeps_no_value_and_an_unavailable_one_has_its_own() {
    let a = digest(Some("alpha"));

    assert_ne!(a, digest(Some("beta")));
    assert_ne!(a, digest(None));
    assert_eq!(digest(None), digest(None));
    assert!(!a.to_hex().contains("alpha"));
}

#[test]
fn a_home_that_recorded_no_identity_records_it_and_is_unchanged() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    fs::write(dir.path().join("machine-id"), "alpha").unwrap();

    assert_eq!(changed(&home).unwrap(), None);

    let recorded = fs::read_to_string(home.node_identity()).unwrap();
    assert_eq!(recorded, digest(Some("alpha")).to_hex());
    assert_eq!(changed(&home).unwrap(), None);
}

#[test]
fn another_value_or_none_at_all_is_a_change() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    fs::write(dir.path().join("machine-id"), "alpha").unwrap();
    changed(&home).unwrap();

    fs::write(dir.path().join("machine-id"), "beta").unwrap();
    assert_eq!(changed(&home).unwrap(), Some(digest(Some("beta"))));

    fs::remove_file(dir.path().join("machine-id")).unwrap();
    assert_eq!(changed(&home).unwrap(), Some(digest(None)));

    record(&home, digest(None)).unwrap();
    assert_eq!(changed(&home).unwrap(), None);
}

#[test]
fn an_unsupported_source_is_never_checked() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir).with_identity(IdentitySource::Unsupported);

    assert_eq!(changed(&home).unwrap(), None);
    assert!(!home.node_identity().exists());
}

#[test]
fn a_recorded_digest_that_cannot_be_read_or_written_is_returned() {
    let dir = TempDir::new().unwrap();
    let a = home(&dir);
    fs::create_dir(a.node_identity()).unwrap();

    assert!(matches!(changed(&a), Err(AdminError::Io { .. })));
    assert!(matches!(
        record(&a, digest(None)),
        Err(AdminError::Io { .. })
    ));

    let dir = TempDir::new().unwrap();
    let b = home(&dir);
    fs::create_dir(staging(&b.node_identity())).unwrap();
    fs::write(staging(&b.node_identity()).join("x"), "").unwrap();
    assert!(matches!(
        record(&b, digest(None)),
        Err(AdminError::Io { .. })
    ));
}
