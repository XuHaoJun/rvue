//! Async runtime support for Rvue
//!
//! This module provides async task spawning, UI dispatch, and reactive resource patterns.
//! All items are feature-gated behind the `async` feature.
//!
//! # Overview
//!
//! - [`spawn_task`] - Spawn an async task that runs on the Tokio runtime
//! - [`spawn_interval`] - Run a callback at a regular interval
//! - [`spawn_debounced`] - Debounce a callback
//! - [`watch_signal`] - Watch a signal and invoke callback on changes
//! - [`dispatch_to_ui`] - Dispatch a closure to be executed on the UI thread
//! - [`UiThreadDispatcher`] - Send signal updates from async contexts
//! - [`ComponentScope`] - Dynamic component tracking for async operations
//!
//! # GC Safety
//!
//! This module uses rudo-gc's `AsyncHandleScope` and `GcScope` for safe GC-managed
//! async operations. Signals and components can be accessed directly from async
//! contexts with automatic lifetime validation.
//!
//! # Example
//!
//! ```
//! use rvue::prelude::*;
//! use rvue::async_runtime::{spawn_task, dispatch_to_ui, watch_signal};
//! use std::time::Duration;
//!
//! fn my_async_work() {
//!     let (count, set_count) = create_signal(0i32);
//!
//!     // Watch signal with async callback
//!     watch_signal(count, set_count, Duration::from_millis(100), |value| {
//!         println!("Count: {}", value);
//!         None
//!     });
//!
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
//! # async fn some_async_operation() -> i32 { 42 }
//! ```

#[cfg(feature = "async")]
pub mod cancellation;

#[cfg(feature = "async")]
pub mod dispatch;

#[cfg(feature = "async")]
pub mod task;

#[cfg(feature = "async")]
pub mod registry;

#[cfg(feature = "async")]
pub mod component_scope;

#[cfg(feature = "async")]
pub mod resource;

#[cfg(feature = "async")]
pub use dispatch::{dispatch_to_ui, UiDispatchQueue};

#[cfg(feature = "async")]
pub use task::{
    block_on, get_or_init_runtime, spawn_debounced, spawn_interval, spawn_on_runtime, spawn_task,
    watch_signal, DebouncedTask, IntervalHandle, SignalWatcher, TaskHandle, TaskId,
};

#[cfg(feature = "async")]
pub mod ui_thread_dispatcher;

#[cfg(feature = "async")]
pub use ui_thread_dispatcher::{UiThreadDispatcher, WriteSignalUiExt};

#[cfg(feature = "async")]
pub use registry::TaskRegistry;

#[cfg(feature = "async")]
pub use component_scope::ComponentScope;

#[cfg(feature = "async")]
pub use resource::{create_resource, Resource, ResourceState};
