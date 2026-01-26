use crate::event::context::EventContext;
use crate::event::status::{FocusEvent, InputEvent};
use crate::event::types::{KeyboardEvent, PointerButtonEvent, PointerMoveEvent};
use rudo_gc::Trace;
use std::cell::RefCell;
use std::rc::Rc;

enum DynHandler {
    ZeroArg(Box<dyn Fn()>),
    OneArgPointerButton(Box<dyn Fn(&PointerButtonEvent)>),
    OneArgInput(Box<dyn Fn(&InputEvent)>),
    OneArgKeyboard(Box<dyn Fn(&KeyboardEvent)>),
    OneArgFocus(Box<dyn Fn(&FocusEvent)>),
    OneArgPointerMove(Box<dyn Fn(&PointerMoveEvent)>),
    TwoArgPointerButton(Box<dyn Fn(&PointerButtonEvent, &mut EventContext)>),
    TwoArgInput(Box<dyn Fn(&InputEvent, &mut EventContext)>),
    TwoArgKeyboard(Box<dyn Fn(&KeyboardEvent, &mut EventContext)>),
    TwoArgFocus(Box<dyn Fn(&FocusEvent, &mut EventContext)>),
    TwoArgPointerMove(Box<dyn Fn(&PointerMoveEvent, &mut EventContext)>),
}

pub enum AnyEventHandler {
    PointerButton(EventHandler<PointerButtonEvent>),
    Input(EventHandler<InputEvent>),
    Keyboard(EventHandler<KeyboardEvent>),
    Focus(EventHandler<FocusEvent>),
    PointerMove(EventHandler<PointerMoveEvent>),
}

pub struct EventHandler<E: 'static> {
    inner: Rc<RefCell<Option<DynHandler>>>,
    _phantom: std::marker::PhantomData<E>,
}

impl Clone for AnyEventHandler {
    fn clone(&self) -> Self {
        match self {
            AnyEventHandler::PointerButton(h) => AnyEventHandler::PointerButton(h.clone()),
            AnyEventHandler::Input(h) => AnyEventHandler::Input(h.clone()),
            AnyEventHandler::Keyboard(h) => AnyEventHandler::Keyboard(h.clone()),
            AnyEventHandler::Focus(h) => AnyEventHandler::Focus(h.clone()),
            AnyEventHandler::PointerMove(h) => AnyEventHandler::PointerMove(h.clone()),
        }
    }
}

impl<E> Clone for EventHandler<E> {
    fn clone(&self) -> Self {
        EventHandler { inner: Rc::clone(&self.inner), _phantom: std::marker::PhantomData }
    }
}

unsafe impl<E: 'static> Trace for EventHandler<E> {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

