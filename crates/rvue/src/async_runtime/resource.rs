use std::cell::RefCell;
use std::future::Future;

use rudo_gc::handles::HandleScope;
use rudo_gc::heap::current_thread_control_block;
use rudo_gc::Gc;
use rudo_gc::Trace;

use crate::async_runtime::get_or_init_runtime;
use crate::async_runtime::ui_thread_dispatcher::WriteSignalUiExt;
use crate::effect::{create_effect, Effect};
use crate::signal::{create_signal, ReadSignal, WriteSignal};

thread_local! {
    static CURRENT_TASK_HANDLE: RefCell<Option<tokio::task::JoinHandle<()>>> = RefCell::new(None);
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
pub struct Resource<T: Trace + Clone + 'static, S: Trace + Clone + 'static> {
    state: ReadSignal<Gc<ResourceState<T>>>,
    refetch_counter: WriteSignal<usize>,
    source: ReadSignal<S>,
    effect: Gc<Effect>,
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
        // Trace the effect to keep it alive as long as the Resource is alive
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

    let source_for_effect = source.clone();
    let fetcher_clone = fetcher.clone();
    let set_state_clone = set_state.clone();

    let dispatcher = set_state.ui_dispatcher();

    let effect = create_effect(move || {
        let _ = refetch_counter_read.get();

        // Abort any previous task before spawning a new one
        CURRENT_TASK_HANDLE.with(|h| {
            if let Some(handle) = h.borrow_mut().take() {
                handle.abort();
            }
        });

        let source_value = source_for_effect.get();
        let fetcher = fetcher_clone.clone();
        let dispatcher = dispatcher.clone();

        let tcb = current_thread_control_block().expect("GC not initialized");
        let scope = HandleScope::new(&tcb);
        let loading_state = Gc::new(ResourceState::Loading);
        let loading_handle = scope.handle(&loading_state);
        set_state_clone.set(loading_handle.to_gc());

        let rt = get_or_init_runtime();
        let handle = rt.spawn(async move {
            let result = fetcher(source_value).await;

            let new_state = match result {
                Ok(data) => ResourceState::Ready(data),
                Err(err) => ResourceState::Error(err),
            };

            dispatcher.set(Gc::new(new_state)).await;
        });

        // Store the handle so it can be aborted on cleanup
        CURRENT_TASK_HANDLE.with(|h| {
            *h.borrow_mut() = Some(handle);
        });
    });

    // Register effect with current component for proper lifecycle management
    // This ensures the effect is cleaned up when the component unmounts
    if let Some(component) = crate::runtime::current_owner() {
        component.add_effect(effect.clone());
    }

    // Register cleanup to abort the spawned task when the effect is cleaned up
    crate::effect::on_cleanup(move || {
        CURRENT_TASK_HANDLE.with(|h| {
            if let Some(handle) = h.borrow_mut().take() {
                handle.abort();
            }
        });
    });

    Resource { state, refetch_counter, source, effect }
}
