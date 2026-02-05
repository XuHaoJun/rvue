// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Event recording for tracking widget lifecycle events.

use rudo_gc::Gc;
use rvue::component::Component;
use std::collections::HashMap;

/// A recorded event from the widget lifecycle.
#[derive(Debug, Clone)]
pub enum RecordedEvent {
    /// Pointer event was sent to the widget
    PointerEvent(PointerRecord),
    /// Widget received focus
    FocusReceived,
    /// Widget lost focus
    FocusLost,
    /// Widget was hovered
    HoverStarted,
    /// Widget hover ended
    HoverEnded,
    /// Widget was clicked
    Clicked,
    /// Widget scroll offset changed
    ScrollChanged { old_offset: (f64, f64), new_offset: (f64, f64) },
    /// Widget visibility changed
    VisibilityChanged { visible: bool },
    /// Widget was laid out
    LayoutPerformed,
    /// Widget was rendered
    Rendered,
    /// Custom event
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerRecord {
    pub event_type: PointerEventType,
    pub position: (f64, f64),
    pub button: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerEventType {
    Move,
    Down,
    Up,
    Scroll,
    Enter,
    Leave,
}

/// A recorder that tracks events for a specific widget.
#[derive(Default)]
pub struct WidgetRecorder {
    events: Vec<RecordedEvent>,
}

impl WidgetRecorder {
    pub fn record(&mut self, event: RecordedEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[RecordedEvent] {
        &self.events
    }

    pub fn take(&mut self) -> Vec<RecordedEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn has_event(&self, predicate: impl Fn(&RecordedEvent) -> bool) -> bool {
        self.events.iter().any(predicate)
    }
}

/// Event recorder for tracking widget lifecycle events across the widget tree.
#[derive(Default)]
pub struct EventRecorder {
    widget_records: HashMap<u32, WidgetRecorder>,
}

impl EventRecorder {
    /// Record an event for a specific widget.
    pub fn record(&mut self, widget_id: u32, event: RecordedEvent) {
        self.widget_records.entry(widget_id).or_default().record(event);
    }

    /// Get all recorded events for a widget.
    pub fn get_records(&self, widget_id: u32) -> &[RecordedEvent] {
        self.widget_records.get(&widget_id).map(|r| r.events()).unwrap_or(&[])
    }

    /// Take (and clear) all recorded events for a widget.
    pub fn take_records(&mut self, widget_id: u32) -> Vec<RecordedEvent> {
        self.widget_records.entry(widget_id).or_default().take()
    }

    /// Check if a widget has any events matching the predicate.
    pub fn has_event(&self, widget_id: u32, predicate: impl Fn(&RecordedEvent) -> bool) -> bool {
        self.widget_records.get(&widget_id).map(|r| r.has_event(&predicate)).unwrap_or(false)
    }

    /// Get all widget IDs that have recorded events.
    pub fn recorded_widget_ids(&self) -> Vec<u32> {
        self.widget_records.keys().cloned().collect()
    }

    /// Clear all recorded events.
    pub fn clear(&mut self) {
        self.widget_records.clear();
    }
}

/// Helper trait for recording events.
#[allow(dead_code)]
pub trait RecordEvent {
    fn record(&mut self, _event: RecordedEvent);
}

impl RecordEvent for Gc<Component> {
    fn record(&mut self, _event: RecordedEvent) {
        // This would be called from within the widget's event handlers
        // The actual recording happens in the harness
    }
}
