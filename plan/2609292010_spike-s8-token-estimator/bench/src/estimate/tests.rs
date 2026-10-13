use super::*;

#[test]
fn class_sorts_ascii() {
    assert_eq!(class('q'), Class::Letter);
    assert_eq!(class('Z'), Class::Letter);
    assert_eq!(class('7'), Class::Digit);
    assert_eq!(class(' '), Class::Space);
    assert_eq!(class('\n'), Class::Break);
    assert_eq!(class('\t'), Class::Break);
    assert_eq!(class('{'), Class::Punct);
    assert_eq!(class('~'), Class::Punct);
}

#[test]
fn class_sorts_by_script() {
    assert_eq!(class('é'), Class::Latin);
    assert_eq!(class('Ł'), Class::Latin);
    assert_eq!(class('Ж'), Class::Two);
    assert_eq!(class('ש'), Class::Two);
    assert_eq!(class('ह'), Class::Three);
    assert_eq!(class('ก'), Class::Three);
    assert_eq!(class('—'), Class::Symbol);
    assert_eq!(class('┌'), Class::Symbol);
    assert_eq!(class('東'), Class::Cjk);
    assert_eq!(class('の'), Class::Cjk);
    assert_eq!(class('한'), Class::Cjk);
    assert_eq!(class('Ａ'), Class::Cjk);
    assert_eq!(class('🚀'), Class::Four);
    assert_eq!(class('𠀋'), Class::Four);
}

#[test]
fn counts_counts_classes_and_switches() {
    let (n, s) = counts("ab 1\n{é東🚀");
    assert_eq!(n, [2, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1]);
    assert_eq!(s, 0);
    assert_eq!(counts("aB3c").1, 3);
    assert_eq!(counts("ab cd").1, 0);
    assert_eq!(counts(""), ([0; CLASSES], 0));
}

#[test]
fn from_counts_rounds_up_and_adds_the_base() {
    assert_eq!(from_counts(&[0; CLASSES], 0), BASE);
    let mut n = [0; CLASSES];
    n[0] = 1;
    assert_eq!(from_counts(&n, 0), 1 + BASE);
    assert_eq!(from_counts(&n, 1), 2 + BASE);
}

#[test]
fn estimate_is_deterministic_and_monotone() {
    let a = estimate("Never push to main.");
    assert_eq!(a, estimate("Never push to main."));
    assert!(estimate("Never push to main. Never.") >= a);
    assert!(estimate("x") >= 1);
}
