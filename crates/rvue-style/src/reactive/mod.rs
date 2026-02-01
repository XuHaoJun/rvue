//! Reactive styling system for rvue
//!
//! This module provides reactive styling capabilities that integrate with
//! rvue's signal system for fine-grained reactivity.

pub mod style_signal;

pub use style_signal::{
    create_reactive_signal, create_style_effect, on_style_cleanup, ReactiveProperty,
    ReactiveReadSignal, ReactiveSignal, ReactiveSignalWrite, ReactiveStyles,
};
