//! Reactive signal implementation for fine-grained reactivity
//!
//! This module provides signal types built on top of rvue-signals core types,
//! adding subscriber tracking for the effect system.

pub use rvue_signals::{create_signal as create_signal_base, SignalData, SignalRead, SignalWrite};

use crate::effect::{current_effect, Effect};
use rudo_gc::{Gc, GcCell, Trace, Weak};
use std::cell::RefCell;
use std::sync::atomic::Ordering;

thread_local! {
    pub(crate) static LEAKED_EFFECTS: RefCell<Vec<Gc<Effect>>> = const { RefCell::new(Vec::new()) };
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
        let weak_effect = Gc::downgrade(&effect);
        let effect_ptr = Gc::as_ptr(&effect) as *const ();
        let signal_ptr = self as *const _ as *const ();

        let already_subscribed = {
            let subscribers = self.subscribers.borrow();
            subscribers.iter().any(|sub| {
                let upgraded =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sub.upgrade()));
                upgraded
                    .ok()
                    .flatten()
                    .map(|e| (Gc::as_ptr(&e) as *const ()) == effect_ptr)
                    .unwrap_or(false)
            })
        };

        if !already_subscribed {
            let mut subscribers = self.subscribers.borrow_mut_gen_only();
            subscribers.push(weak_effect.clone());
            effect.add_subscription(signal_ptr, &weak_effect);
        }
    }

    pub(crate) fn notify_subscribers(&self) {
        let effects_to_update: Vec<Gc<Effect>> = {
            let subscribers = self.subscribers.borrow();
            subscribers
                .iter()
                .filter_map(|weak| {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| weak.upgrade()))
                        .ok()
                        .flatten()
                })
                .collect()
        };

        for effect in effects_to_update.iter() {
            effect.mark_dirty();
        }

        for effect in effects_to_update.iter() {
            if effect.is_dirty() {
                Effect::update_if_dirty(effect);
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn unsubscribe_by_ptr(effect_ptr: *const (), weak_effect: &Weak<Effect>) {
        if effect_ptr.is_null() {
            return;
        }

        let addr = effect_ptr as usize;
        if addr < 4096 || addr % std::mem::align_of::<SignalDataInner<()>>() != 0 {
            return;
        }

        unsafe {
            let signal = &*effect_ptr.cast::<SignalDataInner<()>>();
            let mut subscribers = signal.subscribers.borrow_mut_gen_only();
            subscribers.retain(|weak| !Weak::ptr_eq(weak, weak_effect));
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
        let subscribers = self.subscribers.borrow();

        let mut valid_subscribers = Vec::new();
        for weak in subscribers.iter() {
            let upgraded =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| weak.upgrade()));
            if let Ok(Some(effect)) = upgraded {
                valid_subscribers.push(effect);
            }
        }

        for effect in valid_subscribers {
            effect.trace(visitor);
        }
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
        if let Some(effect) = current_effect() {
            self.data.subscribe(effect);
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
        *self.data.inner.value.borrow_mut_gen_only() = value;
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }

    #[inline(always)]
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut *self.data.inner.value.borrow_mut_gen_only());
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }

    /// Sets the value WITHOUT effect tracking or scope validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// 1. The signal is still alive (not garbage collected)
    /// 2. Any necessary effect tracking is handled separately
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
        *self.data.inner.value.borrow_mut_gen_only() = value;
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }

    #[inline(always)]
    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut *self.data.inner.value.borrow_mut_gen_only());
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }
}

pub fn create_signal<T: Trace + Clone + 'static>(
    initial_value: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    let data = Gc::new(SignalDataInner::new(initial_value));
    (ReadSignal { data: Gc::clone(&data) }, WriteSignal { data })
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
