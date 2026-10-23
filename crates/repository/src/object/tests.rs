use super::{Kind, MAX_OBJECT};

#[test]
fn the_four_pack_types_name_the_four_kinds() {
    assert_eq!(Kind::from_pack(1), Some(Kind::Commit));
    assert_eq!(Kind::from_pack(2), Some(Kind::Tree));
    assert_eq!(Kind::from_pack(3), Some(Kind::Blob));
    assert_eq!(Kind::from_pack(4), Some(Kind::Tag));
}

#[test]
fn the_other_pack_types_name_no_kind() {
    for code in [0, 5, 6, 7, 8, 255] {
        assert_eq!(Kind::from_pack(code), None, "{code}");
    }
}

#[test]
fn the_four_header_names_name_the_four_kinds() {
    assert_eq!(Kind::from_name(b"commit"), Some(Kind::Commit));
    assert_eq!(Kind::from_name(b"tree"), Some(Kind::Tree));
    assert_eq!(Kind::from_name(b"blob"), Some(Kind::Blob));
    assert_eq!(Kind::from_name(b"tag"), Some(Kind::Tag));
}

#[test]
fn any_other_header_name_names_no_kind() {
    for name in [&b""[..], b"Commit", b"commit ", b"delta"] {
        assert_eq!(Kind::from_name(name), None);
    }
}

#[test]
fn the_object_bound_holds_a_commit_with_room_to_spare() {
    assert!(MAX_OBJECT >= 1 << 20);
}
