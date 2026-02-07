# Rvue Async/Await Best Practices

> Based on research of Xilem, Leptos, Freya, and rudo-gc

## Overview

Rvue adopts a **hybrid executor model** for async/await:

- **tokio** for background async work (I/O, computation)
- **winit event loop** for UI thread operations
- **Message passing** for async-to-UI communication
- **rudo-gc integration** for GC-safe async operations

---

## rudo-gc Async Support

rudo-gc provides comprehensive tokio integration for garbage-collected objects in async contexts:

### Key Components

| Component | Purpose |
|-----------|---------|
| `GcRootSet` | Process-level singleton tracking GC roots across all tokio tasks |
| `GcRootGuard` | RAII guard - registers root on creation, unregisters on drop |
| `GcTokioExt` trait | Extension trait with `root_guard()` and `yield_now()` methods |
| `AsyncHandleScope` | Manages handles that persist across `.await` points |
| `AsyncHandle<T>` | GC handle usable in async code (Send + Sync) |
| `spawn_with_gc!` macro | Spawns tasks with automatic GC root tracking |

### Usage Patterns

```rust
// Pattern 1: Root Guard (simple case)
let gc = Gc::new(Data { value: 42 });
let _guard = gc.root_guard(); // Register as root

tokio::spawn(async move {
    println!("{}", gc.value); // Safe to access
});

// Pattern 2: AsyncHandleScope (complex case)
let tcb = rudo_gc::heap::current_thread_control_block()?;
let scope = AsyncHandleScope::new(&tcb);
let handle = scope.handle(&gc);

tokio::task::yield_now().await;
println!("{}", unsafe { handle.get().value });

// Pattern 3: spawn_with_gc! macro (convenient)
rudo_gc::spawn_with_gc!(gc => |handle| {
    tokio::task::yield_now().await;
    println!("{}", unsafe { handle.get().value });
}).await;
```

### Cooperative GC Scheduling

```rust
// Yield during long computations to allow GC to run
for i in 0..10000 {
    process_data(i);
    gc.yield_now().await; // Cooperatively schedule GC
}
```

---

## Framework Comparison

| Aspect | Xilem | Leptos | Freya | Rvue (Proposed) |
|--------|-------|--------|-------|-----------------|
| Async Runtime | tokio | any_spawner | custom futures | tokio + winit |
| Task Cancellation | JoinHandle.abort() | Abortable futures | TaskHandle | TaskHandle |
| UI Communication | MessageProxy | Signals | State mutation | dispatch_to_ui |
| GC Integration | manual | owner-based | None | rudo-gc native |
| Lifecycle Binding | View-based | Owner-based | Component-scoped | Component-scoped |

---

## Core Patterns

### Pattern 1: Simple Async Computation

```rust
// For CPU-bound work that shouldn't block the UI
spawn_background_task(async move {
    let result = expensive_computation().await;
    dispatch_to_ui(move || {
        signal.set(result);
    });
});
```

### Pattern 2: Async Resource Loading

```rust
// For loading data from network or disk
spawn_resource(async move {
    let data = fetch_json(url).await?;
    serde_json::from_str(&data)
}, |result| {
    // This closure runs on UI thread
    match result {
        Ok(data) => signal.set(Some(data)),
        Err(e) => error_signal.set(e.to_string()),
    }
});
```

### Pattern 3: Periodic Updates

```rust
// For polling or repeated async operations
let _ = spawn_interval(Duration::from_secs(30), || async {
    let data = fetch_updates().await;
    dispatch_to_ui(move || {
        update_signals(data);
    });
});
```

### Pattern 4: Debounced Async Operations

```rust
// For search input with async suggestions
let (suggestions, set_suggestions) = create_signal(Vec::new());

on_cleanup(cx, || {
    // Cancel any pending debounced search
});

spawn_debounced(Duration::from_millis(300), move || async {
    let results = search_suggestions(query.get()).await;
    dispatch_to_ui(move || {
        set_suggestions.set(results);
    });
});
```

---

## Required API

### dispatch_to_ui()

Queue a closure to run on the main thread (UI thread):

```rust
/// Queue a closure to run on the main thread (UI thread)
pub fn dispatch_to_ui<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    // Store f in an AppState field
    // Execute during run_update_passes()
}
```

### spawn_task()

Spawn an async task with GC-aware root tracking:

```rust
/// Spawn an async task with GC-aware root tracking
pub fn spawn_task<F>(task: F) -> TaskHandle
where
    F: FnOnce(TaskContext) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
{
    // Implementation using rudo-gc's spawn_with_gc! pattern
}
```

### TaskHandle

Handle for managing spawned tasks:

```rust
pub struct TaskHandle {
    abort_handle: AbortHandle,
}

impl TaskHandle {
    pub fn abort(&self) {
        self.abort_handle.abort();
    }
}
```

---

## Example: Complete Async Component

