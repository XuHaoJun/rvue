# Subscription Management Refactoring Plan: Leptos-Style Implementation

## Overview

This document outlines a comprehensive refactoring of Rvue's reactive signal subscription system to match Leptos's architecture. The goal is to fix memory corruption issues, subscription cleanup problems, and loading behavior issues.

## Problem Statement

### Current Issues

1. **Memory Corruption Crashes**: Stale weak references in subscription lists cause crashes
2. **Loading State Not Working**: After 3-4 refresh clicks, loading state stops triggering
3. **Subscription Cleanup Issues**: Old subscriptions not properly removed when effects re-run
4. **Weak Reference Accumulation**: Invalid weak refs accumulate over time

### Root Cause

The current implementation has fundamental architectural issues:
- Effect stores `(SignalPtr, Weak<Effect>)` instead of proper weak references
- Cleanup uses pointer comparison which may be inaccurate
- Bidirectional cleanup is incomplete

## Current Implementation Analysis

### Rvue Current Structure

```rust
// Signal
SignalDataInner<T> {
    subscribers: GcCell<Vec<Weak<Effect>>>  // Weak refs to effects
}

// Effect  
Effect {
    subscriptions: GcCell<Vec<(SignalPtr, Weak<Effect>)>>  // Problem: stores raw pointers
}
```

### Problems Identified

1. **Storing Raw Pointers**: `SignalPtr` is a raw pointer, not a proper reference
2. **Inaccurate Cleanup**: `Weak::ptr_eq()` comparison may fail
3. **Incomplete Bidirectional Links**: Cleanup only happens one direction

## Leptos Implementation Analysis

### Leptos Structure

```rust
// Signal (Source)
pub struct ArcReadSignal<T> {
    pub(crate) value: Arc<RwLock<T>>,
    pub(crate) inner: Arc<RwLock<SubscriberSet>>,  // Weak<Subscriber>
}

// Effect (Subscriber)
pub struct Effect {
    sources: RwLock<SourceSet>,  // Weak<Source>
}
```

### Key Mechanisms

1. **Bidirectional Links**: Signal→Subscriber AND Subscriber→Source
2. **Proper Weak References**: Uses `Weak<dyn Subscriber>` not raw pointers
3. **Bidirectional Cleanup**: When effect re-runs:
   - Clear sources from effect
   - Remove effect from each source's subscriber set
4. **Upgrade Checks**: Every operation checks if weak ref can be upgraded

## Refactoring Plan

### Phase 1: Foundation - Define Traits and Types

#### 1.1 Create Source and Subscriber Traits

Create `crates/rvue/src/reactivity.rs`:

```rust
use rudo_gc::{Gc, Weak, Trace};

/// Trait for types that can be subscribed to (signals, memos)
pub trait Source: Trace + 'static {
    fn add_subscriber(&self, subscriber: Weak<dyn Subscriber>);
    fn remove_subscriber(&self, subscriber: &Weak<dyn Subscriber>);
    fn notify_subscribers(&self);
}

/// Trait for types that can subscribe to sources (effects, memos)
pub trait Subscriber: Trace + 'static {
    fn mark_dirty(&self);
    fn is_dirty(&self) -> bool;
}
```

#### 1.2 Modify SignalDataInner

**File**: `crates/rvue/src/signal.rs`

Add proper subscription storage:

```rust
pub struct SignalDataInner<T: Trace + Clone + 'static> {
    pub inner: SignalData<T>,
    pub subscribers: GcCell<Vec<Weak<dyn Subscriber>>>,  // Changed: proper weak refs
}
```

### Phase 2: Modify Subscription Logic

#### 2.1 Update subscribe() Function

```rust
pub(crate) fn subscribe(&self, effect: Gc<Effect>) {
    // Check if already subscribed using proper weak ref comparison
    let already_subscribed = {
        let subscribers = self.subscribers.borrow();
        subscribers.iter().any(|sub| {
            // Use proper weak ref comparison
            if let Some(e) = sub.upgrade() {
                Gc::ptr_eq(&e, &effect)
            } else {
                false
            }
        })
    };
    
    if !already_subscribed {
        // Add to signal's subscriber list
        self.subscribers.borrow_mut_gen_only()
            .push(Gc::downgrade(&effect));
        
        // Add signal to effect's sources (bidirectional)
        effect.add_source(Gc::downgrade(self));
    }
}
```

#### 2.2 Update notify_subscribers() Function

```rust
pub(crate) fn notify_subscribers(&self) {
    let subscribers: Vec<_> = {
        let subscribers = self.subscribers.borrow();
        // Collect clones to avoid borrow issues
        subscribers.iter()
            .filter_map(|sub| sub.upgrade())
            .collect()
    };
    
    for subscriber in subscribers {
        subscriber.mark_dirty();
        // Trigger update if dirty
    }
}
```

