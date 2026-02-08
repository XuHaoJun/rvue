use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::time;

use super::dispatch::dispatch_cross_thread;
use super::registry::REGISTRY;

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

    if let Some(owner) = crate::runtime::current_owner() {
        REGISTRY.with(|r| r.lock().unwrap().register(owner.id, handle.clone()));
    }

    get_or_init_runtime().spawn(async move {
        let _ = join_handle.await;
        completed.store(true, Ordering::SeqCst);
    });

    handle
}

pub fn spawn_task_with_result<F, T, C>(future: F, on_complete: C) -> TaskHandle
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
    C: FnOnce(T) + Send + 'static,
{
    let id = next_task_id();
    let join_handle = get_or_init_runtime().spawn(async move {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| future)) {
            Ok(f) => {
                let result = f.await;
                dispatch_cross_thread(move || {
                    on_complete(result);
                });
            }
            Err(_) => {}
        }
    });

    let completed = Arc::new(AtomicBool::new(false));
    let abort_handle = join_handle.abort_handle();
    let handle = TaskHandle { id, abort_handle, completed: completed.clone() };

    if let Some(owner) = crate::runtime::current_owner() {
        REGISTRY.with(|r| r.lock().unwrap().register(owner.id, handle.clone()));
    }

    get_or_init_runtime().spawn(async move {
        let _ = join_handle.await;
        completed.store(true, Ordering::SeqCst);
    });

    handle
}

/// Spawn a recurring async task that runs at a fixed interval.
///
/// The task starts immediately and repeats every `period`.
///
/// # GC Safety
/// **IMPORTANT**: The closure MUST NOT capture `Gc<T>` objects.
///
/// `Gc<T>` is `!Send + !Sync` and cannot be moved across thread boundaries
/// without explicit handling. The closure is spawned on a tokio worker thread,
/// so any captured `Gc<T>` will cause memory corruption.
///
/// **Correct Usage**:
/// ```ignore
/// let count = create_signal(0i32);
/// let current = *count.get();  // Extract value
/// spawn_interval(Duration::from_secs(1), move || {
///     println!("{}", current);  // Safe: current is owned value
/// });
/// ```
///
/// **For Signal Watching**: Use `spawn_watch_signal()` instead.
///
/// **Example**:
/// ```
/// use std::time::Duration;
/// use rvue::async_runtime::{spawn_interval, dispatch_to_ui};
///
/// let handle = spawn_interval(Duration::from_secs(30), || async {
///     let status = check_server_status().await;
///     dispatch_to_ui(move || {
///         set_status(status);
///     });
/// });
/// ```
pub fn spawn_interval<F, Fut>(period: Duration, mut f: F) -> TaskHandle
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let id = next_task_id();
    let join_handle = get_or_init_runtime().spawn(async move {
        let mut interval = time::interval(period);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            f();
        }
    });

    let completed = Arc::new(AtomicBool::new(false));
    let abort_handle = join_handle.abort_handle();
    let handle = TaskHandle { id, abort_handle, completed: completed.clone() };

    if let Some(owner) = crate::runtime::current_owner() {
        REGISTRY.with(|r| r.lock().unwrap().register(owner.id, handle.clone()));
    }

    get_or_init_runtime().spawn(async move {
        let _ = join_handle.await;
        completed.store(true, Ordering::SeqCst);
    });

    handle
}

pub struct DebouncedTask<T: Send + 'static> {
    sender: mpsc::UnboundedSender<T>,
    handle: TaskHandle,
}

impl<T: Send + 'static> DebouncedTask<T> {
    pub fn call(&self, value: T) {
        let _ = self.sender.send(value);
    }

    pub fn cancel(&self) {
        self.handle.abort();
    }
}

/// Create a debounced async operation.
///
/// When `call()` is invoked, the operation is delayed by `delay`.
/// If `call()` is invoked again before the delay expires, the
/// previous pending execution is cancelled and the timer resets.
///
/// # GC Safety
/// **IMPORTANT**: The handler closure and the value type `T` MUST NOT contain `Gc<T>`.
///
/// `Gc<T>` is `!Send + !Sync`. Values are sent through an mpsc channel to the
/// debounce task, which runs on a tokio worker thread.
///
/// **Correct Usage**:
/// ```ignore
/// let gc_data = Gc::new(MyData { value: 42 });
/// let value = gc_data.value.clone();  // Extract value
///
/// let search = spawn_debounced(Duration::from_millis(300), move |query: String| {
///     let data = value.clone();  // Clone from extracted value
///     async move {
///         let result = search_api(&query, &data).await;
///         dispatch_to_ui(move || { /* update UI */ });
///     }
/// });
/// ```
///
/// **For Signal Watching**: Use `spawn_watch_signal()` instead.
///
/// **Example**:
/// ```
/// use std::time::Duration;
/// use rvue::async_runtime::{spawn_debounced, dispatch_to_ui};
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
    F: Fn(T) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (sender, mut receiver) = mpsc::unbounded_channel::<T>();

    let id = next_task_id();
    let join_handle = get_or_init_runtime().spawn(async move {
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
        }
    });

    let completed = Arc::new(AtomicBool::new(false));
    let abort_handle = join_handle.abort_handle();
    let handle = TaskHandle { id, abort_handle, completed };

    DebouncedTask { sender, handle }
}

#[derive(Clone)]
pub struct SignalWatcher<T: Send + 'static> {
    stopped: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl<T: Send + 'static> SignalWatcher<T> {
    pub fn stop(&self) {
        self.stopped.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Spawn a task that watches a signal and optionally dispatches updates.
///
/// Uses a background thread with std::sync primitives for thread-safe signal access.
///
/// # Example
/// ```ignore
/// use rvue::prelude::*;
/// use rvue::async_runtime::spawn_watch_signal;
///
/// #[component]
/// fn LiveCounter() -> View {
///     let (count, set_count) = create_signal(0i32);
///
///     let watcher = spawn_watch_signal(
///         count,
///         set_count,
///         Duration::from_millis(100),
///         |current| {
///             println!("Count: {}", current);
///             None  // Return Some(value) to update, None to just watch
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
///
/// # Arguments
/// - `read_signal`: The signal to watch
/// - `write_signal`: The signal to update (can be the same as read_signal)
/// - `period`: How often to poll the signal
/// - `callback`: Called with current value; return `Some(v)` to update, `None` to just watch
///
/// # Returns
/// A `SignalWatcher<T>` handle for stopping the watcher.
pub fn spawn_watch_signal<T, F>(
    read_signal: crate::signal::ReadSignal<T>,
    write_signal: crate::signal::WriteSignal<T>,
    period: Duration,
    callback: F,
) -> SignalWatcher<T>
where
    T: rudo_gc::Trace + Clone + Send + Sync + 'static,
    F: FnMut(T) -> Option<T> + Send + Sync + 'static,
{
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    let stopped = Arc::new(AtomicBool::new(false));
    let stopped_clone = stopped.clone();

    let mut callback = callback;
    let initial_value = read_signal.get();

    thread::spawn(move || {
        let mut last_value = initial_value;

        loop {
            if stopped_clone.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(period);

            let current = read_signal.get();
            let result = callback(current);

            if let Some(new_value) = result {
                if new_value != last_value {
                    write_signal.set(new_value.clone());
                    last_value = new_value;
                }
            }
        }
    });

    SignalWatcher { stopped }
}
