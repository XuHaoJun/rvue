use std::future::Future;
use std::sync::Arc;

use rudo_gc::Gc;
use rudo_gc::Trace;

use super::dispatch::dispatch_to_ui;
use crate::signal::{create_signal, ReadSignal};

pub struct Resource<T: Trace + Clone + 'static> {
    state: ReadSignal<Gc<ResourceState<T>>>,
    refetch_fn: Arc<dyn Fn()>,
}

impl<T: Trace + Clone + 'static> Resource<T> {
    pub fn get(&self) -> Gc<ResourceState<T>> {
        self.state.get()
    }

    pub fn refetch(&self) {
        (self.refetch_fn)();
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

pub fn create_resource<S, T, Fu, Fetcher>(source: S, fetcher: Fetcher) -> Resource<T>
where
    S: Fn() -> T + Send + Sync + Clone + 'static,
    T: Trace + Clone + 'static,
    Fu: Future<Output = Result<T, String>> + Send + 'static,
    Fetcher: Fn(T) -> Fu + Send + Sync + Clone + 'static,
{
    let (state, set_state) = create_signal(Gc::new(ResourceState::<T>::Pending));

    set_state.set(Gc::new(ResourceState::Loading));

    let fetcher_clone = fetcher.clone();
    let source_clone = source.clone();
    let set_state_clone = set_state.clone();

    let do_fetch = move || {
        let source_value = source_clone();
        let fetcher = fetcher_clone.clone();
        let set_state = set_state_clone.clone();

        dispatch_to_ui(move || {
            let rt = tokio::runtime::Handle::current();
            let result = rt.block_on(async move { fetcher(source_value).await });
            match result {
                Ok(data) => {
                    set_state.set(Gc::new(ResourceState::Ready(data)));
                }
                Err(err) => {
                    set_state.set(Gc::new(ResourceState::Error(err)));
                }
            }
        });
    };

    do_fetch();

    let refetch_fn: Arc<dyn Fn()> = Arc::new(do_fetch);
    let refetch_fn_clone = refetch_fn.clone();
    let refetch_fn_final = Arc::new(move || {
        refetch_fn_clone();
    });

    Resource { state: state.clone(), refetch_fn: refetch_fn_final }
}
