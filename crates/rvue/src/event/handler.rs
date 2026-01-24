use crate::event::context::EventContext;
use crate::event::status::{FocusEvent, InputEvent};
use crate::event::types::{KeyboardEvent, PointerButtonEvent, PointerMoveEvent};
use std::cell::RefCell;
use std::rc::Rc;

pub struct EventHandler<E: 'static> {
    inner: Rc<RefCell<Option<Box<dyn Fn(&E, &mut EventContext)>>>>,
}

impl<E> Clone for EventHandler<E> {
    fn clone(&self) -> Self {
        EventHandler { inner: Rc::clone(&self.inner) }
    }
}

impl<E> EventHandler<E> {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&E, &mut EventContext) + 'static,
    {
        EventHandler { inner: Rc::new(RefCell::new(Some(Box::new(handler)))) }
    }

    pub fn call(&self, event: &E, ctx: &mut EventContext) {
        if let Some(handler) = self.inner.borrow().as_ref() {
            handler(event, ctx);
        }
    }
}

#[derive(Default, Clone)]
pub struct EventHandlers {
    pub on_pointer_down: Option<EventHandler<PointerButtonEvent>>,
    pub on_pointer_up: Option<EventHandler<PointerButtonEvent>>,
    pub on_pointer_move: Option<EventHandler<PointerMoveEvent>>,
    pub on_pointer_enter: Option<EventHandler<PointerButtonEvent>>,
    pub on_pointer_leave: Option<EventHandler<PointerButtonEvent>>,
    pub on_click: Option<EventHandler<PointerButtonEvent>>,
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    pub on_key_up: Option<EventHandler<KeyboardEvent>>,
    pub on_focus: Option<EventHandler<FocusEvent>>,
    pub on_blur: Option<EventHandler<FocusEvent>>,
    pub on_input: Option<EventHandler<InputEvent>>,
    pub on_change: Option<EventHandler<InputEvent>>,
}

impl EventHandlers {
    pub fn get_pointer_down(&self) -> Option<&EventHandler<PointerButtonEvent>> {
        self.on_pointer_down.as_ref()
    }

    pub fn get_pointer_up(&self) -> Option<&EventHandler<PointerButtonEvent>> {
        self.on_pointer_up.as_ref()
    }

    pub fn get_pointer_move(&self) -> Option<&EventHandler<PointerMoveEvent>> {
        self.on_pointer_move.as_ref()
    }

    pub fn get_click(&self) -> Option<&EventHandler<PointerButtonEvent>> {
        self.on_click.as_ref()
    }

    pub fn get_key_down(&self) -> Option<&EventHandler<KeyboardEvent>> {
        self.on_key_down.as_ref()
    }

    pub fn get_key_up(&self) -> Option<&EventHandler<KeyboardEvent>> {
        self.on_key_up.as_ref()
    }

    pub fn get_focus(&self) -> Option<&EventHandler<FocusEvent>> {
        self.on_focus.as_ref()
    }

    pub fn get_blur(&self) -> Option<&EventHandler<FocusEvent>> {
        self.on_blur.as_ref()
    }

    pub fn get_input(&self) -> Option<&EventHandler<InputEvent>> {
        self.on_input.as_ref()
    }

    pub fn get_change(&self) -> Option<&EventHandler<InputEvent>> {
        self.on_change.as_ref()
    }
}