### Phase 3: Modify Effect Structure and Cleanup

#### 3.1 Update Effect Structure

**File**: `crates/rvue/src/effect.rs`

```rust
pub struct Effect {
    pub owner: Gc<Component>,
    pub closure: Box<dyn FnMut()>,
    pub sources: GcCell<Vec<Weak<dyn Source>>>,  // Changed: stores sources
    pub cleanups: GcCell<Vec<Box<dyn FnOnce()>>>,
    // ... other fields
}
```

#### 3.2 Implement add_source() Method

```rust
impl Effect {
    pub fn add_source(&self, source: Weak<dyn Source>) {
        let mut sources = self.sources.borrow_mut_gen_only();
        // Check if already subscribed
        if !sources.iter().any(|s| {
            if let Some(src) = s.upgrade() {
                // Compare source pointers
                Gc::as_ptr(&src) == Gc::as_ptr(&source)
            } else {
                false
            }
        }) {
            sources.push(source);
        }
    }
}
```

#### 3.3 Update run() for Bidirectional Cleanup

```rust
pub fn run(gc_effect: &Gc<Self>) {
    if gc_effect.is_running.swap(true, Ordering::SeqCst) {
        return;
    }

    // === BIDIRECTIONAL CLEANUP ===
    // 1. Remove this effect from all sources' subscriber lists
    let sources = std::mem::take(&mut *gc_effect.sources.borrow_mut_gen_only());
    for source_weak in sources {
        if let Some(source) = source_weak.upgrade() {
            // Remove this effect from source's subscriber list
            source.remove_subscriber(&Gc::downgrade(gc_effect));
        }
    }
    // ==============================

    // Run cleanups from previous run
    // ... existing code ...

    // Execute closure (will re-subscribe to sources)
    // ... existing code ...
}
```

### Phase 4: Update Dependencies

#### 4.1 Effect Depends on Signal (Signal→Subscriber)

In `SignalDataInner::subscribe()`:
```rust
effect.add_source(Gc::downgrade(self));  // Effect now knows about this signal
```

#### 4.2 Signal Notifies Effect (Source→Subscriber)

In `SignalDataInner::notify_subscribers()`:
```rust
for subscriber in subscribers {
    subscriber.mark_dirty();  // Signal notifies effect
}
```

### Phase 5: Testing

#### 5.1 Unit Tests

- `test_signal_subscription_basic`: Basic subscribe/notify
- `test_effect_cleanup_on_rerun`: Verify cleanup on effect re-run
- `test_bidirectional_links`: Verify both directions work

#### 5.2 Integration Tests

- `test_hackernews_refresh`: Click refresh multiple times
- `test_loading_state`: Verify loading state triggers correctly

#### 5.3 Stress Tests

- Run refresh 100+ times
- Force GC between refreshes
- Verify no crashes

## Implementation Order

1. **Define traits** in new `reactivity.rs` file
2. **Update SignalDataInner** structure
3. **Update subscribe()** function
4. **Update notify_subscribers()** function
5. **Update Effect** structure
6. **Implement add_source()** method
7. **Update run()** cleanup logic
8. **Update tests**
9. **Run hackernews** verification

## Expected Outcomes

### Before Refactoring

- Crashes after 3-4 refresh clicks
- Loading state stops working
- Weak refs accumulate

### After Refactoring

- ✅ No crashes on multiple refresh clicks
- ✅ Loading state always works correctly
- ✅ Proper subscription cleanup
- ✅ Bidirectional links working correctly

## Risk Assessment

### Risks

1. **Breaking Changes**: This is a major refactor that may break existing code
2. **New Bugs**: May introduce new issues during transition
3. **API Changes**: External code using Effect/Signal APIs may need updates

### Mitigation

1. **Comprehensive Tests**: Ensure all existing functionality works
2. **Incremental Changes**: Make small, testable changes
3. **Documentation**: Update docs for any API changes

## Files to Modify

| File | Changes |
|------|---------|
| `crates/rvue/src/reactivity.rs` | New file - define traits |
| `crates/rvue/src/signal.rs` | Update SignalDataInner, subscribe, notify |
| `crates/rvue/src/effect.rs` | Update Effect structure, run cleanup |
| `crates/rvue/src/lib.rs` | Export new traits |
| `crates/rvue/tests/*.rs` | Update/add tests |

## Success Criteria

1. All existing tests pass
2. No crashes on 100+ refresh clicks in hackernews
3. Loading state works correctly on every refresh
4. Memory usage stays stable (no accumulation)

## Timeline

- Phase 1-3: Core implementation (1-2 hours)
- Phase 4: Testing and fixes (1-2 hours)
- Phase 5: Verification (30 minutes)

**Total Estimated Time**: 2.5 - 4.5 hours
