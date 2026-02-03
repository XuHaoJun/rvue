// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Macros for snapshot testing.

/// Assert that the current render matches an existing snapshot.
///
/// This macro renders the current state of the test harness and compares it
/// against a snapshot file with the given name.
///
/// # Usage
///
/// ```rust,ignore
/// use rvue_testing::{TestHarness, TestWidgetBuilder, assert_snapshot};
///
/// #[test]
/// fn test_render() {
///     let widget = TestWidgetBuilder::new().build();
///     let mut harness = TestHarness::create(widget);
///
///     // Check against tests/snapshots/test_render.png
///     assert_snapshot!(harness, "test_render");
/// }
/// ```
///
/// # Snapshot Location
///
/// Snapshots are stored in `tests/snapshots/` relative to the crate root.
///
/// # Updating Snapshots
///
/// To update (bless) snapshots, run tests with the `RVUE_TEST_BLESS` environment variable:
///
/// ```bash
/// RVUE_TEST_BLESS=1 cargo test -p rvue-testing
/// ```
#[macro_export]
macro_rules! assert_snapshot {
    ($harness:expr, $name:expr) => {
        $harness.assert_snapshot($name)
    };
}

/// Assert that the current render matches an existing snapshot, using the test function name.
///
/// This is a convenience macro that uses the test function name as the snapshot name.
#[macro_export]
macro_rules! assert_render_snapshot {
    ($harness:expr) => {
        $harness.assert_snapshot(module_path!())
    };
    ($harness:expr, $name:expr) => {
        $harness.assert_snapshot($name)
    };
}