```rust
use rvue::prelude::*;

#[component]
fn UserProfile(cx: Scope) -> View {
    let user_id = use_context::<UserId>(cx);
    let (user, set_user) = create_signal::<Option<User>>(None);
    let (posts, set_posts) = create_signal::<Vec<Post>>(Vec::new());

    // Load user data on mount
    spawn_task(move |_ctx| {
        Box::pin(async move {
            // Fetch user
            if let Ok(user_data) = fetch_user(user_id.0).await {
                dispatch_to_ui(move || {
                    set_user.set(Some(user_data));
                });
            }

            // Fetch posts (can run concurrently)
            if let Ok(user_posts) = fetch_posts(user_id.0).await {
                dispatch_to_ui(move || {
                    set_posts.set(user_posts);
                });
            }
        })
    });

    view! {
        <Flex direction=Column padding=20>
            if let Some(user) = user.get().as_ref() {
                <Text value=user.name.clone() />
                <Text value=user.bio.clone() />
            }
            <For each=posts>
                |post| view! {
                    <PostCard post=post.clone() />
                }
            </For>
        </Flex>
    }
}
```

---

## Error Handling with Retries

```rust
async fn fetch_with_retry<T, F>(
    mut fetch_fn: F,
    max_retries: u32,
    backoff: Duration
) -> Result<T, String>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, String>>>>,
{
    let mut last_error = None;

    for attempt in 0..max_retries {
        match fetch_fn().await {
            Ok(data) => return Ok(data),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    tokio::time::sleep(backoff * (attempt + 1)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "Unknown error".to_string()))
}

// Usage
spawn_task(move |_ctx| {
    Box::pin(async move {
        let result = fetch_with_retry(
            || Box::pin(fetch_data()),
            3,
            Duration::from_secs(1)
        ).await;

        dispatch_to_ui(move || {
            match result {
                Ok(data) => signal.set(data),
                Err(e) => error_signal.set(e),
            }
        });
    })
});
```

---

## Progress Updates

```rust
fn download_file(
    url: String,
    on_progress: impl Fn(u64, u64) + Send + 'static,
) -> impl Future<Output = Result<Vec<u8>, String>> {
    async move {
        let response = reqwest::get(&url).await?;
        let total = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut bytes = Vec::new();

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            bytes.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            on_progress(downloaded, total);
        }

        Ok(bytes)
    }
}

// Usage with progress signal
spawn_task(move |_ctx| {
    Box::pin(async move {
        let progress = progress_signal.clone();
        let result = download_file(url, move |curr, total| {
            dispatch_to_ui(move || {
                progress.set((curr, total));
            });
        }).await;

        dispatch_to_ui(move || {
            match result {
                Ok(bytes) => { /* handle success */ }
                Err(e) => { /* handle error */ }
            }
        });
    })
});
```

---

## Concerns and Mitigations

### Thread Safety

| Concern | Mitigation |
|---------|------------|
| GC objects across threads | Use `GcRootGuard` or `AsyncHandleScope` |
| Signal access from async | Always dispatch to UI thread |
| Component lifecycle | Cancel tasks on component unmount |

### Memory Management

| Concern | Mitigation |
|---------|------------|
| GC during async | Use `gc.yield_now().await` in long loops |
| Leaked tasks | `TaskHandle` with `abort()` on cleanup |
| Memory growth | Tokio's own limits + rudo-gc incremental marking |

### Ordering Issues

| Concern | Mitigation |
|---------|------------|
| Race conditions | Dispatch all UI updates via `dispatch_to_ui` |
| Stale data | Include version counters in messages |
| Out-of-order updates | Use `SeqCst` ordering for signals |

### Error Handling

| Concern | Mitigation |
|---------|------------|
| Panics in tasks | Catch unwinds in tokio task |
| Propagating errors | Error signals + dispatch_to_ui |
| Task abort during cleanup | Check cancellation token |

### Performance

| Concern | Mitigation |
|---------|------------|
| GC pauses | Yield during large operations |
| Channel contention | Batched dispatches |
| Task overhead | Tokio's work-stealing scheduler |

---

## Implementation Roadmap

### Phase 1: Basic Infrastructure
1. Add tokio dependency to rvue
2. Implement `dispatch_to_ui` queue in AppState
3. Create basic `spawn_task` using rudo-gc patterns

### Phase 2: Task Management
1. Implement `TaskHandle` with abort support
2. Add lifecycle binding (cancel on unmount)
3. Create `on_cleanup` task cancellation

### Phase 3: Higher-Level APIs
1. Implement `spawn_resource` for data loading
2. Add `spawn_debounced` and `spawn_throttled`
3. Create progress notification helpers

### Phase 4: Documentation
1. Write async best practices guide
2. Add examples to the examples directory
3. Document migration patterns from sync code

---

## References

- [Xilem Masonry](https://github.com/linebender/xilem/tree/main/masonry)
- [Leptos Framework](https://github.com/leptos-rs/leptos)
- [Freya Framework](https://github.com/marc2332/freya)
- [rudo-gc](https://github.com/xuhaojun/rudo)
