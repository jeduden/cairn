use super::*;

#[test]
fn a_temp_dir_exists_until_dropped() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_path_buf();
    fs::write(path.join("a"), "x").unwrap();

    assert!(path.is_dir());
    drop(dir);
    assert!(!path.exists());
}

#[test]
fn two_temp_dirs_never_share_a_path() {
    let (a, b) = (TempDir::new().unwrap(), TempDir::new().unwrap());

    assert_ne!(a.path(), b.path());
}

#[cfg(unix)]
#[test]
fn a_temp_dir_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();

    assert_eq!(
        fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[test]
fn create_unique_skips_a_name_that_exists() {
    let base = TempDir::new().unwrap();
    fs::create_dir(base.path().join("a")).unwrap();

    let got = create_unique(base.path(), &mut ["a".into(), "b".into()].into_iter()).unwrap();

    assert_eq!(got, base.path().join("b"));
}

#[test]
fn create_unique_reports_running_out_of_names() {
    let base = TempDir::new().unwrap();
    fs::create_dir(base.path().join("a")).unwrap();

    let err = create_unique(base.path(), &mut ["a".into()].into_iter()).unwrap_err();

    assert!(err.to_string().contains("no free directory name"), "{err}");
}

#[test]
fn create_unique_passes_other_errors_on() {
    let base = TempDir::new().unwrap();

    let err =
        create_unique(&base.path().join("missing"), &mut ["a".into()].into_iter()).unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}

#[test]
fn repo_root_holds_the_workspace_manifest() {
    assert!(repo_root().join("Cargo.toml").is_file());
}
