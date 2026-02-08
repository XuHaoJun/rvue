use std::future::Future;
use std::sync::Arc;

use rudo_gc::Gc;
use rudo_gc::Trace;

use super::dispatch::dispatch_to_ui;
use crate::effect::create_effect;
use crate::signal::{create_memo, create_signal, ReadSignal};

pub struct Resource<T: Trace + Clone + Send + Sync + 'static> {
    state: ReadSignal<Gc<ResourceState<T>>>,
    source: ReadSignal<()>,
}

impl<T: Trace + Clone + Send + Sync + 'static> Resource<T> {
    pub fn get(&self) -> Gc<ResourceState<T>> {
        self.state.get()
    }

    pub fn refetch(&self) {
        self.source.set(());
    }
}

#[derive(Clone, Debug)]
pub enum ResourceState<T: Trace + Clone + Send + Sync + 'static> {
    Pending,
    Loading,
    Ready(T),
    Error(String),
}

impl<T: Trace + Clone + Send + Sync + 'static> ResourceState<T> {
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

unsafe impl<T: Trace + Clone + Send + Sync + 'static> Trace for ResourceState<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        match self {
            ResourceState::Pending => {}
            ResourceState::Loading => {}
            ResourceState::Ready(t) => t.trace(visitor),
            ResourceState::Error(_) => {}
        }
    }
}

pub fn create_resource<S, T, Fu, Fetcher>(source: ReadSignal<S>, fetcher: Fetcher) -> Resource<T>
where
    S: PartialEq + Clone + Send + Sync + 'static,
    T: Trace + Clone + Send + Sync + 'static,
    Fu: Future<Output = Result<T, String>> + Send + 'static,
    Fetcher: Fn(S) -> Fu + Send + Sync + Clone + 'static,
{
    let (state, set_state) = create_signal(Gc::new(ResourceState::<T>::Pending));

    let (refetch_counter, _) = create_signal(0usize);

    let source_memo = create_memo(move || {
        let _ = refetch_counter.get();
        source.get()
    });

    let fetcher_clone = fetcher.clone();
    let set_state_clone = set_state.clone();

    create_effect(move || {
        let _source_value = source_memo.get();

        set_state_clone.set(Gc::new(ResourceState::Loading));

        let fetcher = fetcher_clone.clone();
        let source_value = source.get();
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
    });

    Resource { state, source: refetch_counter }
}
