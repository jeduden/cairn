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

#[test]
fn a_home_names_its_audit_log_and_its_home_id_binding() {
    let home = Home::locate(Some(OsStr::new("/c")), None).unwrap();

    assert_eq!(home.audit(), Path::new("/c/audit"));
    assert_eq!(home.binding(), Path::new("/c/home.json"));
}

/// A home made by `create`, with a file in it, readable only by this
/// OS user.
fn made_home(dir: &TempDir) -> Home {
    let home = Home::locate(Some(dir.path().join(".cairn").as_os_str()), None).unwrap();
    home.create().unwrap();
    owner_only_file(&home.store());
    home
}

fn owner_only_file(path: &Path) {
    std::fs::write(path, "").unwrap();
    #[cfg(unix)]
    set_mode(path, 0o600);
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
#[test]
fn the_running_os_user_is_the_one_new_files_belong_to() {
    use std::os::unix::fs::MetadataExt;
    let dir = TempDir::new().unwrap();
    let mine = std::fs::metadata(dir.path()).unwrap().uid();

    assert_eq!(running_uid().unwrap(), mine);
}

#[test]
fn a_home_that_does_not_exist_yet_passes_the_check() {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join("none").as_os_str()), None).unwrap();

    home.check().unwrap();
}

#[cfg(unix)]
#[test]
fn an_owner_only_home_of_the_running_os_user_passes() {
    let dir = TempDir::new().unwrap();
    let home = made_home(&dir);
    std::fs::create_dir(home.root().join("payloads")).unwrap();
    set_mode(&home.root().join("payloads"), 0o700);
    owner_only_file(&home.root().join("payloads/p1"));

    home.check().unwrap();
}

#[cfg(unix)]
#[test]
fn a_home_or_a_file_others_can_reach_is_refused_naming_its_mode() {
    let dir = TempDir::new().unwrap();
    let home = made_home(&dir);
    set_mode(home.root(), 0o755);
    let err = home.check().unwrap_err();
    assert!(
        matches!(err, AdminError::LoosePermissions { mode: 0o755, .. }),
        "{err:?}"
    );
    assert_eq!(
        err.to_string(),
        format!(
            "{}: mode 0755 lets other OS users in; the home wants owner-only permissions",
            home.root().display()
        )
    );
    set_mode(home.root(), 0o700);

    set_mode(&home.store(), 0o644);
    let err = home.check().unwrap_err();
    assert!(
        err.to_string()
            .starts_with(&format!("{}: mode 0644", home.store().display()))
    );
    set_mode(&home.store(), 0o600);

    set_mode(&home.root().join("keys"), 0o710);
    let err = home.check().unwrap_err();
    assert!(err.to_string().contains("keys: mode 0710"), "{err}");
}

#[cfg(unix)]
#[test]
fn a_home_or_a_file_of_another_os_user_is_refused_naming_its_owner() {
    use std::os::unix::fs::MetadataExt;
    let dir = TempDir::new().unwrap();
    let home = made_home(&dir);
    let mine = std::fs::metadata(home.root()).unwrap().uid();
    let other = mine + 1;

    let err = home.check_as(other).unwrap_err();

    assert!(
        matches!(err, AdminError::OtherOsUser { uid, running, .. } if uid == mine && running == other),
        "{err:?}"
    );
    assert_eq!(
        err.to_string(),
        format!(
            "{}: belongs to UID {mine}, not the running UID {other}",
            home.root().display()
        )
    );
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_inside_the_home_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = made_home(&dir);
    std::os::unix::fs::symlink(home.store(), home.root().join("link")).unwrap();

    let err = home.check().unwrap_err();

    assert!(matches!(err, AdminError::SymbolicLink { .. }), "{err:?}");
    assert_eq!(
        err.to_string(),
        format!(
            "{}: a symbolic link inside the home",
            home.root().join("link").display()
        )
    );
}

#[cfg(unix)]
#[test]
fn the_home_itself_may_be_reached_through_a_symbolic_link() {
    let dir = TempDir::new().unwrap();
    let home = made_home(&dir);
    let link = dir.path().join("linked");
    std::os::unix::fs::symlink(home.root(), &link).unwrap();
    let linked = Home::locate(Some(link.as_os_str()), None).unwrap();

    linked.check().unwrap();
}

#[cfg(unix)]
#[test]
fn a_home_directory_that_cannot_be_listed_is_an_io_failure() {
    let dir = TempDir::new().unwrap();
    // A home that is a file: its metadata reads, its listing fails.
    let file = dir.path().join("f");
    owner_only_file(&file);
    let home = Home::locate(Some(file.as_os_str()), None).unwrap();

    let err = home.check().unwrap_err();

    assert!(matches!(err, AdminError::Io { .. }), "{err:?}");
}

#[test]
fn a_home_with_no_home_id_binding_opens_whatever_the_environment_supplies() {
    let dir = TempDir::new().unwrap();
    let home = made_home(&dir);

    home.check_home_id(None).unwrap();
    home.check_home_id(Some(OsStr::new("home-a"))).unwrap();
}

#[test]
fn a_bound_home_opens_only_for_its_own_home_id() {
    let dir = TempDir::new().unwrap();
    let home = made_home(&dir);
    std::fs::write(home.binding(), r#"{"id":"home-a"}"#).unwrap();

    home.check_home_id(Some(OsStr::new("home-a"))).unwrap();
    let err = home.check_home_id(Some(OsStr::new("home-b"))).unwrap_err();
    assert!(matches!(err, AdminError::HomeIdMismatch), "{err:?}");
    assert_eq!(
        err.to_string(),
        "the home id CAIRN_HOME_ID supplies does not match the home's"
    );
    let err = home.check_home_id(None).unwrap_err();
    assert!(matches!(err, AdminError::HomeIdMissing), "{err:?}");
    assert_eq!(
        err.to_string(),
        "the home is bound to a home id, and CAIRN_HOME_ID supplies none"
    );
}

#[test]
fn a_home_id_binding_that_is_not_one_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = made_home(&dir);
    for text in [
        "not json",
        "[]",
        r#"{"id":""}"#,
        r#"{"id":7}"#,
        r#"{"id":"a","more":1}"#,
        r#"{}"#,
    ] {
        std::fs::write(home.binding(), text).unwrap();
        let err = home.check_home_id(Some(OsStr::new("a"))).unwrap_err();
        assert!(
            matches!(err, AdminError::HomeIdBinding { .. }),
            "{text}: {err:?}"
        );
        assert_eq!(
            err.to_string(),
            format!(
                "{}: want {{\"id\": \"<home id>\"}}",
                home.binding().display()
            )
        );
    }
    std::fs::remove_file(home.binding()).unwrap();
    std::fs::create_dir(home.binding()).unwrap();
    let err = home.check_home_id(None).unwrap_err();
    assert!(matches!(err, AdminError::Io { .. }), "{err:?}");
}
