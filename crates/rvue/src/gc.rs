//! Re-export of rudo-gc's impl_gc_capture macro for types that don't contain Gc pointers.
//!
//! # Usage
//!
//! ```rust
//! use rudo_gc::Trace;
//! use rvue::impl_gc_capture;
//!
//! #[derive(Clone)]
//! struct MyItem {
//!     id: i64,
//!     name: String,
//! }
//!
//! unsafe impl rudo_gc::Trace for MyItem {
//!     fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
//! }
//!
//! impl_gc_capture!(MyItem);
//! ```

#[macro_export]
macro_rules! impl_gc_capture {
    ($t:ty) => {
        impl ::rudo_gc::cell::GcCapture for $t {
            #[inline]
            fn capture_gc_ptrs(&self) -> &[::std::ptr::NonNull<::rudo_gc::GcBox<()>>] {
                &[]
            }

            #[inline]
            fn capture_gc_ptrs_into(
                &self,
                _ptrs: &mut Vec<::std::ptr::NonNull<::rudo_gc::GcBox<()>>>,
            ) {
            }
        }
    };
}