impl EventHandler<PointerButtonEvent> {
    pub fn new_0arg<F>(handler: F) -> Self
    where
        F: Fn() + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::ZeroArg(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_1arg<F>(handler: F) -> Self
    where
        F: Fn(&PointerButtonEvent) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::OneArgPointerButton(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&PointerButtonEvent, &mut EventContext) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::TwoArgPointerButton(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl EventHandler<InputEvent> {
    pub fn new_0arg<F>(handler: F) -> Self
    where
        F: Fn() + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::ZeroArg(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_1arg<F>(handler: F) -> Self
    where
        F: Fn(&InputEvent) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::OneArgInput(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&InputEvent, &mut EventContext) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::TwoArgInput(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl EventHandler<KeyboardEvent> {
    pub fn new_0arg<F>(handler: F) -> Self
    where
        F: Fn() + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::ZeroArg(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_1arg<F>(handler: F) -> Self
    where
        F: Fn(&KeyboardEvent) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::OneArgKeyboard(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&KeyboardEvent, &mut EventContext) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::TwoArgKeyboard(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl EventHandler<FocusEvent> {
    pub fn new_0arg<F>(handler: F) -> Self
    where
        F: Fn() + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::ZeroArg(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_1arg<F>(handler: F) -> Self
    where
        F: Fn(&FocusEvent) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::OneArgFocus(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&FocusEvent, &mut EventContext) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::TwoArgFocus(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl EventHandler<PointerMoveEvent> {
    pub fn new_0arg<F>(handler: F) -> Self
    where
        F: Fn() + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::ZeroArg(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_1arg<F>(handler: F) -> Self
    where
        F: Fn(&PointerMoveEvent) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::OneArgPointerMove(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&PointerMoveEvent, &mut EventContext) + 'static,
    {
        EventHandler {
            inner: Rc::new(RefCell::new(Some(DynHandler::TwoArgPointerMove(Box::new(handler))))),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl EventHandler<PointerButtonEvent> {
    pub fn call(&self, event: &PointerButtonEvent, ctx: &mut EventContext) {
        if let Some(handler) = self.inner.borrow().as_ref() {
            match handler {
                DynHandler::ZeroArg(f) => f(),
                DynHandler::OneArgPointerButton(f) => f(event),
                DynHandler::TwoArgPointerButton(f) => f(event, ctx),
                _ => {}
            }
        }
    }
}

impl EventHandler<InputEvent> {
    pub fn call(&self, event: &InputEvent, ctx: &mut EventContext) {
        if let Some(handler) = self.inner.borrow().as_ref() {
            match handler {
                DynHandler::ZeroArg(f) => f(),
                DynHandler::OneArgInput(f) => f(event),
                DynHandler::TwoArgInput(f) => f(event, ctx),
                _ => {}
            }
        }
    }
}

impl EventHandler<KeyboardEvent> {
    pub fn call(&self, event: &KeyboardEvent, ctx: &mut EventContext) {
        if let Some(handler) = self.inner.borrow().as_ref() {
            match handler {
                DynHandler::ZeroArg(f) => f(),
                DynHandler::OneArgKeyboard(f) => f(event),
                DynHandler::TwoArgKeyboard(f) => f(event, ctx),
                _ => {}
            }
        }
    }
}

impl EventHandler<FocusEvent> {
    pub fn call(&self, event: &FocusEvent, ctx: &mut EventContext) {
        if let Some(handler) = self.inner.borrow().as_ref() {
            match handler {
                DynHandler::ZeroArg(f) => f(),
                DynHandler::OneArgFocus(f) => f(event),
                DynHandler::TwoArgFocus(f) => f(event, ctx),
                _ => {}
            }
        }
    }
}

impl EventHandler<PointerMoveEvent> {
    pub fn call(&self, event: &PointerMoveEvent, ctx: &mut EventContext) {
        if let Some(handler) = self.inner.borrow().as_ref() {
            match handler {
                DynHandler::ZeroArg(f) => f(),
                DynHandler::OneArgPointerMove(f) => f(event),
                DynHandler::TwoArgPointerMove(f) => f(event, ctx),
                _ => {}
            }
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

unsafe impl Trace for EventHandlers {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {
        // EventHandlers contain EventHandler which don't contain GC pointers
    }
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

    pub fn set_handler<E: 'static>(&mut self, handler: EventHandler<E>) {
        let type_id = std::any::TypeId::of::<E>();
        if type_id == std::any::TypeId::of::<PointerButtonEvent>() {
            let ptr = &handler as *const EventHandler<E> as *const EventHandler<PointerButtonEvent>;
            let inner_ptr = ptr as *const _ as *const std::cell::RefCell<Option<DynHandler>>;
            let taken = unsafe { std::ptr::read(inner_ptr) };
            self.on_click =
                Some(EventHandler { inner: Rc::new(taken), _phantom: std::marker::PhantomData });
        } else if type_id == std::any::TypeId::of::<InputEvent>() {
            let ptr = &handler as *const EventHandler<E> as *const EventHandler<InputEvent>;
            let inner_ptr = ptr as *const _ as *const std::cell::RefCell<Option<DynHandler>>;
            let taken = unsafe { std::ptr::read(inner_ptr) };
            self.on_input =
                Some(EventHandler { inner: Rc::new(taken), _phantom: std::marker::PhantomData });
        } else if type_id == std::any::TypeId::of::<KeyboardEvent>() {
            let ptr = &handler as *const EventHandler<E> as *const EventHandler<KeyboardEvent>;
            let inner_ptr = ptr as *const _ as *const std::cell::RefCell<Option<DynHandler>>;
            let taken = unsafe { std::ptr::read(inner_ptr) };
            self.on_key_down =
                Some(EventHandler { inner: Rc::new(taken), _phantom: std::marker::PhantomData });
        } else if type_id == std::any::TypeId::of::<FocusEvent>() {
            let ptr = &handler as *const EventHandler<E> as *const EventHandler<FocusEvent>;
            let inner_ptr = ptr as *const _ as *const std::cell::RefCell<Option<DynHandler>>;
            let taken = unsafe { std::ptr::read(inner_ptr) };
            self.on_focus =
                Some(EventHandler { inner: Rc::new(taken), _phantom: std::marker::PhantomData });
        } else if type_id == std::any::TypeId::of::<PointerMoveEvent>() {
            let ptr = &handler as *const EventHandler<E> as *const EventHandler<PointerMoveEvent>;
            let inner_ptr = ptr as *const _ as *const std::cell::RefCell<Option<DynHandler>>;
            let taken = unsafe { std::ptr::read(inner_ptr) };
            self.on_pointer_move =
                Some(EventHandler { inner: Rc::new(taken), _phantom: std::marker::PhantomData });
        }
    }
}
