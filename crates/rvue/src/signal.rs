//! Reactive signal implementation for fine-grained reactivity
//!
//! This module provides signal types built on top of rvue-signals core types,
//! adding subscriber tracking for the effect system.

pub use rvue_signals::{create_signal as create_signal_base, SignalData, SignalRead, SignalWrite};

use crate::effect::{current_effect, Effect};
use rudo_gc::handles::HandleScope;
use rudo_gc::heap::current_thread_control_block;
use rudo_gc::{Gc, GcCell, Trace, Weak};
use std::cell::RefCell;
use std::sync::atomic::Ordering;

thread_local! {
    static LEAKED_EFFECTS: RefCell<Vec<Gc<Effect>>> = const { RefCell::new(Vec::new()) };
    pub static NOTIFYING: RefCell<bool> = const { RefCell::new(false) };
}

/// Internal signal data structure containing the value, version tracking, and subscribers.
///
/// This is exposed for async runtime integration.
pub struct SignalDataInner<T: Trace + Clone + 'static> {
    pub(crate) inner: SignalData<T>,
    pub(crate) subscribers: GcCell<Vec<Weak<Effect>>>,
}

impl<T: Trace + Clone + 'static> SignalDataInner<T> {
    pub fn new(value: T) -> Self {
        Self { inner: SignalData::new(value), subscribers: GcCell::new(Vec::new()) }
    }

    #[inline(always)]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.get()
    }

    #[inline(always)]
    pub fn version(&self) -> u64 {
        self.inner.version()
    }

    pub(crate) fn subscribe(&self, effect: Gc<Effect>) {
        // Skip if effect is being cleaned up (marked invalid)
        if !effect.is_valid() {
            return;
        }

        let weak_effect = Gc::downgrade(&effect);
        let effect_ptr = Gc::as_ptr(&effect) as *const ();
        let signal_ptr = self as *const _ as *const ();
        let signal_ptr_usize = signal_ptr as usize;

        let already_subscribed = {
            let subscribers = self.subscribers.borrow();
            subscribers.iter().any(|sub| {
                sub.try_upgrade()
                    .map(|e| (Gc::as_ptr(&e) as *const ()) == effect_ptr)
                    .unwrap_or(false)
            })
        };

        if !already_subscribed {
            log::debug!(
                "subscribe: Adding new subscription for effect {:?} to signal {:?}",
                effect_ptr,
                signal_ptr
            );
            let mut subscribers = self.subscribers.borrow_mut_gen_only();
            let len_before = subscribers.len();
            subscribers.push(weak_effect.clone());
            log::debug!("subscribe: Subscribers count: {} -> {}", len_before, subscribers.len());
            effect.add_subscription(signal_ptr_usize, &weak_effect);
        } else {
            log::debug!(
                "subscribe: Effect {:?} already subscribed to signal {:?}",
                effect_ptr,
                signal_ptr
            );
        }
    }

    pub(crate) fn notify_subscribers(&self) {
        let signal_ptr = self as *const _ as *const ();
        NOTIFYING.with(|n| *n.borrow_mut() = true);

        log::debug!("notify_subscribers: START signal {:?}", signal_ptr);

        // First, get valid effects to notify
        let effects_to_update: Vec<Gc<Effect>> = {
            // Clean up stale weak refs safely - avoid Drop on invalid Weaks
            // Use a scope to release mutable borrow before immutable borrow
            let subscriber_count: usize;
            {
                let mut subscribers = self.subscribers.borrow_mut_gen_only();
                let valid_count = subscribers.iter().filter(|w| w.try_upgrade().is_some()).count();
                let mut new_subscribers = Vec::with_capacity(valid_count);
                for weak in subscribers.drain(..) {
                    if weak.try_upgrade().is_some() {
                        new_subscribers.push(weak);
                    }
                }
                *subscribers = new_subscribers;
                subscriber_count = subscribers.len();
            }
            // Mutable borrow released here

            let subscribers = self.subscribers.borrow();
            log::debug!(
                "notify_subscribers: {} total subscribers (had {} before cleanup)",
                subscribers.len(),
                subscriber_count
            );
            // Filter: must be able to upgrade AND effect must be valid
            subscribers
                .iter()
                .filter_map(|weak| weak.try_upgrade())
                .filter(|e| e.is_valid())
                .collect()
        };

        log::debug!("notify_subscribers: {} effects to update", effects_to_update.len());
        for (i, effect) in effects_to_update.iter().enumerate() {
            let effect_ptr = Gc::as_ptr(effect) as *const ();
            log::debug!("notify_subscribers: effect[{}] ptr={:?}", i, effect_ptr);
        }

        // Run effects
        for effect in effects_to_update.iter() {
            effect.mark_dirty();
        }

        for effect in effects_to_update.iter() {
            if effect.is_dirty() {
                Effect::update_if_dirty(effect);
            }
        }

        NOTIFYING.with(|n| *n.borrow_mut() = false);
    }

    #[allow(dead_code)]
    pub(crate) fn unsubscribe_by_ptr(effect_ptr: *const (), weak_effect: &Weak<Effect>) {
        unsafe {
            let signal = &*effect_ptr.cast::<SignalDataInner<()>>();
            let mut subscribers = signal.subscribers.borrow_mut_gen_only();

            // Manual filter to avoid Drop on invalid Weaks
            let mut new_subscribers = Vec::with_capacity(subscribers.len());
            for weak in subscribers.drain(..) {
                if !Weak::ptr_eq(&weak, weak_effect) {
                    new_subscribers.push(weak);
                }
            }
            *subscribers = new_subscribers;
        }
    }
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for SignalDataInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalDataInner").field("version", &self.inner.version()).finish()
    }
}

