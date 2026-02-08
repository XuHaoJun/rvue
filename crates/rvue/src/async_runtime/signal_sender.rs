use std::sync::Arc;

use super::dispatch::dispatch_to_ui;

#[derive(Clone)]
pub struct SignalSender<T: Clone + Send + 'static> {
    setter: Arc<dyn Fn(T) + Send + Sync>,
}

impl<T: Clone + Send + 'static> SignalSender<T> {
    pub fn new<F>(setter: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        Self { setter: Arc::new(setter) }
    }

    pub fn set(&self, value: T) {
        let setter = self.setter.clone();
        dispatch_to_ui(move || {
            setter(value);
        });
    }
}

unsafe impl<T: Clone + Send + 'static> Send for SignalSender<T> {}
unsafe impl<T: Clone + Send + 'static> Sync for SignalSender<T> {}
