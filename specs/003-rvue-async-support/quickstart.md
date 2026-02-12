# Quickstart: Rvue Async Runtime

**Branch**: `003-rvue-async-support` | **Date**: 2026-02-07

This guide gets you started with async operations in Rvue in under 5 minutes.

---

## 1. Enable the Feature

Add the `async` feature to your dependency:

```toml
# In your app's Cargo.toml
[dependencies]
rvue = { path = "../../crates/rvue", features = ["async"] }
```

---

## 2. Basic Pattern: Fetch Data Without Blocking

The most common pattern is spawning an async task to fetch data and updating a signal when done.

```rust
use rvue::prelude::*;
use rvue::async_runtime::{spawn_task, dispatch_to_ui};
use std::time::Duration;

#[component]
fn UserProfile() -> View {
    let (user_name, set_user_name) = create_signal(String::from("Loading..."));

    // Spawn an async task — does NOT block the UI
    spawn_task(async move {
        // Simulate a network request
        tokio::time::sleep(Duration::from_secs(1)).await;
        let name = String::from("Alice");

        // Send result back to UI thread
        dispatch_to_ui(move || {
            set_user_name(name);
        });
    });

    view! {
        <Text value=user_name.get() />
    }
}
```

**Key points**:
- `spawn_task()` runs the future on a background thread
- `dispatch_to_ui()` safely delivers the result to the UI thread
- The UI remains responsive during the async work
- The task is automatically cancelled when the component unmounts

---

## 3. Simpler Pattern: SignalSender

Use `SignalSender` to skip the manual `dispatch_to_ui` call:

```rust
use rvue::prelude::*;
use rvue::async_runtime::spawn_task;

#[component]
fn Counter() -> View {
    let (count, set_count) = create_signal(0i32);

    // Create a thread-safe sender
    let sender = set_count.sender();

    spawn_task(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        sender.set(42); // Automatically dispatches to UI thread
    });

    view! {
        <Text value=format!("Count: {}", count.get()) />
    }
}
```

---

## 4. Reactive Resources

For data fetching with loading/error states, use `create_resource`:

```rust
use rvue::prelude::*;
use rvue::async_runtime::{create_resource, ResourceState};

async fn fetch_user(id: i32) -> Result<String, String> {
    // Real app: reqwest::get(...).await
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(format!("User #{}", id))
}

#[component]
fn UserCard(user_id: i32) -> View {
    let resource = create_resource(
        move || user_id,
        |id| fetch_user(id),
    );

    view! {
        match resource.get() {
            ResourceState::Loading => view! { <Text value="Loading..." /> },
            ResourceState::Ready(name) => view! { <Text value=name /> },
            ResourceState::Error(err) => view! { <Text value=format!("Error: {}", err) /> },
            ResourceState::Pending => view! { <Text value="" /> },
        }
    }
}
```

---

## 5. Debounced Search

For search-as-you-type with rate limiting:

```rust
use rvue::prelude::*;
use rvue::async_runtime::{spawn_debounced, dispatch_to_ui};
use std::time::Duration;

#[component]
fn SearchBox() -> View {
    let (query, set_query) = create_signal(String::new());
    let (results, set_results) = create_signal(Vec::<String>::new());

    let results_sender = set_results.sender();

    // Only search after 300ms of no typing
    let search = spawn_debounced(Duration::from_millis(300), move |q: String| {
        let sender = results_sender.clone();
        async move {
            let data = search_api(&q).await;
            sender.set(data);
        }
    });

    // Wire up input → debounced search
    create_effect(move || {
        let q = query.get();
        if !q.is_empty() {
            search.call(q);
        }
    });

    view! {
        <Input value=query on_change=set_query />
        <For each=results.get() key=|r| r.clone()>
            |result| view! { <Text value=result /> }
        </For>
    }
}
```

---

## 6. Periodic Polling

For live data that refreshes automatically:

```rust
use rvue::prelude::*;
use rvue::async_runtime::spawn_interval;
use std::time::Duration;

#[component]
fn LiveStatus() -> View {
    let (status, set_status) = create_signal(String::from("Checking..."));
    let sender = set_status.sender();

    let handle = spawn_interval(Duration::from_secs(10), move || {
        let sender = sender.clone();
        async move {
            let s = check_health().await;
            sender.set(s);
        }
    });

    // Cancel polling when component unmounts
    on_cleanup(move || handle.abort());

    view! {
        <Text value=status.get() />
    }
}
```

