use std::cell::RefCell;
use std::collections::VecDeque;

use winit::event_loop::EventLoopProxy;

use crate::app::RvueUserEvent;

pub type UiCallback = Box<dyn FnOnce() + Send + 'static>;

thread_local! {
    static LOCAL_CALLBACKS: RefCell<VecDeque<Box<dyn FnOnce() + 'static>>> = RefCell::new(VecDeque::new());
}

static GLOBAL_CALLBACKS: std::sync::Mutex<VecDeque<UiCallback>> =
    std::sync::Mutex::new(VecDeque::new());
static PROXY: std::sync::OnceLock<EventLoopProxy<RvueUserEvent>> = std::sync::OnceLock::new();

pub struct UiDispatchQueue;

impl UiDispatchQueue {
    pub fn set_proxy(proxy: EventLoopProxy<RvueUserEvent>) {
        let _ = PROXY.set(proxy);
    }

    fn get_proxy() -> Option<&'static EventLoopProxy<RvueUserEvent>> {
        PROXY.get()
    }

    pub fn dispatch<F>(callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut queue = GLOBAL_CALLBACKS.lock().unwrap();
        queue.push_back(Box::new(callback));

        if let Some(proxy) = Self::get_proxy() {
            let _ = proxy.send_event(RvueUserEvent::AsyncDispatchReady);
        }
    }

    pub fn drain_local_and_execute() {
        LOCAL_CALLBACKS.with(|cell| {
            let mut callbacks = cell.borrow_mut();
            let callbacks_vec: Vec<_> = callbacks.drain(..).collect();
            drop(callbacks);

            for callback in callbacks_vec {
                if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback))
                {
                    log::error!("Panic in dispatch_to_ui callback: {:?}", panic);
                }
            }
        });
    }

    pub fn drain_global_and_execute() {
        let mut queue = GLOBAL_CALLBACKS.lock().unwrap();
        let callbacks: Vec<_> = queue.drain(..).collect();
        drop(queue);

        for callback in callbacks {
            if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)) {
                log::error!("Panic in dispatch_to_ui callback: {:?}", panic);
            }
        }
    }

    pub fn drain_all_and_execute() {
        Self::drain_local_and_execute();
        Self::drain_global_and_execute();
    }

    pub fn len() -> usize {
        let global_len = GLOBAL_CALLBACKS.lock().unwrap().len();
        let local_len = LOCAL_CALLBACKS.with(|cell| cell.borrow().len());
        global_len + local_len
    }

    pub fn is_empty() -> bool {
        let global_empty = GLOBAL_CALLBACKS.lock().unwrap().is_empty();
        let local_empty = LOCAL_CALLBACKS.with(|cell| cell.borrow().is_empty());
        global_empty && local_empty
    }
}

pub fn dispatch_to_ui<F>(callback: F)
where
    F: FnOnce() + 'static,
{
    LOCAL_CALLBACKS.with(|cell| {
        cell.borrow_mut().push_back(Box::new(callback));
    });

    if let Some(proxy) = UiDispatchQueue::get_proxy() {
        let _ = proxy.send_event(RvueUserEvent::AsyncDispatchReady);
    }
}

pub fn dispatch_cross_thread<F>(callback: F)
where
    F: FnOnce() + Send + 'static,
{
    UiDispatchQueue::dispatch(callback);
}
