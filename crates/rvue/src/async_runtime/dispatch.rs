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

        // Drain any pending callbacks that were queued before proxy was available
        Self::drain_global_and_execute();
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
        } else {
            log::warn!("[Dispatch] NO PROXY - callback queued but not executed!");
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
                    log::error!("Panic in drain_local_and_execute callback: {:?}", panic);
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
                log::error!("Panic in drain_global_and_execute callback: {:?}", panic);
            }
        }
    }

    #[doc(hidden)]
    pub fn force_drain_all() {
        Self::drain_all_and_execute();
    }

    /// Spawn a closure to run on the main thread, blocking until it completes.
    ///
    /// # Warning: Do NOT call from the UI thread
    ///
    /// This function uses a blocking `recv()` call. If invoked from the UI thread,
    /// it will deadlock because the UI thread is blocked waiting for a callback
    /// that can only execute when the UI thread is free.
    ///
    /// Use `dispatch()` or `dispatch_to_ui()` for async dispatch instead.
    #[doc(hidden)]
    pub fn spawn_main_thread<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        GLOBAL_CALLBACKS.lock().unwrap().push_back(Box::new(move || {
            f();
            let _ = tx.send(());
        }));

        let proxy = Self::get_proxy();
        if let Some(p) = proxy {
            let _ = p.send_event(RvueUserEvent::AsyncDispatchReady);
        }

        let _ = rx.recv();
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
    F: FnOnce() + Send + 'static,
{
    let mut queue = GLOBAL_CALLBACKS.lock().unwrap();
    queue.push_back(Box::new(callback));
    drop(queue);

    if let Some(proxy) = UiDispatchQueue::get_proxy() {
        let _ = proxy.send_event(RvueUserEvent::AsyncDispatchReady);
    }
}
