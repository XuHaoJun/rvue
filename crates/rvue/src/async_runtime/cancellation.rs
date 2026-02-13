//! Simple cancellation using AtomicBool
//!
//! This provides a lightweight cancellation mechanism for async tasks.
//! When the `cancelled` flag is set to true, the task should check and exit.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A cancellation flag that can be checked by async tasks.
#[derive(Clone)]
pub struct Cancellation {
    cancelled: Arc<AtomicBool>,
}

impl Cancellation {
    /// Create a new cancellation flag
    pub fn new() -> Self {
        Self { cancelled: Arc::new(AtomicBool::new(false)) }
    }

    /// Check if cancellation was requested (non-blocking)
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Request cancellation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Reset for reuse (creates new flag)
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}
