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

## 8. GC-Safe Async (Advanced)

If you need to access `Gc<T>` objects across `.await` points:

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
    spawn_with_gc!(data => |handle| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let value = unsafe { handle.get().value };
        dispatch_to_ui(move || { /* update UI */ });
    });
}
```

**Prefer Option A** (clone before spawn) in most cases. Option B is only needed when you need to read GC data *after* an `.await` point.

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
