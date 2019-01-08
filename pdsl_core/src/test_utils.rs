//! The minimal test framework for the pdsl core libraries.

/// The set-up procedure of the entire crate under test.
fn setup() {
	let _ = env_logger::try_init();
}

/// The tear-down procedure of the entire crate under test.
fn teardown() {}

/// Runs the given test.
///
/// This executes general setup routines before executing
/// the test and general tear-down procedures after executing.
pub(crate) fn run_test<F>(test: F) -> ()
    where F: FnOnce() -> () + std::panic::UnwindSafe
{
    setup();
    let result = std::panic::catch_unwind(|| {
        test()
    });
    teardown();
    assert!(result.is_ok())
}
