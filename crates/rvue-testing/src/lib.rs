// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Headless testing framework for rvue applications.
//!
//! This crate provides tools for writing unit and integration tests for rvue
//! applications without requiring a visible window. It includes:
//!
//! - [`TestHarness`]: A headless environment for testing rvue applications
//! - Event simulation: Mouse, keyboard, scroll, and focus events
//! - Snapshot testing: Render and compare screenshots
//! - Event recording: Track widget lifecycle events
//!
//! # Example
//!
//! ```rust,ignore
//! use rvue_testing::{TestHarness, TestWidgetBuilder};
//!
//! #[test]
//! fn test_button_click() {
//!     let widget = TestWidgetBuilder::new()
//!         .with_tag("my-button")
//!         .build();
//!
//!     let mut harness = TestHarness::create(widget);
//!     let button = harness.get_widget_by_tag("my-button").unwrap();
//!
//!     harness.mouse_click_on(button);
//!
//!     // Verify the button was clicked
//!     let records = harness.take_records(button);
//! }
//! ```

mod event_recorder;
mod harness;
mod snapshot;
mod test_widget;

#[macro_use]
mod macros;

pub use event_recorder::{EventRecorder, PointerEventType, PointerRecord, RecordedEvent};
pub use harness::{PointerButton, TestHarness, TestHarnessParams};
pub use snapshot::{SnapshotError, SnapshotManager, SnapshotOptions};
pub use test_widget::TestWidgetBuilder;
