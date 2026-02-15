use rudo_gc::Gc;
use rudo_gc::Trace;

#[derive(Clone)]
struct TestItem {
    id: u32,
}

unsafe impl Trace for TestItem {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

#[test]
fn test_gc_vec_corruption() {
    let mut items: Vec<Gc<TestItem>> = Vec::with_capacity(10);

    for i in 0..10 {
        let item = Gc::new(TestItem { id: i as u32 });
        items.push(Gc::clone(&item));
        eprintln!("Pushed item {} at index {}", item.id, i);
        // Force GC after each push to simulate the For widget behavior
        if i == 2 {
            eprintln!("Forcing GC at i=2");
            rudo_gc::collect();
            eprintln!("After GC at i=2:");
            for (idx, item) in items.iter().enumerate() {
                eprintln!("items[{}] = {}", idx, item.id);
            }
        }
    }

    eprintln!("Before GC:");
    for (idx, item) in items.iter().enumerate() {
        eprintln!("items[{}] = {}", idx, item.id);
    }

    eprintln!("Calling rudo_gc::collect()...");
    rudo_gc::collect();
    eprintln!("After GC:");
    for (idx, item) in items.iter().enumerate() {
        eprintln!("items[{}] = {}", idx, item.id);
        assert_eq!(item.id, idx as u32, "Corruption at index {}", idx);
    }
}
