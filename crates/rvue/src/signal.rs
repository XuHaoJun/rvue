//! Reactive signal implementation for fine-grained reactivity
//!
//! This module provides signal types built on top of rvue-signals core types,
//! adding subscriber tracking for the effect system.

pub use rvue_signals::create_signal as create_signal_base;
pub use rvue_signals::{SignalData, SignalRead, SignalWrite};

use crate::effect::{current_effect, Effect};
use rudo_gc::{Gc, Trace, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::Ordering;

thread_local! {
    pub static NOTIFYING: RefCell<bool> = const { RefCell::new(false) };
    static STRONG_SIGNAL_SUBS: RefCell<HashMap<usize, Vec<Gc<Effect>>>> =
        RefCell::new(HashMap::new());
}

/// Clear signal subscriptions without dropping Gc refs.
/// Call before test end to avoid TLS destroy-order: dropping Gc during our
/// thread_local destructor would trigger GC, which can access already-destroyed
/// rudo-gc thread locals.
#[doc(hidden)]
pub fn __test_clear_signal_subscriptions() {
    STRONG_SIGNAL_SUBS.with(|cell| {
        let mut map = cell.borrow_mut();
        for (_, vec) in map.drain() {
            std::mem::forget(vec);
        }
    });
}

pub(crate) trait SignalDataExt {
    fn subscribe(&self, signal_weak: &Weak<()>, effect: Gc<Effect>);
    fn notify_subscribers(&self);
}

impl<T: Trace + Clone + 'static> SignalDataExt for SignalData<T> {
    fn subscribe(&self, signal_weak: &Weak<()>, effect: Gc<Effect>) {
        // Skip if effect is being cleaned up (marked invalid)
        if !effect.is_valid() {
            return;
        }

        let weak_effect = Gc::downgrade(&effect);
        let effect_ptr = effect.as_ptr() as *const ();
        let signal_ptr = self as *const _ as *const ();
        let signal_ptr_usize = signal_ptr as usize;
        // Use Weak<()> for storage in SignalData (cast to opaque)
        let weak_opaque = weak_effect.cast::<()>();

        let already_subscribed = STRONG_SIGNAL_SUBS.with(|subs| {
            let subs = subs.borrow();
            subs.get(&signal_ptr_usize)
                .map(|list| list.iter().any(|e| e.as_ptr() as *const () == effect_ptr))
                .unwrap_or(false)
        });

        if !already_subscribed {
            log::debug!(
                "subscribe: Adding new subscription for effect {:?} to signal {:?} (weak={:#x})",
                effect_ptr,
                signal_ptr,
                weak_opaque.raw_addr()
            );
            STRONG_SIGNAL_SUBS.with(|subs| {
                let mut subs = subs.borrow_mut();
                let list = subs.entry(signal_ptr_usize).or_default();
                list.push(effect.clone());
                log::debug!("subscribe: Subscribers count: {} -> {}", list.len() - 1, list.len());
            });
            effect.add_subscription(signal_ptr_usize, signal_weak, &weak_opaque);
        } else {
            log::debug!(
                "subscribe: Effect {:?} already subscribed to signal {:?}",
                effect_ptr,
                signal_ptr
            );
            // Keep effect-side subscription list in sync across reruns.
            effect.add_subscription(signal_ptr_usize, signal_weak, &weak_opaque);
        }
    }

    fn notify_subscribers(&self) {
        let signal_ptr = self as *const _ as *const ();
        NOTIFYING.with(|n| *n.borrow_mut() = true);

        log::debug!("notify_subscribers: START signal {:?}", signal_ptr);

        // Build a clean subscriber list each notify:
        // - drop stale/invalid weak entries
        // - deduplicate by effect pointer
        let signal_ptr_usize = signal_ptr as usize;
        let effects_to_update: Vec<Gc<Effect>> = STRONG_SIGNAL_SUBS.with(|subs| {
            let mut subs = subs.borrow_mut();
            let list = subs.entry(signal_ptr_usize).or_default();
            list.retain(|e| e.is_valid());
            log::debug!("notify_subscribers: {} total subscribers", list.len());

            let mut seen_effect_ptrs: HashSet<*const ()> = HashSet::new();
            let mut deduped: Vec<Gc<Effect>> = Vec::with_capacity(list.len());
            for effect in list.iter() {
                let effect_ptr = effect.as_ptr() as *const ();
                if seen_effect_ptrs.insert(effect_ptr) {
                    deduped.push(effect.clone());
                }
            }
            *list = deduped.clone();
            deduped
        });

        log::debug!("notify_subscribers: {} effects to update", effects_to_update.len());
        for (i, effect) in effects_to_update.iter().enumerate() {
            let effect_ptr = effect.as_ptr() as *const ();
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
}

#[allow(dead_code)]
pub(crate) fn unsubscribe_by_ptr(signal_ptr: *const (), weak_opaque: &Weak<()>) {
    unsafe {
        // CAST SAFETY: All SignalData<T> are repr(C) and start with GcCell<Vec<Weak<()>>>.
        let subscribers_cell = &*signal_ptr.cast::<rudo_gc::GcCell<Vec<Weak<()>>>>();
        let mut subscribers = subscribers_cell.borrow_mut_gen_only();

        // Manual filter to avoid Drop on invalid Weaks
        let mut new_subscribers = Vec::with_capacity(subscribers.len());
        for weak in subscribers.drain(..) {
            if !Weak::ptr_eq(&weak, weak_opaque) {
                new_subscribers.push(weak);
            }
        }
        *subscribers = new_subscribers;
    }
}

#[derive(Clone)]
pub struct ReadSignal<T: Trace + Clone + 'static> {
    pub(crate) data: Gc<SignalData<T>>,
}

impl<T: Trace + Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        let signal_ptr = self.data.as_ptr() as *const ();
        if let Some(effect) = current_effect() {
            let effect_ptr = effect.as_ptr() as *const ();
            let signal_weak = Gc::downgrade(&self.data).cast::<()>();
            log::debug!(
                "ReadSignal::get: effect {:?} subscribing to signal {:?}",
                effect_ptr,
                signal_ptr
            );
            self.data.subscribe(&signal_weak, effect);
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
        self.data.value.borrow().clone()
    }
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadSignal")
            .field("data_ptr", &self.data.as_ptr())
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
    pub(crate) data: Gc<SignalData<T>>,
}

