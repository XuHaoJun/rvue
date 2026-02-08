//! Async runtime utilities for Rvue.
//!
//! This module provides async utilities built on tokio with full GC safety
//! via rudo-gc's AsyncHandleScope.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::time;

use rudo_gc::handles::AsyncHandleScope;
use rudo_gc::Gc;
use rudo_gc::Trace;

use crate::signal::{ReadSignal, SignalDataInner, WriteSignal};

static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

static TASK_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_task_id() -> TaskId {
    TaskId(TASK_COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[derive(Debug, Clone)]
pub struct TaskHandle {
    pub id: TaskId,
    pub abort_handle: tokio::task::AbortHandle,
    pub completed: Arc<AtomicBool>,
}

impl TaskHandle {
    pub fn abort(&self) {
        self.abort_handle.abort();
        self.completed.store(true, Ordering::SeqCst);
    }

    pub fn is_completed(&self) -> bool {
        self.completed.load(Ordering::SeqCst)
    }

    pub fn is_running(&self) -> bool {
        !self.is_completed() && !self.abort_handle.is_finished()
    }

    pub fn id(&self) -> TaskId {
        self.id
    }
}

fn get_or_init_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .thread_name("rvue-async")
            .build()
            .expect("Failed to create tokio runtime")
    })
}

/// Block on a future using the global tokio runtime.
///
/// This is useful for running async code from synchronous contexts,
/// such as within effects that need to wait for async operations.
///
/// # Panics
/// Panics if called outside a tokio runtime context (use `spawn_task` for async contexts).
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    get_or_init_runtime().block_on(future)
}

/// Spawn an async task that completes immediately.
///
/// Returns a handle for managing the task.
pub fn spawn_task<F>(future: F) -> TaskHandle
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let id = next_task_id();
    let join_handle = get_or_init_runtime().spawn(async move {
        let _ = future.await;
    });

    let completed = Arc::new(AtomicBool::new(false));
    let abort_handle = join_handle.abort_handle();
    let handle = TaskHandle { id, abort_handle, completed: completed.clone() };

    get_or_init_runtime().spawn(async move {
        let _ = join_handle.await;
        completed.store(true, Ordering::SeqCst);
    });

    handle
}

/// Handle for stopping a signal watcher.
#[derive(Clone)]
pub struct SignalWatcher {
    stopped: Arc<AtomicBool>,
}

impl SignalWatcher {
    /// Create a new stopped signal watcher.
    pub fn new() -> Self {
        Self { stopped: Arc::new(AtomicBool::new(false)) }
    }

    /// Stop watching (drops the scope, stopping the task).
    pub fn stop(self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
}

impl Default for SignalWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Watch a signal and invoke callback on changes.
///
/// Uses AsyncHandle for safe GC-managed access in async context.
/// The watcher runs on a tokio task and polls the signal at the given period.
///
/// # Arguments
/// - `read_signal`: The signal to watch
/// - `write_signal`: The signal to update (can be the same as read_signal)
/// - `period`: How often to poll the signal
/// - `callback`: Called with current value; return `Some(v)` to update, `None` to just watch
///
/// # Returns
/// A `SignalWatcher` handle for stopping the watcher.
///
/// # Example
/// ```ignore
/// use rvue::prelude::*;
/// use rvue::async_runtime::watch_signal;
/// use std::time::Duration;
///
/// #[component]
/// fn LiveCounter() -> View {
///     let (count, set_count) = create_signal(0i32);
///
///     let watcher = watch_signal(
///         count,
///         set_count,
///         Duration::from_millis(100),
///         |current| {
///             println!("Count: {}", current);
///             None  // Just watch, don't update
///         }
///     );
///
///     on_cleanup(move || watcher.stop());
///
///     view! {
///         <Text value=format!("Count: {}", count.get()) />
///     }
/// }
/// ```
pub async fn watch_signal<T>(
    read_signal: ReadSignal<T>,
    write_signal: WriteSignal<T>,
    period: Duration,
    mut callback: impl FnMut(T) -> Option<T> + Send + Sync + 'static,
) -> SignalWatcher
where
    T: Trace + Clone + Send + Sync + 'static,
{
    let tcb =
        rudo_gc::heap::current_thread_control_block().expect("watch_signal requires GC thread");

    let scope = Arc::new(AsyncHandleScope::new(&tcb));
    let read_handle = scope.handle(&read_signal.data);
    let write_handle = scope.handle(&write_signal.data);

    let stopped = Arc::new(AtomicBool::new(false));
    let stopped_clone = stopped.clone();

    get_or_init_runtime().spawn(async move {
        let mut interval = time::interval(period);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                biased;
                _ = interval.tick() => {
                    // Safe: handle validates scope is alive
                    let current = read_handle.get().inner.get();
                    if let Some(new_value) = callback(current) {
                        write_handle.get().inner.set(new_value);
                    }
                }
            }

            if stopped_clone.load(Ordering::SeqCst) {
                break;
            }
        }
    });

    SignalWatcher { stopped }
}

