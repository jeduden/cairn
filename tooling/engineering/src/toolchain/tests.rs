use super::*;

#[test]
fn go_toolchain_pinned_wants_a_toolchain_line() {
    assert!(go_toolchain_pinned("go 1.26.0\ntoolchain go1.26.8\n"));
    assert!(!go_toolchain_pinned("go 1.26.0\n"));
    assert!(!go_toolchain_pinned("toolchain default\n"));
    assert!(!go_toolchain_pinned("toolchain go1.26.8 extra\n"));
}

#[test]
fn rust_toolchain_pinned_wants_an_exact_release() {
    assert!(rust_toolchain_pinned("[toolchain]\nchannel = \"1.99.0\"\n").is_ok());
    for (toml, want) in [
        (
            "[toolchain]\nchannel = \"stable\"\n",
            r#"does not pin an exact release: "stable""#,
        ),
        (
            "[toolchain]\nchannel = \"1.99\"\n",
            r#"does not pin an exact release: "1.99""#,
        ),
        (
            "[toolchain]\nchannel = \"1.x.0\"\n",
            r#"does not pin an exact release: "1.x.0""#,
        ),
        ("[toolchain]\n", "names no channel"),
    ] {
        assert_eq!(
            rust_toolchain_pinned(toml).unwrap_err(),
            format!("rust-toolchain.toml {want}")
        );
    }
}
