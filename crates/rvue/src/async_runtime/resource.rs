use std::future::Future;

use rudo_gc::Gc;
use rudo_gc::Trace;

use super::task::block_on;
use crate::effect::create_effect;
use crate::signal::{create_signal, ReadSignal, WriteSignal, LEAKED_EFFECTS};

#[derive(Clone)]
pub struct Resource<T: Trace + Clone + 'static, S: Trace + Clone + 'static> {
    state: ReadSignal<Gc<ResourceState<T>>>,
    refetch_counter: WriteSignal<usize>,
    source: ReadSignal<S>,
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

pub fn create_resource<S, T, Fu, Fetcher>(source: ReadSignal<S>, fetcher: Fetcher) -> Resource<T, S>
where
    S: PartialEq + Clone + Trace + 'static,
    T: Trace + Clone + 'static,
    Fu: Future<Output = Result<T, String>> + Send + 'static,
    Fetcher: Fn(S) -> Fu + Clone + 'static,
{
    let (state, set_state) = create_signal(Gc::new(ResourceState::<T>::Pending));

    let (refetch_counter_read, refetch_counter) = create_signal(0usize);

    let source_for_effect = source.clone();
    let fetcher_clone = fetcher.clone();
    let set_state_clone = set_state.clone();

    let effect = create_effect(move || {
        let _ = refetch_counter_read.get();

        set_state_clone.set(Gc::new(ResourceState::Loading));

        let fetcher = fetcher_clone.clone();
        let source_value = source_for_effect.get();

        let result = block_on(fetcher(source_value));
        match result {
            Ok(data) => {
                set_state_clone.set(Gc::new(ResourceState::Ready(data)));
            }
            Err(err) => {
                set_state_clone.set(Gc::new(ResourceState::Error(err)));
            }
        }
    });

    LEAKED_EFFECTS.with(|cell| {
        let mut leaked = cell.borrow_mut();
        leaked.push(effect);
    });

    Resource { state, refetch_counter, source }
}