/// Creates an async-safe sender for cross-thread signal updates.
#[derive(Clone)]
pub struct AsyncSignalSender<T: Trace + Clone + 'static> {
    data: Gc<SignalDataInner<T>>,
}

impl<T: Trace + Clone + 'static> AsyncSignalSender<T> {
    /// Create a new AsyncSignalSender.
    pub fn new(write_signal: WriteSignal<T>) -> Self {
        let data = Gc::clone(&write_signal.data);
        Self { data }
    }

    /// Set the signal value asynchronously.
    pub async fn set(&self, value: T) {
        *self.data.inner.value.write() = value;
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }
}

/// Extension trait for creating async signal senders.
pub trait WriteSignalExt<T: Trace + Clone + 'static> {
    /// Creates an AsyncSignalSender for thread-safe signal updates from async contexts.
    fn sender(&self) -> AsyncSignalSender<T>;
}

impl<T: Trace + Clone + 'static> WriteSignalExt<T> for WriteSignal<T> {
    fn sender(&self) -> AsyncSignalSender<T> {
        AsyncSignalSender::new(self.clone())
    }
}

/// A handle for stopping interval tasks.
#[derive(Clone)]
pub struct IntervalHandle {
    stopped: Arc<AtomicBool>,
}

impl IntervalHandle {
    /// Create a new stopped interval handle.
    pub fn new() -> Self {
        Self { stopped: Arc::new(AtomicBool::new(false)) }
    }

    /// Stop the interval task.
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
}

impl Default for IntervalHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn a recurring async task that runs at a fixed interval.
///
/// # Example
/// ```ignore
/// use rvue::async_runtime::{spawn_interval, dispatch_to_ui};
/// use std::time::Duration;
///
/// let handle = spawn_interval(Duration::from_secs(30), || async {
///     let status = check_server_status().await;
///     dispatch_to_ui(move || {
///         set_status(status);
///     });
/// });
/// ```
pub fn spawn_interval<F, Fut>(period: Duration, mut f: F) -> IntervalHandle
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let stopped = Arc::new(AtomicBool::new(false));
    let stopped_clone = stopped.clone();

    let handle = IntervalHandle { stopped: stopped_clone.clone() };

    get_or_init_runtime().spawn(async move {
        let mut interval = time::interval(period);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            if stopped_clone.load(Ordering::SeqCst) {
                break;
            }

            f();
        }
    });

    handle
}

/// A debounced async operation.
#[derive(Clone)]
pub struct DebouncedTask<T: Send + 'static> {
    sender: mpsc::UnboundedSender<T>,
    stopped: Arc<AtomicBool>,
}

impl<T: Send + 'static> DebouncedTask<T> {
    /// Call the debounced task with a value.
    pub fn call(&self, value: T) {
        let _ = self.sender.send(value);
    }

    /// Cancel the debounced task.
    pub fn cancel(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
}

/// Create a debounced async operation.
///
/// When `call()` is invoked, the operation is delayed by `delay`.
/// If `call()` is invoked again before the delay expires, the
/// previous pending execution is cancelled and the timer resets.
///
/// # Example
/// ```ignore
/// use rvue::async_runtime::{spawn_debounced, dispatch_to_ui};
/// use std::time::Duration;
///
/// let search = spawn_debounced(Duration::from_millis(300), |query: String| async move {
///     let results = search_api(&query).await;
///     dispatch_to_ui(move || {
///         set_suggestions(results);
///     });
/// });
///
/// // In event handler:
/// search.call(input.get());
/// ```
pub fn spawn_debounced<T, F, Fut>(delay: Duration, handler: F) -> DebouncedTask<T>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (sender, mut receiver) = mpsc::unbounded_channel::<T>();
    let stopped = Arc::new(AtomicBool::new(false));
    let stopped_clone = stopped.clone();

    let task = DebouncedTask { sender, stopped: stopped_clone.clone() };

    get_or_init_runtime().spawn(async move {
        let mut pending_value: Option<T> = None;
        let mut timer = Box::pin(time::sleep(delay));

        loop {
            tokio::select! {
                biased;
                _ = &mut timer => {
                    if let Some(value) = pending_value.take() {
                        handler(value).await;
                    }
                }
                value = receiver.recv() => {
                    match value {
                        Some(v) => {
                            pending_value = Some(v);
                            timer = Box::pin(time::sleep(delay));
                        }
                        None => break,
                    }
                }
            }

            if stopped_clone.load(Ordering::SeqCst) {
                break;
            }
        }
    });

    task
}
