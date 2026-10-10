use super::*;

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| (*s).to_owned()).collect()
}

fn tags(t: &[&str]) -> Vec<String> {
    args(t)
}

#[test]
fn parse_args_reads_tags_filters_and_ignores_libtest_flags() {
    let got = parse_args(&args(&[
        "--tags",
        "@ENG-24",
        "--nocapture",
        "--test-threads=2",
        "--color",
        "never",
        "title",
    ]))
    .unwrap();

    let expr = |e: &str| format!("{:?}", Some(e.parse::<TagOperation>().unwrap()));
    assert_eq!(format!("{:?}", got.tags), expr("@ENG-24"));
    assert_eq!(got.filters, ["title"]);
    let short = parse_args(&args(&["-t=@a"])).unwrap();
    assert_eq!(format!("{:?}", short.tags), expr("@a"));
    let none = parse_args(&[]).unwrap();
    assert!(none.tags.is_none() && none.filters.is_empty());
}

#[test]
fn parse_args_refuses_what_it_does_not_know() {
    assert_eq!(
        parse_args(&args(&["--bogus"])).unwrap_err(),
        r#"unknown flag "--bogus""#
    );
    assert_eq!(
        parse_args(&args(&["--tags"])).unwrap_err(),
        "--tags needs a value"
    );
    assert_eq!(
        parse_args(&args(&["--test-threads"])).unwrap_err(),
        "--test-threads needs a value"
    );
    assert!(
        parse_args(&args(&["--tags", "@a and"]))
            .unwrap_err()
            .starts_with(r#"--tags "@a and": "#)
    );
}

#[test]
fn selects_skips_pending_and_applies_tags_and_filters() {
    let all = Options::default();
    let eng24 = parse_args(&args(&["--tags", "@ENG-24"])).unwrap();
    let named = parse_args(&args(&["disclosure"])).unwrap();
    let by_tag = parse_args(&args(&["@ENG-24"])).unwrap();
    let bound = tags(&["ENG-24", "P0"]);
    let pending = tags(&["ENG-25", "P1", "pending"]);
    let title = "SECURITY.md names a private channel and a 90-day disclosure policy";

    assert!(all.selects(title, bound.iter()));
    assert!(!all.selects("x", pending.iter()));
    assert!(eng24.selects(title, bound.iter()));
    assert!(!eng24.selects("x", tags(&["ENG-01"]).iter()));
    assert!(!eng24.selects("x", tags(&["ENG-24", "pending"]).iter()));
    assert!(named.selects(title, bound.iter()));
    assert!(!named.selects("another", bound.iter()));
    assert!(by_tag.selects("another", bound.iter()));
}

/// A future that is pending until polled `left` more times, waking
/// itself each time: by reference on this thread, or by value from
/// another thread.
struct Countdown {
    left: u8,
}

impl Future for Countdown {
    type Output = &'static str;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.left {
            0 => Poll::Ready("done"),
            n => {
                self.left -= 1;
                if n % 2 == 0 {
                    cx.waker().wake_by_ref();
                } else {
                    let waker = cx.waker().clone();
                    thread::spawn(move || waker.wake());
                }
                Poll::Pending
            }
        }
    }
}

#[test]
fn block_on_drives_a_future_through_its_waits() {
    assert_eq!(block_on(Countdown { left: 3 }), "done");
    assert_eq!(block_on(async { 7 }), 7);
}
