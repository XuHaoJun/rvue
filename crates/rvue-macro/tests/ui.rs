//! Compile-fail tests for view! macro
//!
//! These tests verify that the macro correctly rejects invalid syntax
//! and produces helpful error messages.

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/unknown_widget.rs");
    t.compile_fail("tests/ui/malformed_rsx.rs");
    t.compile_fail("tests/ui/invalid_attribute.rs");
}
