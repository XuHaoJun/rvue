use crate::component::Component;
use rudo_gc::Gc;
use vello::kurbo::Point;

pub trait EventContextOps {
    fn request_paint(&mut self);
    fn request_layout(&mut self);
    fn capture_pointer(&mut self, component: Gc<Component>);
    fn release_pointer(&mut self);
    fn request_focus(&mut self);
    fn resign_focus(&mut self);
    fn set_handled(&mut self);
    fn is_handled(&self) -> bool;
    fn target(&self) -> Gc<Component>;
    fn local_position(&self, window_pos: Point) -> Point;
    fn has_pointer_capture(&self) -> bool;
}

pub struct EventContext<'a> {
    target: Gc<Component>,
    app_state: &'a mut dyn EventContextOps,
    is_handled: bool,
    pointer_capture: Option<Gc<Component>>,
}

impl<'a> EventContext<'a> {
    pub fn new(
        target: Gc<Component>,
        app_state: &'a mut dyn EventContextOps,
        pointer_capture: Option<Gc<Component>>,
    ) -> Self {
        EventContext { target, app_state, is_handled: false, pointer_capture }
    }

    pub fn target(&self) -> Gc<Component> {
        Gc::clone(&self.target)
    }

    pub fn stop_propagation(&mut self) {
        self.is_handled = true;
    }

    pub fn is_handled(&self) -> bool {
        self.is_handled
    }

    pub fn capture_pointer(&mut self) {
        self.pointer_capture = Some(Gc::clone(&self.target));
        self.app_state.capture_pointer(Gc::clone(&self.target));
    }

    pub fn release_pointer(&mut self) {
        self.pointer_capture = None;
        self.app_state.release_pointer();
    }

    pub fn has_pointer_capture(&self) -> bool {
        self.pointer_capture.as_ref().is_some_and(|c| Gc::ptr_eq(c, &self.target))
    }

    pub fn request_focus(&mut self) {
        self.app_state.request_focus();
    }

    pub fn resign_focus(&mut self) {
        self.app_state.resign_focus();
    }

    pub fn request_paint(&mut self) {
        self.app_state.request_paint();
    }

    pub fn request_layout(&mut self) {
        self.app_state.request_layout();
    }

    pub fn local_position(&self, window_pos: Point) -> Point {
        self.app_state.local_position(window_pos)
    }
}
