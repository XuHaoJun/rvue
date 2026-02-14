use std::cell::RefCell;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rudo_gc::tokio::GcTokioExt;
use rudo_gc::Gc;
use rudo_gc::Trace;

use crate::async_runtime::cancellation::Cancellation;
use crate::async_runtime::get_or_init_runtime;
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

        log::debug!("[Resource] Setting Loading state, version={}", current_version);

        // Set loading state - now thread-safe with GcThreadSafeCell
        set_state_clone.set(Gc::new(ResourceState::Loading));
        println!("[Resource] Set Loading state");

        // Non-blocking approach: spawn async task and dispatch result to main thread
        // Use GcHandle to pass signal data across threads (GcHandle is Send+Sync)
        let rt = get_or_init_runtime();

        // Get the signal's inner Gc and create a cross-thread handle for it
        let signal_gc = set_state_clone.data.clone();
        let signal_handle = signal_gc.cross_thread_handle();

        // Clone version and cancellation for both async task and dispatch
        let version_for_task = Arc::clone(&version_clone);
        let version_for_dispatch = Arc::clone(&version_clone);
        let cancellation_for_task = task_cancellation.clone();
        let cancellation_for_dispatch = task_cancellation.clone();

        // Create channel to send result from async task to main thread
        let (tx, rx) = std::sync::mpsc::channel();

        // Spawn async task (non-blocking - returns immediately)
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
                return;
            }

            // Send result through channel to main thread
            println!("[Resource] Sending result via channel, version={}", current_version);
            let _ = tx.send(result);
        });

        // Dispatch callback to run on main thread - this will receive result and update signal
        // Uses GcHandle which is Send+Sync to avoid capturing non-Send WriteSignal
        use crate::async_runtime::dispatch::UiDispatchQueue;
        UiDispatchQueue::dispatch(move || {
            // Try to receive the result (non-blocking)
            match rx.try_recv() {
                Ok(result) => {
                    // Check version
                    if version_for_dispatch.load(Ordering::SeqCst) != current_version {
                        println!("[Resource] Task stale in dispatch callback");
                        return;
                    }

                    // Check cancellation
                    if cancellation_for_dispatch.is_cancelled() {
                        println!("[Resource] Task cancelled in dispatch");
                        return;
                    }

                    let new_state = match result {
                        Ok(data) => ResourceState::Ready(data),
                        Err(err) => ResourceState::Error(err),
                    };

                    println!(
                        "[Resource] Setting final state from dispatch, version={}",
                        current_version
                    );

                    // Resolve GcHandle to get Gc<SignalDataInner>
                    let signal_inner = signal_handle.resolve();

                    // Create final state GC object
                    let final_state = Gc::new(new_state);
                    let _guard = final_state.root_guard();

                    // Directly update signal inner (same as WriteSignal::set)
                    *signal_inner.inner.value.borrow_mut_simple() = final_state;
                    signal_inner.inner.version.fetch_add(1, Ordering::SeqCst);
                    signal_inner.notify_subscribers();

                    println!("[Resource] State updated successfully, version={}", current_version);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Result not ready yet - re-dispatch to try again later
                    // This is the key to non-blocking: keep re-dispatching until result is ready
                    println!("[Resource] Result not ready, re-dispatching...");
                    use crate::async_runtime::dispatch::UiDispatchQueue;
                    UiDispatchQueue::dispatch(move || {
                        // Re-use the same logic - this creates a loop until result is ready
                        match rx.try_recv() {
                            Ok(result) => {
                                if version_for_dispatch.load(Ordering::SeqCst) != current_version {
                                    return;
                                }
                                if cancellation_for_dispatch.is_cancelled() {
                                    return;
                                }
                                let new_state = match result {
                                    Ok(data) => ResourceState::Ready(data),
                                    Err(err) => ResourceState::Error(err),
                                };
                                println!(
                                    "[Resource] Setting final state from re-dispatch, version={}",
                                    current_version
                                );
                                let signal_inner = signal_handle.resolve();
                                let final_state = Gc::new(new_state);
                                let _guard = final_state.root_guard();
                                *signal_inner.inner.value.borrow_mut_simple() = final_state;
                                signal_inner.inner.version.fetch_add(1, Ordering::SeqCst);
                                signal_inner.notify_subscribers();
                                println!(
                                    "[Resource] State updated successfully, version={}",
                                    current_version
                                );
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                // Still not ready, re-dispatch again
                                println!("[Resource] Still not ready, re-dispatching again...");
                                UiDispatchQueue::dispatch(move || {
                                    match rx.try_recv() {
                                        Ok(r) => {
                                            if version_for_dispatch.load(Ordering::SeqCst)
                                                != current_version
                                            {
                                                return;
                                            }
                                            if cancellation_for_dispatch.is_cancelled() {
                                                return;
                                            }
                                            let ns = match r {
                                                Ok(d) => ResourceState::Ready(d),
                                                Err(e) => ResourceState::Error(e),
                                            };
                                            println!("[Resource] Setting final state from 2nd re-dispatch, version={}", current_version);
                                            let si = signal_handle.resolve();
                                            let fs = Gc::new(ns);
                                            let _g = fs.root_guard();
                                            *si.inner.value.borrow_mut_simple() = fs;
                                            si.inner.version.fetch_add(1, Ordering::SeqCst);
                                            si.notify_subscribers();
                                        }
                                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                                            // Give up after multiple tries - will be handled by next effect trigger
                                            println!("[Resource] Giving up after multiple tries");
                                        }
                                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                            println!("[Resource] Channel disconnected!");
                                        }
                                    }
                                });
                            }
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                println!("[Resource] Channel disconnected!");
                            }
                        }
                    });
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    println!("[Resource] Channel disconnected!");
                }
            }
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
