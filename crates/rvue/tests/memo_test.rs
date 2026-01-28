use rvue::{create_memo, create_signal};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_memo_basic() {
    let (read, write) = create_signal(10);
    let count = Rc::new(Cell::new(0));
    let count_clone = count.clone();

    let memo = create_memo(move || {
        count_clone.set(count_clone.get() + 1);
        read.get() * 2
    });

    // Initial value
    // Note: Due to current implementation, it might be 2 if called twice.
    assert_eq!(memo.get(), 20);
    // Let's see how many times it was called.
    // Currently f() is called in create_signal(f()) and then in create_effect(move || write.set(f())).
    // So it should be 2.
    assert_eq!(count.get(), 2);

    write.set(20);
    assert_eq!(memo.get(), 40);
    assert_eq!(count.get(), 3);
}

#[test]
fn test_memo_equality() {
    let (read, write) = create_signal(10);
    let count = Rc::new(Cell::new(0));
    let count_clone = count.clone();

    let memo = rvue::create_memo_with_equality(move || {
        count_clone.set(count_clone.get() + 1);
        read.get() / 5 // Result is same for 10-14, 15-19, etc.
    });

    assert_eq!(memo.get(), 2);
    assert_eq!(count.get(), 2);

    write.set(12);
    assert_eq!(memo.get(), 2);
    assert_eq!(count.get(), 3); // Previous 2 + this decrement run

    // Nested effects shouldn't run if memo didn't change
    let effect_count = Rc::new(Cell::new(0));
    let effect_count_clone = effect_count.clone();
    let memo_clone = memo.clone();
    // Store the effect to prevent it from being dropped
    let _effect = rvue::create_effect(move || {
        memo_clone.get();
        effect_count_clone.set(effect_count_clone.get() + 1);
    });

    assert_eq!(effect_count.get(), 1);

    write.set(14);
    assert_eq!(memo.get(), 2);
    assert_eq!(count.get(), 4);
    assert_eq!(effect_count.get(), 1); // Should NOT have run again

    write.set(15);
    assert_eq!(memo.get(), 3);
    assert_eq!(count.get(), 5);
    assert_eq!(effect_count.get(), 2); // Should have run again
}

#[test]
fn test_get_untracked() {
    let (read, write) = create_signal(10);
    let effect_count = Rc::new(Cell::new(0));
    let effect_count_clone = effect_count.clone();

    rvue::create_effect(move || {
        read.get_untracked();
        effect_count_clone.set(effect_count_clone.get() + 1);
    });

    assert_eq!(effect_count.get(), 1);

    write.set(20);
    assert_eq!(effect_count.get(), 1); // Should not update
}

#[test]
fn test_untracked_block() {
    let (read, write) = create_signal(10);
    let effect_count = Rc::new(Cell::new(0));
    let effect_count_clone = effect_count.clone();

    rvue::create_effect(move || {
        rvue::untracked(|| {
            read.get();
        });
        effect_count_clone.set(effect_count_clone.get() + 1);
    });

    assert_eq!(effect_count.get(), 1);

    write.set(20);
    assert_eq!(effect_count.get(), 1); // Should not update
}

#[test]
fn test_memo_dependency() {
    let (read1, write1) = create_signal(10);
    let (read2, write2) = create_signal(20);

    let memo = create_memo(move || read1.get() + read2.get());

    assert_eq!(memo.get(), 30);

    write1.set(15);
    assert_eq!(memo.get(), 35);

    write2.set(25);
    assert_eq!(memo.get(), 40);
}

#[test]
fn test_memo_nested() {
    let (read, write) = create_signal(10);
    let memo1 = create_memo(move || read.get() + 1);
    let memo2 = create_memo(move || memo1.get() * 2);

    assert_eq!(memo2.get(), 22);

    write.set(20);
    assert_eq!(memo2.get(), 42);
}
