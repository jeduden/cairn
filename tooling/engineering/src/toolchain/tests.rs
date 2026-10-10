use super::*;

#[test]
fn go_toolchain_pinned_wants_a_toolchain_line() {
    assert!(go_toolchain_pinned("go 1.26.0\ntoolchain go1.26.8\n"));
    assert!(!go_toolchain_pinned("go 1.26.0\n"));
    assert!(!go_toolchain_pinned("toolchain default\n"));
    assert!(!go_toolchain_pinned("toolchain go1.26.8 extra\n"));
}
