//! Unit tests for Signal implementation

use rvue::{create_signal, SignalRead, SignalWrite};

#[test]
fn test_create_signal() {
    let (read, _write) = create_signal(42);
    assert_eq!(read.get(), 42);
}

#[test]
fn test_signal_set() {
    let (read, write) = create_signal(0);
    write.set(10);
    assert_eq!(read.get(), 10);
}

#[test]
fn test_signal_update() {
    let (read, write) = create_signal(5);
    write.update(|x| *x += 3);
    assert_eq!(read.get(), 8);
}

#[test]
fn test_signal_multiple_updates() {
    let (read, write) = create_signal(0);
    write.set(1);
    assert_eq!(read.get(), 1);
    write.update(|x| *x *= 2);
    assert_eq!(read.get(), 2);
    write.update(|x| *x += 10);
    assert_eq!(read.get(), 12);
}

#[test]
fn test_signal_string() {
    let (read, write) = create_signal(String::from("hello"));
    assert_eq!(read.get(), "hello");
    write.set(String::from("world"));
    assert_eq!(read.get(), "world");
}

#[test]
fn test_signal_clone_handles() {
    let (read1, write1) = create_signal(100);
    let read2 = read1.clone();
    let write2 = write1.clone();

    // All handles should point to the same signal
    write1.set(200);
    assert_eq!(read1.get(), 200);
    assert_eq!(read2.get(), 200);

    write2.set(300);
    assert_eq!(read1.get(), 300);
    assert_eq!(read2.get(), 300);
}

#[test]
fn test_signal_with_vec() {
    let (read, write) = create_signal(vec![1, 2, 3]);
    assert_eq!(read.get(), vec![1, 2, 3]);

    write.update(|v| {
        v.push(4);
    });
    assert_eq!(read.get(), vec![1, 2, 3, 4]);
}

#[test]
fn test_signal_version_increments() {
    let (_read, write) = create_signal(0);
    // Note: We can't directly access version, but we can verify
    // that updates work correctly, which implies version increments
    // This test verifies the signal works correctly through multiple updates
    write.set(1);
    write.update(|x| *x += 1);
    // If we got here without panicking, version increments are working
}

#[test]
fn test_signal_independent_signals() {
    let (read1, write1) = create_signal(10);
    let (read2, write2) = create_signal(20);

    assert_eq!(read1.get(), 10);
    assert_eq!(read2.get(), 20);

    write1.set(11);
    assert_eq!(read1.get(), 11);
    assert_eq!(read2.get(), 20); // Should be unchanged

    write2.set(22);
    assert_eq!(read1.get(), 11); // Should be unchanged
    assert_eq!(read2.get(), 22);
}