impl<T: Trace + Clone + 'static> WriteSignal<T> {
    #[inline(always)]
    pub fn set(&self, value: T) {
        let signal_ptr = self.data.as_ptr() as *const ();
        log::debug!("WriteSignal::set: signal {:?} setting new value", signal_ptr);
        // Defer drop of old value until after notify_subscribers completes.
        let _old_value = {
            let mut guard = self.data.value.borrow_mut_simple();
            std::mem::replace(&mut *guard, value)
        };
        self.data.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
        log::debug!("WriteSignal::set: notified subscribers");
    }

    #[inline(always)]
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let signal_ptr = self.data.as_ptr() as *const ();
        log::debug!("WriteSignal::update: signal {:?} starting update", signal_ptr);
        let _old_value = {
            let mut guard = self.data.value.borrow_mut_simple();
            let old = (*guard).clone();
            f(&mut *guard);
            old
        };
        self.data.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
        log::debug!("WriteSignal::update: notified subscribers");
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
        *self.data.value.borrow_mut_simple() = value;
        self.data.version.fetch_add(1, Ordering::SeqCst);
    }
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteSignal")
            .field("data_ptr", &self.data.as_ptr())
            .field("version", &self.data.version())
            .finish()
    }
}

unsafe impl<T: Trace + Clone + 'static> Trace for WriteSignal<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.data.trace(visitor);
    }
}

impl<T: Trace + Clone + 'static> SignalRead<T> for ReadSignal<T> {
    fn get(&self) -> T {
        ReadSignal::get(self)
    }

    fn get_untracked(&self) -> T {
        self.data.get()
    }
}

impl<T: Trace + Clone + 'static> SignalWrite<T> for WriteSignal<T> {
    #[inline(always)]
    fn set(&self, value: T) {
        WriteSignal::set(self, value);
    }

    #[inline(always)]
    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        WriteSignal::update(self, f);
    }
}

pub fn create_signal<T: Trace + Clone + 'static>(
    initial_value: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    let escaped_gc = Gc::new(SignalData::new(initial_value));

    (ReadSignal { data: Gc::clone(&escaped_gc) }, WriteSignal { data: escaped_gc })
}

pub fn leak_effect(effect: Gc<Effect>) {
    // Register as global GC root for conservative tracing integrations.
    let ptr = effect.as_ptr();
    log::debug!("leak_effect: registering effect {:?}", ptr);
    rudo_gc::tokio::GcRootSet::global().register(ptr as usize);
    // Intentionally leak one strong reference so this effect outlives thread-local
    // storage lifetimes. Reactive global/memo effects are expected to be process-long.
    std::mem::forget(effect);
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
    log::debug!(
        "create_memo: memo effect {:?} -> signal {:?}",
        effect.as_ptr(),
        read.data.as_ptr()
    );

    leak_effect(effect);

    read
}

pub fn create_memo_with_equality<T: Trace + Clone + PartialEq + 'static, F>(f: F) -> ReadSignal<T>
where
    F: Fn() -> T + 'static,
{
    let initial_value = crate::effect::untracked(&f);
    let (read, write) = create_signal(initial_value.clone());

    let last_value = rudo_gc::GcCell::new(initial_value);
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
    log::debug!(
        "create_memo_with_equality: memo effect {:?} -> signal {:?}",
        effect.as_ptr(),
        read.data.as_ptr()
    );

    leak_effect(effect);

    read
}