unsafe impl<T: Trace + Clone + 'static> Trace for SignalDataInner<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.inner.trace(visitor);

        // Skip tracing subscribers entirely to avoid RefCell borrow conflicts
        // and issues with stale weak refs during GC.
        // The effect system keeps effects alive through its own mechanisms.
    }
}

#[derive(Clone)]
pub struct ReadSignal<T: Trace + Clone + 'static> {
    pub(crate) data: Gc<SignalDataInner<T>>,
}

impl<T: Trace + Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        let signal_ptr = Gc::as_ptr(&self.data) as *const ();
        if let Some(effect) = current_effect() {
            let effect_ptr = Gc::as_ptr(&effect) as *const ();
            log::debug!(
                "ReadSignal::get: effect {:?} subscribing to signal {:?}",
                effect_ptr,
                signal_ptr
            );
            self.data.subscribe(effect);
        } else {
            log::debug!(
                "ReadSignal::get: No current effect for signal {:?}, not subscribing",
                signal_ptr
            );
        }
        self.data.get()
    }

    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        self.data.get()
    }

    /// Gets the value WITHOUT effect tracking or scope validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// 1. The signal is still alive (not garbage collected)
    /// 2. Any necessary effect tracking is handled separately
    ///
    /// This method skips the effect subscription check for performance.
    /// Use in hot paths where you can prove the signal is valid.
    #[inline]
    pub unsafe fn get_unchecked(&self) -> T
    where
        T: Clone,
    {
        self.data.inner.get()
    }
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadSignal")
            .field("data_ptr", &Gc::as_ptr(&self.data))
            .field("version", &self.data.version())
            .finish()
    }
}

impl<T: Trace + Clone + std::fmt::Display + 'static> std::fmt::Display for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data.get())
    }
}

unsafe impl<T: Trace + Clone + 'static> Trace for ReadSignal<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.data.trace(visitor);
    }
}

#[derive(Clone)]
pub struct WriteSignal<T: Trace + Clone + 'static> {
    pub(crate) data: Gc<SignalDataInner<T>>,
}