---

## 7. Manual Task Management

For full control over task lifecycle:

```rust
use rvue::prelude::*;
use rvue::async_runtime::{spawn_task, TaskHandle};

#[component]
fn FileProcessor() -> View {
    let (progress, set_progress) = create_signal(0u32);
    let (handle, set_handle) = create_signal(None::<TaskHandle>);
    let sender = set_progress.sender();

    let start = move || {
        let sender = sender.clone();
        let h = spawn_task(async move {
            for i in 0..=100 {
                tokio::time::sleep(Duration::from_millis(50)).await;
                sender.set(i);
            }
        });
        set_handle(Some(h));
    };

    let cancel = move || {
        if let Some(h) = handle.get() {
            h.abort();
            set_handle(None);
        }
    };

    view! {
        <Text value=format!("Progress: {}%", progress.get()) />
        <Button on_click=start>
            <Text value="Start" />
        </Button>
        <Button on_click=cancel>
            <Text value="Cancel" />
        </Button>
    }
}
```

---

## 8. GC Safety (Required Reading)

Rvue uses rudo-gc for automatic memory management. When using async operations, you MUST understand these rules to avoid memory corruption.

### The Problem

```rust
// DANGEROUS: Closure captures Gc<T>, may be collected between awaits
let gc_data = Gc::new(MyData { value: 42 });
spawn_interval(Duration::from_secs(1), move || {
    // At this await point, GC may collect gc_data
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("{}", gc_data.value);  // Use-after-free!
});
```

This happens because:
1. `Gc<T>` is `!Send + !Sync` and cannot cross thread boundaries without explicit handling
2. `Gc<T>` is not automatically registered as a GC root in async tasks
3. GC runs at await points and may collect "unreachable" objects

### The Solution: Clone Before Spawn

```rust
// SAFE: Clone the value first
let gc_data = Gc::new(MyData { value: 42 });
let value_clone = gc_data.value.clone();  // Extract clone

spawn_interval(Duration::from_secs(1), move || {
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("{}", value_clone);  // Safe: value_clone is owned value
});
```

---

### Pattern 1: Extract Primitive Values (Simplest)

For simple types like `i32`, `String`, extract the value before spawning:

```rust
let count = create_signal(0i32);
let current_count = *count.get();  // Extract i32

spawn_interval(Duration::from_secs(1), move || {
    println!("Count: {}", current_count);  // Safe: current_count is i32
});
```

---

### Pattern 2: Clone Gc<T> Before Spawn

When you need `Gc<T>` features (e.g., cloning):

```rust
let gc_data = Gc::new(MyData { value: 42 });
let gc_clone = gc_data.clone();  // Must clone before spawn

spawn_interval(Duration::from_secs(1), move || {
    let value = gc_clone.value.clone();  // Extract value
    println!("{}", value);
});
```

---

### Pattern 3: watch_signal (Recommended for Signals)

For watching signal values at intervals, use the built-in helper:

```rust
use rvue::prelude::*;
use rvue::async_runtime::watch_signal;
use std::time::Duration;

#[component]
fn LiveCounter() -> View {
    let (count, set_count) = create_signal(0i32);

    // Watch signal, automatically dispatch to UI
    let watcher = watch_signal(
        count,
        Duration::from_millis(100),
        |current| {
            println!("Count: {}", current);
            None  // Return Some(value) to update, None to just watch
        }
    );

    on_cleanup(move || watcher.stop());

    // Changing count triggers the watcher
    set_count(42);

    view! {
        <Text value=format!("Count: {}", count.get()) />
    }
}
```

**Panic Handling**:

```rust
#[component]
fn RobustWatcher() -> View {
    let (count, set_count) = create_signal(0i32);

    let mut watcher = watch_signal(
        count,
        Duration::from_millis(100),
        |current| {
            // This callback is resilient to panics
            Some(current * 2)
        }
    );

    // Set a panic handler for observability
    watcher.set_on_panic(|| {
        log::error!("Watch callback panicked!");
    });

    on_cleanup(move || watcher.stop());

    view! { ... }
}
```

