//!! Async runtime support for Rvue
//!
//! This module provides async task spawning, UI dispatch, and reactive resource patterns.
//! All items are feature-gated behind the `async` feature.
//!
//! # Overview
//!
//! - [`spawn_task`] - Spawn an async task that runs on the Tokio runtime
//! - [`spawn_task_with_result`] - Spawn a task and call a callback with the result on the UI thread
//! - [`spawn_interval`] - Run a callback at a regular interval
//! - [`spawn_debounced`] - Debounce a callback
//! - [`dispatch_to_ui`] - Dispatch a closure to be executed on the UI thread
//! - [`SignalSender`] - Send signal updates from async contexts
//! - [`TaskRegistry`] - Registry of tasks associated with components (for cleanup)
//!
//! # Thread Safety
//!
//! All async functions require their closures to be `Send` because they may run on
//! any thread in the Tokio runtime pool. Use [`dispatch_to_ui`] to update UI state
//! from async callbacks.
//!
//! # Example
//!
//! ```ignore
//! use rvue::prelude::*;
//! use rvue::async_runtime::{spawn_task, dispatch_to_ui};
//!
//! fn my_async_work() {
//!     spawn_task(async move {
//!         // This runs on the Tokio runtime
//!         let result = some_async_operation().await;
//!
//!         // Dispatch UI updates back to the main thread
//!         dispatch_to_ui(move || {
//!             // Update signals here
//!         });
//!     });
//! }
//! ```

#[cfg(feature = "async")]
pub mod dispatch;

#[cfg(feature = "async")]
pub mod task;

#[cfg(feature = "async")]
pub mod registry;

#[cfg(feature = "async")]
pub mod signal_sender;

#[cfg(feature = "async")]
pub use dispatch::{dispatch_to_ui, UiDispatchQueue};

#[cfg(feature = "async")]
pub use task::{
    spawn_debounced, spawn_interval, spawn_task, spawn_task_with_result, DebouncedTask, TaskHandle,
    TaskId,
};

#[cfg(feature = "async")]
pub use registry::TaskRegistry;

#[cfg(feature = "async")]
pub use signal_sender::SignalSender;
