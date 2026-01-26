//! Event descriptors for Leptos-style event handling
//!
//! Use with `component.on(ev::click, |e| { ... })`

use crate::event::EventDescriptor;

pub struct Click;
impl EventDescriptor for Click {
    type EventType = crate::event::types::PointerButtonEvent;
}

pub struct Input;
impl EventDescriptor for Input {
    type EventType = crate::event::status::InputEvent;
}

pub struct Change;
impl EventDescriptor for Change {
    type EventType = crate::event::status::InputEvent;
}

pub struct PointerDown;
impl EventDescriptor for PointerDown {
    type EventType = crate::event::types::PointerButtonEvent;
}

pub struct PointerUp;
impl EventDescriptor for PointerUp {
    type EventType = crate::event::types::PointerButtonEvent;
}

pub struct PointerMove;
impl EventDescriptor for PointerMove {
    type EventType = crate::event::types::PointerMoveEvent;
}

pub struct KeyDown;
impl EventDescriptor for KeyDown {
    type EventType = crate::event::types::KeyboardEvent;
}

pub struct KeyUp;
impl EventDescriptor for KeyUp {
    type EventType = crate::event::types::KeyboardEvent;
}

pub struct Focus;
impl EventDescriptor for Focus {
    type EventType = crate::event::status::FocusEvent;
}

pub struct Blur;
impl EventDescriptor for Blur {
    type EventType = crate::event::status::FocusEvent;
}
