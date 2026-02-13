//! Thread-safe signal update dispatcher using GcHandle.
//!
//! Provides UiThreadDispatcher for safely dispatching signal updates
//! from async contexts to the UI thread.

use std::sync::atomic::Ordering;

use rudo_gc::handles::GcHandle;
use rudo_gc::Gc;
use rudo_gc::Trace;

use crate::signal::SignalDataInner;

/// A handle for dispatching signal updates from async contexts to the UI thread.
///
/// Created via `WriteSignal::ui_dispatcher()`. The handle is `Send + Sync`
/// and can be safely sent to any thread. Resolution happens on the UI thread.
#[derive(Clone)]
pub struct UiThreadDispatcher<T: Trace + Clone + 'static> {
    handle: GcHandle<SignalDataInner<T>>,
}

impl<T: Trace + Clone + 'static> UiThreadDispatcher<T> {
    /// Create a new dispatcher from a WriteSignal.
    ///
    /// Must be called on the UI thread where the signal was created.
    pub fn new(signal: &crate::signal::WriteSignal<T>) -> Self {
        Self { handle: signal.data.cross_thread_handle() }
    }

    /// Dispatch a signal update to the UI thread.
    ///
    /// The update is executed via `dispatch_to_ui`, ensuring all effects
    /// run on the UI thread.
    ///
    /// Note: T must be Send because the value is sent through the dispatch queue.
    pub async fn set(&self, value: T)
    where
        T: Send,
    {
        let handle = self.handle.clone();
        super::dispatch::dispatch_to_ui(move || {
            log::debug!("[Dispatcher] Starting dispatch");
            let signal: Gc<SignalDataInner<T>> = handle.resolve();
            log::debug!("[Dispatcher] Resolved signal, setting value");
            *signal.inner.value.borrow_mut_gen_only() = value;
            log::debug!("[Dispatcher] Value set, incrementing version");
            signal.inner.version.fetch_add(1, Ordering::SeqCst);
            log::debug!("[Dispatcher] Calling notify_subscribers");
            signal.notify_subscribers();
            log::debug!("[Dispatcher] notify_subscribers completed");
        });
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
