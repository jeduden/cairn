use super::*;

fn ms(values: &[u64]) -> Vec<Duration> {
    values.iter().map(|v| Duration::from_millis(*v)).collect()
}

#[test]
fn the_median_of_an_odd_count_is_its_middle_sample() {
    assert_eq!(twice_median_nanos(&ms(&[9, 1, 5])), Some(10_000_000));
}

#[test]
fn the_median_of_an_even_count_is_the_mean_of_its_middle_two() {
    assert_eq!(twice_median_nanos(&ms(&[4, 1, 2, 30])), Some(6_000_000));
}

#[test]
fn no_samples_have_no_median() {
    assert_eq!(twice_median_nanos(&[]), None);
}

#[test]
fn a_program_at_the_limit_passes() {
    assert_eq!(within_multiple(&ms(&[50]), &ms(&[10]), 5), Ok(()));
}

#[test]
fn a_program_past_the_limit_fails_naming_the_medians() {
    let err = within_multiple(&ms(&[51]), &ms(&[10]), 5).unwrap_err();

    assert!(
        err.contains("51ms") && err.contains("10ms") && err.contains("5 times"),
        "{err}"
    );
}

#[test]
fn one_outlier_of_the_program_does_not_move_its_median() {
    let program = ms(&[40, 41, 900, 42, 43]);

    assert_eq!(within_multiple(&program, &ms(&[10; 5]), 5), Ok(()));
}

#[test]
fn noise_that_slows_both_cancels() {
    let (quiet_program, quiet_reference) = (ms(&[10]), ms(&[2]));
    let (slow_program, slow_reference) = (ms(&[36]), ms(&[28]));

    assert_eq!(within_multiple(&quiet_program, &quiet_reference, 5), Ok(()));
    assert_eq!(within_multiple(&slow_program, &slow_reference, 5), Ok(()));
}

#[test]
fn either_side_without_samples_fails() {
    assert!(within_multiple(&[], &ms(&[1]), 5).is_err());
    assert!(within_multiple(&ms(&[1]), &[], 5).is_err());
}
