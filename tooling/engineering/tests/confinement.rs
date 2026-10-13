//! ENG-12's core row against the repository itself: every test that
//! starts the core starts it through the end-to-end suite's
//! confinement, so all network is denied and a socket fails the test.

use engineering::{Checkout, confinement};

/// No test of a product crate starts its executable unconfined.
#[test]
fn the_core_s_tests_start_it_confined() {
    let problems = confinement::unconfined_starts(&Checkout::new(&testkit::repo_root()));

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
