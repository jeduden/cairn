use super::*;
use std::ffi::OsStr;
use testkit::TempDir;

#[test]
fn the_home_is_cairn_home_when_set_else_dot_cairn_in_home() {
    assert_eq!(
        Home::locate(Some(OsStr::new("/c")), Some(OsStr::new("/h")))
            .unwrap()
            .root(),
        Path::new("/c")
    );
    assert_eq!(
        Home::locate(None, Some(OsStr::new("/h"))).unwrap().root(),
        Path::new("/h/.cairn")
    );
    assert_eq!(
        Home::locate(Some(OsStr::new("")), Some(OsStr::new("/h")))
            .unwrap()
            .root(),
        Path::new("/h/.cairn")
    );
}

#[test]
fn no_home_at_all_is_refused() {
    for home in [None, Some(OsStr::new(""))] {
        let err = Home::locate(None, home).unwrap_err();
        assert_eq!(err.to_string(), "no home: set CAIRN_HOME or HOME");
    }
}

#[test]
fn a_home_names_its_files() {
    let home = Home::locate(Some(OsStr::new("/c")), None).unwrap();
    let room = record::id::RoomId::derive(&[1; 32], &[2; 16]);
    let seat = record::id::SeatId::derive(&room, &[1; 32]);

    assert_eq!(home.store(), Path::new("/c/store.db"));
    assert_eq!(home.device_key(), Path::new("/c/keys/device.key"));
    assert_eq!(home.recovery_key(), Path::new("/c/keys/recovery.key"));
    assert_eq!(
        home.seat_key(seat),
        PathBuf::from(format!("/c/keys/seat-{seat}.key"))
    );
}

#[test]
fn create_makes_the_home_and_its_keys_directory_for_its_owner_alone() {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join("a/.cairn").as_os_str()), None).unwrap();

    home.create().unwrap();
    home.create().unwrap();

    assert!(home.root().join("keys").is_dir());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [home.root().to_path_buf(), home.root().join("keys")] {
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o700, "{}", path.display());
        }
    }
}

#[test]
fn create_fails_where_a_file_stands() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("f"), "").unwrap();
    let home = Home::locate(Some(dir.path().join("f").as_os_str()), None).unwrap();

    let err = home.create().unwrap_err();

    assert!(matches!(err, AdminError::Io { .. }), "{err}");
}

#[test]
fn a_first_start_stages_its_genesis_in_one_directory_of_keys() {
    let home = Home::locate(Some(OsStr::new("/c")), None).unwrap();

    assert_eq!(home.genesis(), Path::new("/c/keys/genesis"));
}
