use std::cell::RefCell;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rudo_gc::Gc;
use rudo_gc::Trace;

use crate::async_runtime::cancellation::Cancellation;
use crate::async_runtime::get_or_init_runtime;
use crate::async_runtime::ui_thread_dispatcher::WriteSignalUiExt;
use crate::effect::{create_effect, Effect};
use crate::signal::{create_signal, ReadSignal, WriteSignal};

thread_local! {
    static CURRENT_CANCELLATION: RefCell<Option<Cancellation>> = const { RefCell::new(None) };
}

/// A resource that fetches data asynchronously based on a source signal.
///
/// The resource automatically re-fetches when:
/// - The source signal changes
/// - `refetch()` is called explicitly
///
/// The effect is stored within the Resource to prevent GC collection.
/// When the Resource is dropped, the effect is automatically cleaned up.
#[derive(Clone)]
#[allow(dead_code)]
pub struct Resource<T: Trace + Clone + 'static, S: Trace + Clone + 'static> {
    state: ReadSignal<Gc<ResourceState<T>>>,
    refetch_counter: WriteSignal<usize>,
    source: ReadSignal<S>,
    effect: Gc<Effect>,
    version: Arc<AtomicU64>,
    cancellation: Cancellation,
}

impl<T: Trace + Clone + 'static, S: Trace + Clone + 'static> Resource<T, S> {
    #[allow(dead_code)]
    pub fn get(&self) -> Gc<ResourceState<T>> {
        self.state.get()
    }

    #[allow(dead_code)]
    pub fn source(&self) -> ReadSignal<S> {
        self.source.clone()
    }

    pub fn refetch(&self) {
        // Cancel the previous task gracefully
        self.cancellation.cancel();

        // Trigger effect to re-run
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
        match self {
            ResourceState::Pending => {}
            ResourceState::Loading => {}
            ResourceState::Ready(t) => t.trace(visitor),
            ResourceState::Error(_) => {}
        }
    }
}

unsafe impl<T: Trace + Clone + 'static, S: Trace + Clone + 'static> Trace for Resource<T, S> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.state.trace(visitor);
        self.refetch_counter.data.trace(visitor);
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
    let (state, set_state) = create_signal(Gc::new(ResourceState::<T>::Pending));

    let (refetch_counter_read, refetch_counter) = create_signal(0usize);

    // Version tracking to prevent stale updates
    let version = Arc::new(AtomicU64::new(0));

    // Cancellation flag for graceful task termination
    let cancellation = Cancellation::new();

    let source_for_effect = source.clone();
    let fetcher_clone = fetcher.clone();
    let set_state_clone = set_state.clone();

    let dispatcher = set_state.ui_dispatcher();
    let version_clone = Arc::clone(&version);
    let cancellation_clone = cancellation.clone();

    let effect = create_effect(move || {
        let _ = refetch_counter_read.get();
        // Note: We don't read state here - the effect should only react to source
        // or refetch_counter changes, not state changes.

        log::debug!("[Resource] Effect triggered, reading source and refetch_counter");

        // Increment version - old tasks will see stale version and skip
        let current_version = version_clone.fetch_add(1, Ordering::SeqCst) + 1;
        log::debug!("[Resource] Effect triggered, version={}", current_version);

        // Cancel previous task gracefully
        cancellation_clone.cancel();

        // Create new cancellation for this task
        let task_cancellation = Cancellation::new();

        log::debug!("[Resource] Effect reading source signal");
        let source_value = source_for_effect.get();

        let fetcher = fetcher_clone.clone();
        let dispatcher = dispatcher.clone();
        let version_for_task = Arc::clone(&version_clone);
        let cancellation_for_task = task_cancellation.clone();

        log::debug!("[Resource] Setting Loading state, version={}", current_version);

        // Set loading state
        set_state_clone.set(Gc::new(ResourceState::Loading));
        println!("[Resource] Set Loading state");

        let rt = get_or_init_runtime();
        rt.spawn(async move {
            println!("[Resource] Async task started, version={}", current_version);

            // Check cancellation before starting
            if cancellation_for_task.is_cancelled() {
                println!("[Resource] Task cancelled before start");
                return;
            }

            let result = fetcher(source_value).await;

            // Check if cancelled after fetch
            if cancellation_for_task.is_cancelled() {
                println!("[Resource] Task cancelled after fetch");
                return;
            }

            // Check if this task is still current (version check)
            if version_for_task.load(Ordering::SeqCst) != current_version {
                println!("[Resource] Task stale (version check 1), dropping result");
                return; // Stale - a newer fetch started
            }

            let new_state = match result {
                Ok(data) => ResourceState::Ready(data),
                Err(err) => ResourceState::Error(err),
            };

            // Double-check version after async work AND cancellation
            if cancellation_for_task.is_cancelled() {
                println!("[Resource] Task cancelled after async");
                return;
            }

            if version_for_task.load(Ordering::SeqCst) != current_version {
                println!("[Resource] Task stale (version check 2), dropping result");
                return; // Stale
            }

            println!("[Resource] Setting final state, version={}", current_version);
            // Use dispatcher to safely update signal from async context
            dispatcher.set(Gc::new(new_state)).await;
            println!("[Resource] State updated successfully, version={}", current_version);
        });

        // Store cancellation for cleanup
        CURRENT_CANCELLATION.with(|c| {
            *c.borrow_mut() = Some(task_cancellation);
        });
    });

    // Register effect with current component for proper lifecycle management
    // This ensures the effect is cleaned up when the component unmounts
    if let Some(component) = crate::runtime::current_owner() {
        component.add_effect(effect.clone());
    }

    // Register cleanup to cancel the spawned task when the effect is cleaned up
    crate::effect::on_cleanup(move || {
        CURRENT_CANCELLATION.with(|c| {
            if let Some(cancel) = c.borrow_mut().take() {
                cancel.cancel();
            }
        });
    });

    Resource { state, refetch_counter, source, effect, version, cancellation }
}
