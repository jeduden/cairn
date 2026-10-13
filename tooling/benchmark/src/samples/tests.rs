use super::*;

#[test]
fn parse_reads_each_benchmark_s_samples_in_order() {
    let text = "# benchmark nanoseconds\ncore_start_up 30\n\nother 5\ncore_start_up 10\n";

    assert_eq!(
        parse(text),
        Ok(Samples::from([
            ("core_start_up".to_owned(), vec![30, 10]),
            ("other".to_owned(), vec![5]),
        ]))
    );
}

#[test]
fn parse_of_nothing_is_no_samples() {
    assert_eq!(parse(""), Ok(Samples::new()));
}

#[test]
fn parse_names_the_line_it_cannot_read() {
    for bad in [
        "core_start_up",
        "core_start_up ten",
        "core_start_up -1",
        "core_start_up  10",
        "Core 10",
        " 10",
    ] {
        let text = format!("ok 1\n{bad}\n");
        assert_eq!(
            parse(&text),
            Err(format!(
                "line 2: {bad:?} is not a benchmark name and a sample in nanoseconds"
            )),
            "{bad:?}"
        );
    }
}

#[test]
fn render_writes_one_line_per_sample_and_parses_back() {
    let text = render("core_start_up", &[3, 1, 2]);

    assert_eq!(text, "core_start_up 3\ncore_start_up 1\ncore_start_up 2\n");
    assert_eq!(
        parse(&format!("{HEADER}\n{text}")),
        Ok(Samples::from([("core_start_up".to_owned(), vec![3, 1, 2])]))
    );
}

#[test]
fn valid_name_takes_lowercase_letters_digits_and_underscores() {
    assert!(valid_name("core_start_up_2"));
    assert!(!valid_name(""));
    assert!(!valid_name("core start"));
    assert!(!valid_name("Core"));
    assert!(!valid_name("core-start"));
}
