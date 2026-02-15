use std::collections::HashMap;
use std::sync::Mutex;

use crate::ComponentId;

use super::task::{IntervalHandle, SignalWatcher, TaskHandle};

thread_local! {
    pub static REGISTRY: Mutex<TaskRegistryInner> = Mutex::new(TaskRegistryInner::new());
}

pub struct TaskRegistryInner {
    task_handles: HashMap<ComponentId, Vec<TaskHandle>>,
    watchers: HashMap<ComponentId, Vec<SignalWatcher>>,
    intervals: HashMap<ComponentId, Vec<IntervalHandle>>,
}

impl TaskRegistryInner {
    fn new() -> Self {
        Self { task_handles: HashMap::new(), watchers: HashMap::new(), intervals: HashMap::new() }
    }

    pub fn register_task(&mut self, component_id: ComponentId, handle: TaskHandle) {
        self.task_handles.entry(component_id).or_default().push(handle);
    }

    pub fn register_watcher(&mut self, component_id: ComponentId, watcher: SignalWatcher) {
        self.watchers.entry(component_id).or_default().push(watcher);
    }

    pub fn register_interval(&mut self, component_id: ComponentId, handle: IntervalHandle) {
        self.intervals.entry(component_id).or_default().push(handle);
    }

    pub fn cancel_all(&mut self, component_id: ComponentId) {
        for handle in self.task_handles.remove(&component_id).into_iter().flatten() {
            handle.abort();
        }
        for watcher in self.watchers.remove(&component_id).into_iter().flatten() {
            watcher.stop();
        }
        for handle in self.intervals.remove(&component_id).into_iter().flatten() {
            handle.stop();
        }
    }

    pub fn cleanup_completed(&mut self) {
        for handles in self.task_handles.values_mut() {
            handles.retain(|h| h.is_running());
        }
        for handles in self.intervals.values_mut() {
            handles.retain(|h| h.is_running());
        }
    }

    pub fn task_count(&self, component_id: ComponentId) -> usize {
        self.task_handles.get(&component_id).map(|h| h.len()).unwrap_or(0)
    }
}

pub struct TaskRegistry;

impl TaskRegistry {
    pub fn register_task(component_id: ComponentId, handle: TaskHandle) {
        REGISTRY.with(|r| r.lock().unwrap().register_task(component_id, handle));
    }

    pub fn register_watcher(component_id: ComponentId, watcher: SignalWatcher) {
        REGISTRY.with(|r| r.lock().unwrap().register_watcher(component_id, watcher));
    }

    pub fn register_interval(component_id: ComponentId, handle: IntervalHandle) {
        REGISTRY.with(|r| r.lock().unwrap().register_interval(component_id, handle));
    }

    pub fn cancel_all(component_id: ComponentId) {
        REGISTRY.with(|r| r.lock().unwrap().cancel_all(component_id));
    }

    pub fn cleanup_completed() {
        REGISTRY.with(|r| r.lock().unwrap().cleanup_completed());
    }

    pub fn task_count(component_id: ComponentId) -> usize {
        REGISTRY.with(|r| r.lock().unwrap().task_count(component_id))
    }
}

pub fn task_registry() -> &'static TaskRegistry {
    static TASK_REGISTRY_SINGLETON: TaskRegistry = TaskRegistry;
    &TASK_REGISTRY_SINGLETON
}
