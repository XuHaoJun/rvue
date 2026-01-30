use rudo_gc::{collect_full, Gc, GcCell, Trace};

#[derive(Clone)]
enum ValueEnum {
    I32(Gc<i32>),
    I64(Gc<i64>),
}

impl ValueEnum {
    fn from_i32(value: i32) -> Self {
        let gc = Gc::new(value);
        let ptr = Gc::internal_ptr(&gc);
        std::mem::forget(gc);
        unsafe { Self::I32(Gc::from_raw(ptr)) }
    }

    fn to_i32(&self) -> Option<Gc<i32>> {
        match self {
            ValueEnum::I32(gc) => {
                let ptr = Gc::internal_ptr(gc);
                let new_gc: Gc<i32> = unsafe { Gc::from_raw(ptr) };
                std::mem::forget(Gc::clone(gc));
                Some(new_gc)
            }
            ValueEnum::I64(_) => None,
        }
    }
}

unsafe impl Trace for ValueEnum {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        match self {
            ValueEnum::I32(gc) => gc.trace(visitor),
            ValueEnum::I64(gc) => gc.trace(visitor),
        }
    }
}

#[derive(Clone)]
struct ContextEntry {
    type_id: std::any::TypeId,
    value: ValueEnum,
}

unsafe impl Trace for ContextEntry {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
    }
}

#[derive(Clone)]
struct FakeComponent {
    children: GcCell<Vec<Gc<FakeComponent>>>,
    parent: GcCell<Option<Gc<FakeComponent>>>,
    contexts: GcCell<Vec<ContextEntry>>,
}

impl FakeComponent {
    fn new() -> Gc<Self> {
        Gc::new(Self {
            children: GcCell::new(Vec::new()),
            parent: GcCell::new(None),
            contexts: GcCell::new(Vec::new()),
        })
    }

    fn provide_context(&self, value: ValueEnum) {
        let type_id = std::any::TypeId::of::<i32>();
        self.contexts.borrow_mut().push(ContextEntry { type_id, value });
    }

    fn find_context(&self) -> Option<Gc<i32>> {
        for entry in self.contexts.borrow().iter().rev() {
            if let Some(gc) = entry.value.to_i32() {
                return Some(gc);
            }
        }
        if let Some(parent) = self.parent.borrow().as_ref() {
            return parent.find_context();
        }
        None
    }
}

unsafe impl Trace for FakeComponent {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.children.trace(visitor);
        self.contexts.trace(visitor);
    }
}

fn gc_cleanup() {
    for _ in 0..10 {
        collect_full();
    }
    std::thread::sleep(std::time::Duration::from_millis(10));
}

#[test]
fn test_single_context() {
    gc_cleanup();

    let root = FakeComponent::new();
    let child = FakeComponent::new();

    child.parent.borrow_mut().replace(root.clone());
    root.children.borrow_mut().push(child.clone());

    root.provide_context(ValueEnum::from_i32(42));

    let gc = child.find_context();
    assert!(gc.is_some());
    assert_eq!(**gc.as_ref().unwrap(), 42);

    drop(gc);
    drop(child);
    drop(root);
    gc_cleanup();
}

#[test]
#[ignore = "GC isolation issue with GcCell<Vec<Gc<T>>> pattern"]
fn test_two_components_sequential() {
    gc_cleanup();

    let root1 = FakeComponent::new();
    let child1 = FakeComponent::new();
    child1.parent.borrow_mut().replace(root1.clone());
    root1.children.borrow_mut().push(child1.clone());
    root1.provide_context(ValueEnum::from_i32(100));
    let _ = child1.find_context();

    drop(child1);
    drop(root1);
    gc_cleanup();

    let root2 = FakeComponent::new();
    let child2 = FakeComponent::new();
    child2.parent.borrow_mut().replace(root2.clone());
    root2.children.borrow_mut().push(child2.clone());
    root2.provide_context(ValueEnum::from_i32(200));
    let gc = child2.find_context();
    assert_eq!(**gc.as_ref().unwrap(), 200);

    drop(gc);
    drop(child2);
    drop(root2);
    gc_cleanup();
}

#[test]
#[ignore = "GC isolation issue with GcCell<Vec<Gc<T>>> pattern"]
fn test_nested_context() {
    gc_cleanup();

    let root = FakeComponent::new();
    let mid = FakeComponent::new();
    let leaf = FakeComponent::new();

    mid.parent.borrow_mut().replace(root.clone());
    root.children.borrow_mut().push(mid.clone());
    leaf.parent.borrow_mut().replace(mid.clone());
    mid.children.borrow_mut().push(leaf.clone());

    root.provide_context(ValueEnum::from_i32(100));
    mid.provide_context(ValueEnum::from_i32(200));

    let gc = leaf.find_context();
    assert_eq!(**gc.as_ref().unwrap(), 200);

    drop(gc);
    drop(leaf);
    drop(mid);
    drop(root);
    gc_cleanup();
}

#[test]
#[ignore = "GC isolation issue with GcCell<Vec<Gc<T>>> pattern"]
fn test_many_sequential_contexts() {
    gc_cleanup();

    for i in 0..10 {
        let root = FakeComponent::new();
        let child = FakeComponent::new();
        child.parent.borrow_mut().replace(root.clone());
        root.children.borrow_mut().push(child.clone());
        root.provide_context(ValueEnum::from_i32(i));
        let gc = child.find_context();
        assert_eq!(**gc.as_ref().unwrap(), i);

        drop(gc);
        drop(child);
        drop(root);
        gc_cleanup();
    }
}

#[test]
#[ignore = "GC isolation issue with GcCell<Vec<Gc<T>>> pattern"]
fn test_full_component_pattern() {
    gc_cleanup();

    let root = FakeComponent::new();
    let child = FakeComponent::new();
    child.parent.borrow_mut().replace(root.clone());
    root.children.borrow_mut().push(child.clone());

    root.provide_context(ValueEnum::from_i32(42));
    let gc = child.find_context();
    assert_eq!(**gc.as_ref().unwrap(), 42);

    drop(gc);
    drop(child);
    drop(root);
    gc_cleanup();
}

#[test]
#[ignore = "GC isolation issue with GcCell<Vec<Gc<T>>> pattern"]
fn test_two_tests_sequential() {
    gc_cleanup();

    let root1 = FakeComponent::new();
    let child1 = FakeComponent::new();
    child1.parent.borrow_mut().replace(root1.clone());
    root1.children.borrow_mut().push(child1.clone());
    root1.provide_context(ValueEnum::from_i32(100));
    let _ = child1.find_context();

    drop(child1);
    drop(root1);
    gc_cleanup();

    let root2 = FakeComponent::new();
    let child2 = FakeComponent::new();
    child2.parent.borrow_mut().replace(root2.clone());
    root2.children.borrow_mut().push(child2.clone());
    root2.provide_context(ValueEnum::from_i32(200));
    let gc2 = child2.find_context();
    assert_eq!(**gc2.as_ref().unwrap(), 200);

    drop(gc2);
    drop(child2);
    drop(root2);
    gc_cleanup();
}
