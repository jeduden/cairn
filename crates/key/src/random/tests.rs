use super::*;

#[test]
fn os_entropy_fills_the_buffer() {
    let (mut a, mut b) = ([0u8; 32], [0u8; 32]);

    OsEntropy.fill(&mut a).unwrap();
    OsEntropy.fill(&mut b).unwrap();

    assert_ne!(a, [0; 32]);
    assert_ne!(a, b);
}

#[test]
fn counted_entropy_fills_each_buffer_with_its_next_byte() {
    let mut e = Counted::default();
    let (mut a, mut b) = ([0u8; 2], [0u8; 2]);

    e.fill(&mut a).unwrap();
    e.fill(&mut b).unwrap();

    assert_eq!((a, b), ([1, 1], [2, 2]));
}

#[test]
fn draw_returns_fresh_bytes() {
    let mut e = Counted::default();

    assert_eq!(draw::<3>(&mut e).unwrap(), [1, 1, 1]);
}
