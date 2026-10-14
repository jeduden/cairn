use super::*;
use crate::random::Counted;
use crate::signing::DeviceKey;
use testkit::TempDir;

#[test]
fn a_saved_key_loads_back_from_a_file_only_its_owner_reads() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("device.key");
    let key = DeviceKey::mint(&mut Counted::default()).unwrap();

    key.save(&path).unwrap();

    assert_eq!(DeviceKey::load(&path).unwrap().public(), key.public());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}

#[test]
fn a_key_file_is_never_overwritten() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("device.key");
    let mut e = Counted::default();
    DeviceKey::mint(&mut e).unwrap().save(&path).unwrap();

    let err = DeviceKey::mint(&mut e).unwrap().save(&path).unwrap_err();

    assert!(matches!(err, KeyError::File { .. }), "{err}");
    assert!(
        err.to_string()
            .starts_with(&format!("key file {}: ", path.display()))
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn a_missing_or_malformed_key_file_is_refused() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("device.key");
    assert!(matches!(DeviceKey::load(&path), Err(KeyError::File { .. })));

    write_secret(&path, "not hex").unwrap();
    assert!(matches!(
        DeviceKey::load(&path),
        Err(KeyError::Malformed(_))
    ));
}

#[cfg(unix)]
#[test]
fn a_key_file_others_can_read_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("device.key");
    DeviceKey::mint(&mut Counted::default())
        .unwrap()
        .save(&path)
        .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    let err = DeviceKey::load(&path).unwrap_err();

    assert!(
        matches!(err, KeyError::Permissions { mode: 0o644, .. }),
        "{err}"
    );
    assert_eq!(
        err.to_string(),
        format!(
            "key file {} is readable by others (mode 644); want 600",
            path.display()
        )
    );
}

#[test]
fn a_key_file_appears_whole_or_not_at_all_and_a_crash_s_leftover_does_not_block_it() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("device.key");
    // What a crash mid-write leaves: the temporary file, half written.
    fs::write(staging(&path), "half").unwrap();

    write_secret(&path, "whole").unwrap();

    assert_eq!(read_secret(&path).unwrap(), "whole");
    assert!(!staging(&path).exists());
}

#[test]
fn a_key_file_in_a_missing_directory_is_refused() {
    let dir = TempDir::new().unwrap();

    let err = write_secret(&dir.path().join("no/such/device.key"), "x").unwrap_err();

    assert!(matches!(err, KeyError::File { .. }), "{err}");
}

#[test]
fn a_leftover_that_cannot_be_removed_is_refused() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("device.key");
    fs::create_dir(staging(&path)).unwrap();
    fs::write(staging(&path).join("inside"), "").unwrap();

    let err = write_secret(&path, "x").unwrap_err();

    assert!(matches!(err, KeyError::File { .. }), "{err}");
    assert!(!path.exists());
}
