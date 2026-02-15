use std::cell::RefCell;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use rudo_gc::tokio::GcTokioExt;
use rudo_gc::Gc;
use rudo_gc::Trace;

use crate::async_runtime::cancellation::Cancellation;
use crate::async_runtime::get_or_init_runtime;
use crate::effect::{create_effect, on_cleanup, Effect};
use crate::signal::{create_signal, ReadSignal, SignalDataExt, WriteSignal};

thread_local! {
    static CURRENT_CANCELLATION: RefCell<Option<Cancellation>> = const { RefCell::new(None) };
}

/// A resource that fetches data asynchronously based on a source signal.
#[derive(Clone)]
pub struct Resource<T: Trace + Clone + 'static, S: Trace + Clone + 'static> {
    state: ReadSignal<Gc<ResourceState<T>>>,
    refetch_counter: WriteSignal<usize>,
    source: ReadSignal<S>,
    effect: Gc<Effect>,
    #[allow(dead_code)]
    version: Arc<AtomicU64>,
    cancellation: Cancellation,
}

impl<T: Trace + Clone + 'static, S: Trace + Clone + 'static> Resource<T, S> {
    pub fn get(&self) -> Gc<ResourceState<T>> {
        self.state.get()
    }

    pub fn source(&self) -> ReadSignal<S> {
        self.source.clone()
    }

    pub fn refetch(&self) {
        self.cancellation.cancel();
        self.refetch_counter.update(|v| *v += 1);
    }
}

#[derive(Clone, Debug)]
pub enum ResourceState<T: Trace + Clone + 'static> {
    Pending,
    Loading,
    Ready(T),
    Error(String),
}

impl<T: Trace + Clone + 'static> ResourceState<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, ResourceState::Loading)
    }
    pub fn is_ready(&self) -> bool {
        matches!(self, ResourceState::Ready(_))
    }
    pub fn is_error(&self) -> bool {
        matches!(self, ResourceState::Error(_))
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            ResourceState::Ready(t) => Some(t),
            _ => None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            ResourceState::Error(s) => Some(s),
            _ => None,
        }
    }
}

unsafe impl<T: Trace + Clone + 'static> Trace for ResourceState<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        if let ResourceState::Ready(t) = self {
            t.trace(visitor);
        }
    }
}

unsafe impl<T: Trace + Clone + 'static, S: Trace + Clone + 'static> Trace for Resource<T, S> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.state.trace(visitor);
        self.refetch_counter.trace(visitor);
        self.source.trace(visitor);
        self.effect.trace(visitor);
    }
}

pub fn create_resource<S, T, Fu, Fetcher>(source: ReadSignal<S>, fetcher: Fetcher) -> Resource<T, S>
where
    S: PartialEq + Clone + Trace + 'static + Send,
    T: Trace + Clone + 'static + Send + Sync,
    Fu: Future<Output = Result<T, String>> + Send + 'static,
    Fetcher: Fn(S) -> Fu + Clone + Send + 'static,
{
    // Resource spawns background tasks before the app event loop starts.
    // Disable automatic GC globally early to avoid cross-thread collection races.
    rudo_gc::set_gc_enabled(false);

    let (state, set_state) = create_signal(Gc::new(ResourceState::<T>::Pending));
    let (refetch_counter_read, refetch_counter) = create_signal(0usize);
    let version = Arc::new(AtomicU64::new(0));
    let cancellation = Cancellation::new();

    let source_for_effect = source.clone();
    let fetcher_clone = fetcher.clone();
    let set_state_clone = set_state.clone();
    let version_clone = Arc::clone(&version);
    let cancellation_clone = cancellation.clone();
    let state_signal_ptr = state.data.as_ptr();
    let refetch_signal_ptr = refetch_counter_read.data.as_ptr();

    let effect = create_effect(move || {
        let _ = refetch_counter_read.get();
        let current_version = version_clone.fetch_add(1, Ordering::SeqCst) + 1;
        cancellation_clone.cancel();
        let task_cancellation = Cancellation::new();
        let source_value = source_for_effect.get();
        let fetcher = fetcher_clone.clone();

        {
            let mut guard = set_state_clone.data.value.borrow_mut();
            *guard = Gc::new(ResourceState::Loading);
        }
        set_state_clone.data.version.fetch_add(1, Ordering::SeqCst);
        set_state_clone.data.notify_subscribers();

        let rt = get_or_init_runtime();
        let signal_gc = set_state_clone.data.clone();
        let signal_handle_for_task = signal_gc.cross_thread_handle();

        let version_for_task = Arc::clone(&version_clone);
        let cancellation_for_task = task_cancellation.clone();

        // Single write path: when the async task completes, dispatch one callback to the UI
        // queue. That callback runs on the main thread when the queue is drained (e.g. advance()).
        // We do not dispatch a second "poll" callback from the effect, to avoid double-write
        // and ordering races that could corrupt the resource state in headless tests.
        let result_arc: Arc<Mutex<Option<Result<T, String>>>> = Arc::new(Mutex::new(None));
        let result_arc_for_task = Arc::clone(&result_arc);

        rt.spawn(async move {
            if cancellation_for_task.is_cancelled() {
                return;
            }
            let result = fetcher(source_value).await;
            if cancellation_for_task.is_cancelled() {
                return;
            }
            if version_for_task.load(Ordering::SeqCst) != current_version {
                return;
            }

            *result_arc_for_task.lock().unwrap() = Some(result);

            let version_for_task = version_for_task.clone();
            let cancellation_for_task = cancellation_for_task.clone();
            let result_arc_for_dispatch = result_arc_for_task.clone();

            use crate::async_runtime::dispatch::UiDispatchQueue;
            UiDispatchQueue::dispatch(move || {
                let mut guard = result_arc_for_dispatch.lock().unwrap();
                if let Some(result) = guard.take() {
                    if version_for_task.load(Ordering::SeqCst) != current_version {
                        return;
                    }
                    if cancellation_for_task.is_cancelled() {
                        return;
                    }
                    let new_state = match result {
                        Ok(data) => ResourceState::Ready(data),
                        Err(err) => ResourceState::Error(err),
                    };
                    let signal_data = signal_handle_for_task.resolve();
                    let final_state = Gc::new(new_state);
                    let _guard = final_state.root_guard();
                    {
                        let mut cell_guard = signal_data.value.borrow_mut();
                        let _ = std::mem::replace(&mut *cell_guard, final_state);
                    }
                    signal_data.version.fetch_add(1, Ordering::SeqCst);
                    signal_data.notify_subscribers();
                }
            });
        });

        CURRENT_CANCELLATION.with(|c| {
            *c.borrow_mut() = Some(task_cancellation);
        });
    });
    log::debug!(
        "create_resource: effect {:?}, state signal {:?}, refetch signal {:?}",
        effect.as_ptr(),
        state_signal_ptr,
        refetch_signal_ptr
    );

    if let Some(component) = crate::runtime::current_owner() {
        component.add_effect(effect.clone());
    } else {
        crate::signal::leak_effect(effect.clone());
    }

    on_cleanup(move || {
        CURRENT_CANCELLATION.with(|c| {
            if let Some(cancel) = c.borrow_mut().take() {
                cancel.cancel();
            }
        });
    });

    Resource { state, refetch_counter, source, effect, version, cancellation }
}