impl<T: Trace + Clone + 'static> WriteSignal<T> {
    #[inline(always)]
    pub fn set(&self, value: T) {
        let signal_ptr = Gc::as_ptr(&self.data) as *const ();
        log::debug!("WriteSignal::set: signal {:?} setting new value", signal_ptr);
        // Defer drop of old value until after notify_subscribers completes.
        // This prevents double-free when cascading effects trigger GC collection
        // while the old Gc's GcBox memory is being freed.
        let _old_value = {
            let mut guard = self.data.inner.value.borrow_mut_gen_only();
            std::mem::replace(&mut *guard, value)
        };
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
        log::debug!("WriteSignal::set: notified subscribers");
        // _old_value dropped here, after all cascading effects complete
    }

    #[inline(always)]
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let signal_ptr = Gc::as_ptr(&self.data) as *const ();
        log::debug!("WriteSignal::update: signal {:?} starting update", signal_ptr);
        // Capture old value before mutation for deferred drop
        let _old_value = {
            let mut guard = self.data.inner.value.borrow_mut_gen_only();
            let old = (*guard).clone();
            f(&mut *guard);
            old
        };
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
        log::debug!("WriteSignal::update: notified subscribers");
        // _old_value dropped here, after all cascading effects complete
    }

    /// Sets the value WITHOUT effect tracking or scope validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// 1. The signal is still alive (not garbage collected)
    /// 2. Any necessary effect notifications are handled separately
    ///
    /// This method skips the normal update path for performance.
    /// Use in hot paths where you can prove the signal is valid.
    #[inline]
    pub unsafe fn set_unchecked(&self, value: T) {
        *self.data.inner.value.borrow_mut_gen_only() = value;
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
    }
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteSignal")
            .field("data_ptr", &Gc::as_ptr(&self.data))
            .field("version", &self.data.version())
            .finish()
    }
}

impl<T: Trace + Clone + 'static> SignalRead<T> for ReadSignal<T> {
    fn get(&self) -> T {
        if let Some(effect) = current_effect() {
            self.data.subscribe(effect);
        }
        self.data.get()
    }

    fn get_untracked(&self) -> T {
        self.data.get()
    }
}

impl<T: Trace + Clone + 'static> SignalWrite<T> for WriteSignal<T> {
    #[inline(always)]
    fn set(&self, value: T) {
        let _old_value = {
            let mut guard = self.data.inner.value.borrow_mut_gen_only();
            std::mem::replace(&mut *guard, value)
        };
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }

    #[inline(always)]
    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let _old_value = {
            let mut guard = self.data.inner.value.borrow_mut_gen_only();
            let old = (*guard).clone();
            f(&mut *guard);
            old
        };
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }
}

pub fn create_signal<T: Trace + Clone + 'static>(
    initial_value: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    let tcb = current_thread_control_block().expect("GC not initialized");
    let scope = HandleScope::new(&tcb);

    let data = Gc::new(SignalDataInner::new(initial_value));
    let handle = scope.handle(&data);
    let escaped_gc = handle.to_gc();

    (ReadSignal { data: Gc::clone(&escaped_gc) }, WriteSignal { data: escaped_gc })
}

pub fn create_memo<T: Trace + Clone + 'static, F>(f: F) -> ReadSignal<T>
where
    F: Fn() -> T + 'static,
{
    let initial_value = crate::effect::untracked(&f);
    let (read, write) = create_signal(initial_value.clone());

    let f_shared = std::rc::Rc::new(f);
    let f_clone = f_shared.clone();

    let is_first = std::cell::Cell::new(true);
    let effect = crate::effect::create_effect(move || {
        let value = f_clone();
        if is_first.replace(false) {
        } else {
            write.set(value);
        }
    });

    LEAKED_EFFECTS.with(|cell| {
        let mut leaked = cell.borrow_mut();
        leaked.push(effect);
    });

    read
}

pub fn create_memo_with_equality<T: Trace + Clone + PartialEq + 'static, F>(f: F) -> ReadSignal<T>
where
    F: Fn() -> T + 'static,
{
    let initial_value = crate::effect::untracked(&f);
    let (read, write) = create_signal(initial_value.clone());

    let last_value = GcCell::new(initial_value);
    let f_shared = std::rc::Rc::new(f);
    let f_clone = f_shared.clone();

    let is_first = std::cell::Cell::new(true);
    let effect = crate::effect::create_effect(move || {
        let new_value = f_clone();
        if is_first.replace(false) {
        } else if new_value != *last_value.borrow() {
            *last_value.borrow_mut_gen_only() = new_value.clone();
            write.set(new_value);
        }
    });

    LEAKED_EFFECTS.with(|cell| {
        let mut leaked = cell.borrow_mut();
        leaked.push(effect);
    });

    read
}
