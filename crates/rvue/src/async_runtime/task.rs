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
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
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