**Behavior**:
- Callback panics are caught and logged
- `watcher.panic_count()` returns the number of panics
- Watcher continues running after panic (doesn't abort)

---

### Pattern 4: Advanced - spawn_with_gc! (Direct Gc<T> Access)

For cases where you MUST access `Gc<T>` after an await, use rudo-gc's `spawn_with_gc!` macro:

```rust
use rudo_gc::{Gc, spawn_with_gc};
use rvue::async_runtime::dispatch_to_ui;

fn process_gc_data(data: Gc<MyData>) {
    // Option A: Clone data before spawning (preferred)
    let value = data.value.clone();
    spawn_task(async move {
        let result = process(value).await;
        dispatch_to_ui(move || { /* update UI */ });
    });

    // Option B: Use spawn_with_gc! for GC access across await
    // Note: data is MOVED into the macro
    spawn_with_gc!(data => |handle| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        // SAFE: handle keeps Gc<T> alive
        let value = unsafe { handle.get().value };
        dispatch_to_ui(move || { /* update UI */ });
    });
}
```

**Important**: The `Gc<T>` is **MOVED** into `spawn_with_gc!`. The original `data` is no longer usable. Clone first if you need to keep it.

---

### Summary Table

| Pattern | When to Use | Example |
|---------|-------------|---------|
| Extract value | Simple types (i32, String) | `let v = *signal.get();` |
| Clone Gc<T> | Need Gc<T> features | `let gc = gc_data.clone();` |
| watch_signal | Watching signals | `watch_signal(signal, period, cb)` |
| watch_signal + panic_handler | Need panic recovery | `watcher.set_on_panic(cb)` |
| spawn_with_gc! | Must access Gc<T> after await | `spawn_with_gc!(gc => \|h\| {...})` |

---

### Common Mistakes

```rust
// WRONG: Captures Gc<T>
spawn_interval(Duration::from_secs(1), move || {
    let value = count.get();  // count is Gc<T>
});

// RIGHT: Extract before spawn
let current = *count.get();
spawn_interval(Duration::from_secs(1), move || {
    println!("{}", current);
});

// WRONG: Gc<T> moved without clone
let gc = Gc::new(Data { value: 1 });
spawn_with_gc!(gc => |handle| { /* ... */ });
println!("{}", gc.value);  // ERROR: gc was moved!

// RIGHT: Clone before spawn_with_gc!
let gc = Gc::new(Data { value: 1 });
let gc_clone = gc.clone();
spawn_with_gc!(gc => |handle| { /* ... */ });
println!("{}", gc_clone.value);  // OK: gc_clone is separate
```

---

### Why This Matters

GC safety is not optional. Without proper handling:
- `Gc<T>` objects may be collected while still in use
- This causes **Use-After-Free** bugs (memory corruption)
- The bug may appear intermittently, making it hard to debug
- Data races can occur between GC and your code

**Always clone before spawning if your closure captures any Gc<T>-based type.**

---

## Common Mistakes

### Don't access signals directly in async tasks

```rust
// BAD: Signals are !Send — this won't compile
spawn_task(async move {
    let value = count.get(); // ❌ ReadSignal is !Send
});

// GOOD: Read the value before spawning
let current = count.get();
spawn_task(async move {
    println!("Value was: {}", current); // ✅ i32 is Send
});

// GOOD: Use a sender for writes
let sender = set_count.sender();
spawn_task(async move {
    sender.set(42); // ✅ SignalSender is Send
});
```

### Don't forget cleanup for intervals

```rust
// BAD: Interval runs forever, even after unmount
spawn_interval(Duration::from_secs(1), || async { /* ... */ });

// GOOD: Cancel on cleanup
let handle = spawn_interval(Duration::from_secs(1), || async { /* ... */ });
on_cleanup(move || handle.abort());
```

Note: `spawn_task()` tasks are automatically cancelled on unmount (via `TaskRegistry`). `spawn_interval` and `spawn_debounced` should use explicit cleanup for clarity.

---

## Next Steps

- See [API Contract](./contracts/async-api.md) for the full API reference
- See [Data Model](./data-model.md) for entity relationships
- See [Research](./research.md) for design decisions and rationale
