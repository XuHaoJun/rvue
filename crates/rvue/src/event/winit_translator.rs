use ui_events_winit::{WindowEventReducer, WindowEventTranslation};
use vello::kurbo::Point;
use winit::event::WindowEvent as WinitWindowEvent;

#[derive(Debug)]
pub struct WinitTranslator {
    reducer: WindowEventReducer,
}

impl Default for WinitTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl WinitTranslator {
    pub fn new() -> Self {
        Self { reducer: WindowEventReducer::default() }
    }

    pub fn translate(
        &mut self,
        scale_factor: f64,
        event: &WinitWindowEvent,
    ) -> Option<WindowEventTranslation> {
        self.reducer.reduce(scale_factor, event)
    }
}

pub fn get_pointer_event_position(event: &ui_events::pointer::PointerEvent) -> Option<Point> {
    match event {
        ui_events::pointer::PointerEvent::Down(e) => {
            Some(Point::new(e.state.position.x, e.state.position.y))
        }
        ui_events::pointer::PointerEvent::Up(e) => {
            Some(Point::new(e.state.position.x, e.state.position.y))
        }
        ui_events::pointer::PointerEvent::Move(e) => {
            Some(Point::new(e.current.position.x, e.current.position.y))
        }
        ui_events::pointer::PointerEvent::Enter(_) => None,
        ui_events::pointer::PointerEvent::Leave(_) => None,
        ui_events::pointer::PointerEvent::Scroll(e) => {
            Some(Point::new(e.state.position.x, e.state.position.y))
        }
        ui_events::pointer::PointerEvent::Gesture(e) => {
            Some(Point::new(e.state.position.x, e.state.position.y))
        }
        ui_events::pointer::PointerEvent::Cancel(_) => None,
    }
}
