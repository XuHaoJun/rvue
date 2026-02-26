//! Thread-safe signal update dispatcher using GcHandle.
//!
//! Provides UiThreadDispatcher for safely dispatching signal updates
//! from async contexts to the UI thread.
//!
//! Uses a thread-local registry to avoid cloning GcHandle from non-origin
//! threads (GcHandle::clone must be called on the origin thread).

use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use rudo_gc::handles::GcHandle;
use rudo_gc::Gc;
use rudo_gc::Trace;

use crate::signal::{SignalData, SignalDataExt};

static NEXT_DISPATCHER_ID: AtomicUsize = AtomicUsize::new(0);

type DispatcherId = usize;

trait DispatcherHandler {
    fn apply(&self, value: Box<dyn Any>);
}

struct TypedHandler<T: Trace + Clone + 'static> {
    handle: GcHandle<SignalData<T>>,
}

impl<T: Trace + Clone + 'static> DispatcherHandler for TypedHandler<T> {
    fn apply(&self, value: Box<dyn Any>) {
        let v = *value.downcast::<T>().expect("dispatcher value type mismatch");
        let signal: Gc<SignalData<T>> = self.handle.resolve();
        *signal.value.borrow_mut_simple() = v;
        signal.version.fetch_add(1, Ordering::SeqCst);
        signal.notify_subscribers();
    }
}

thread_local! {
    static DISPATCHER_REGISTRY: std::cell::RefCell<HashMap<DispatcherId, Box<dyn DispatcherHandler>>> =
        std::cell::RefCell::new(HashMap::new());
}

fn register_handler<T: Trace + Clone + 'static>(handle: GcHandle<SignalData<T>>) -> DispatcherId {
    let id = NEXT_DISPATCHER_ID.fetch_add(1, Ordering::Relaxed);
    let handler = Box::new(TypedHandler { handle });
    DISPATCHER_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(id, handler);
    });
    id
}

fn apply_dispatcher_update(id: DispatcherId, value: Box<dyn Any + Send>) {
    DISPATCHER_REGISTRY.with(|reg| {
        if let Some(handler) = reg.borrow().get(&id) {
            handler.apply(value);
        }
    });
}

fn apply_dispatcher_update_sync(id: DispatcherId, value: Box<dyn Any>) {
    DISPATCHER_REGISTRY.with(|reg| {
        if let Some(handler) = reg.borrow().get(&id) {
            handler.apply(value);
        }
    });
}

/// A handle for dispatching signal updates from async contexts to the UI thread.
///
/// Created via `WriteSignal::ui_dispatcher()`. The handle is `Send + Sync`
/// and can be safely sent to any thread. Resolution happens on the UI thread.
#[derive(Clone)]
pub struct UiThreadDispatcher<T: Trace + Clone + 'static> {
    id: DispatcherId,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Trace + Clone + 'static> UiThreadDispatcher<T> {
    /// Create a new dispatcher from a WriteSignal.
    ///
    /// Must be called on the UI thread where the signal was created.
    /// The GcHandle is cloned and registered here (on the origin thread).
    pub fn new(signal: &crate::signal::WriteSignal<T>) -> Self {
        let handle = signal.data.cross_thread_handle();
        let id = register_handler(handle);
        Self { id, _marker: std::marker::PhantomData }
    }

    /// Dispatch a signal update to the UI thread.
    ///
    /// The update is executed via `dispatch_to_ui`, ensuring all effects
    /// run on the UI thread. This future completes when the update is
    /// *queued*, not when the UI thread has applied it.
    ///
    /// Note: T must be Send because the value is sent through the dispatch queue.
    pub async fn set(&self, value: T)
    where
        T: Send,
    {
        let id = self.id;
        super::dispatch::dispatch_to_ui(move || {
            apply_dispatcher_update(id, Box::new(value));
        });
    }

    /// Synchronously dispatch a signal update on the current thread.
    ///
    /// This MUST be called on the main thread where the GC heap exists.
    pub fn set_sync(&self, value: T) {
        apply_dispatcher_update_sync(self.id, Box::new(value));
    }
}

/// Extension trait for WriteSignal.
pub trait WriteSignalUiExt<T: Trace + Clone + 'static> {
    /// Create a UI thread dispatcher for this signal.
    fn ui_dispatcher(&self) -> UiThreadDispatcher<T>;
}

impl<T: Trace + Clone + 'static> WriteSignalUiExt<T> for crate::signal::WriteSignal<T> {
    fn ui_dispatcher(&self) -> UiThreadDispatcher<T> {
        UiThreadDispatcher::new(self)
    }
}
