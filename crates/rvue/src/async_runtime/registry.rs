use std::collections::HashMap;
use std::sync::Mutex;

use crate::ComponentId;

use super::task::TaskHandle;

thread_local! {
    pub static REGISTRY: Mutex<TaskRegistryInner> = Mutex::new(TaskRegistryInner::new());
}

pub struct TaskRegistryInner {
    tasks: HashMap<ComponentId, Vec<TaskHandle>>,
}

impl TaskRegistryInner {
    fn new() -> Self {
        Self { tasks: HashMap::new() }
    }

    pub fn register(&mut self, component_id: ComponentId, handle: TaskHandle) {
        self.tasks.entry(component_id).or_default().push(handle);
    }

    pub fn cancel_all(&mut self, component_id: ComponentId) {
        if let Some(handles) = self.tasks.remove(&component_id) {
            for handle in handles {
                handle.abort();
            }
        }
    }

    pub fn cleanup_completed(&mut self) {
        for handles in self.tasks.values_mut() {
            handles.retain(|h| h.is_running());
        }
    }

    pub fn task_count(&self, component_id: ComponentId) -> usize {
        self.tasks.get(&component_id).map(|h| h.len()).unwrap_or(0)
    }
}

pub struct TaskRegistry;

impl TaskRegistry {
    pub fn register(component_id: ComponentId, handle: TaskHandle) {
        REGISTRY.with(|r| r.lock().unwrap().register(component_id, handle));
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
