pub mod context;
pub mod dispatch;
pub mod focus;
pub mod handler;
pub mod hit_test;
pub mod path;
pub mod status;
pub mod types;
pub mod update;

pub use context::EventContext;
pub use dispatch::{run_pointer_event_pass, run_text_event_pass};
pub use focus::find_next_focusable;
pub use handler::{EventHandler, EventHandlers};
pub use hit_test::hit_test;
pub use status::StatusUpdate;
pub use types::{
    ImeEvent, KeyboardEvent, PointerButton, PointerButtonEvent, PointerEvent, PointerMoveEvent,
    PointerScrollEvent, RvueEvent, TextEvent, WindowEvent,
};
pub use update::{run_update_focus_pass, run_update_pointer_pass};

pub trait EventDescriptor {
    type EventType;
}

#[cfg(test)]
mod tests {
    use super::types::AccessEvent;
    use super::*;
    use crate::event::types::map_scroll_delta;
    use crate::event::types::{ImeCause, KeyState, Modifiers, PointerInfo, ScrollDelta};
    use vello::kurbo::{Point, Vec2};
    use winit::event::{ElementState, MouseButton, MouseScrollDelta};

    #[test]
    fn test_pointer_button_conversion() {
        assert_eq!(PointerButton::from(MouseButton::Left), PointerButton::Primary);
        assert_eq!(PointerButton::from(MouseButton::Right), PointerButton::Secondary);
        assert_eq!(PointerButton::from(MouseButton::Middle), PointerButton::Middle);
        assert_eq!(PointerButton::from(MouseButton::Back), PointerButton::Other(4));
        assert_eq!(PointerButton::from(MouseButton::Forward), PointerButton::Other(5));
        assert_eq!(PointerButton::from(MouseButton::Other(3)), PointerButton::Other(3));
    }

    #[test]
    fn test_key_state_conversion() {
        assert_eq!(KeyState::from(ElementState::Pressed), KeyState::Down);
        assert_eq!(KeyState::from(ElementState::Released), KeyState::Up);
    }

    #[test]
    fn test_modifiers_from_state() {
        use winit::keyboard::ModifiersState;

        let state = ModifiersState::empty();
        let modifiers: Modifiers = state.into();
        assert!(!modifiers.shift);
        assert!(!modifiers.ctrl);

        let state = ModifiersState::SHIFT | ModifiersState::CONTROL;
        let modifiers: Modifiers = state.into();
        assert!(modifiers.shift);
        assert!(modifiers.ctrl);
        assert!(!modifiers.alt);
    }

    #[test]
    fn test_scroll_delta_mapping() {
        match map_scroll_delta(MouseScrollDelta::LineDelta(0.0, 3.0)) {
            ScrollDelta::Line(y) => assert!((y - 3.0).abs() < f64::EPSILON),
            _ => panic!("Expected Line delta"),
        }

        match map_scroll_delta(MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition {
            x: 10.0,
            y: 20.0,
        })) {
            ScrollDelta::Pixel(x, y) => {
                assert!((x - 10.0).abs() < f64::EPSILON);
                assert!((y - 20.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected Pixel delta"),
        }
    }

    #[test]
    fn test_pointer_event_position() {
        let down_event = PointerEvent::Down(PointerButtonEvent {
            button: PointerButton::Primary,
            position: Point::new(100.0, 200.0),
            click_count: 1,
            modifiers: Modifiers::default(),
        });
        assert_eq!(down_event.position(), Some(Point::new(100.0, 200.0)));

        let move_event = PointerEvent::Move(PointerMoveEvent {
            position: Point::new(50.0, 75.0),
            delta: Vec2::new(10.0, 20.0),
            modifiers: Modifiers::default(),
        });
        assert_eq!(move_event.position(), Some(Point::new(50.0, 75.0)));

        let scroll_event = PointerEvent::Scroll(PointerScrollEvent {
            delta: ScrollDelta::Line(1.0),
            position: Point::ZERO,
            modifiers: Modifiers::default(),
        });
        assert_eq!(scroll_event.position(), Some(Point::ZERO));
    }

    #[test]
    fn test_status_update_variants() {
        assert_eq!(StatusUpdate::HoveredChanged(true), StatusUpdate::HoveredChanged(true));
        assert_ne!(StatusUpdate::HoveredChanged(true), StatusUpdate::HoveredChanged(false));

        assert_eq!(StatusUpdate::ActiveChanged(false), StatusUpdate::ActiveChanged(false));
        assert_eq!(StatusUpdate::FocusChanged(true), StatusUpdate::FocusChanged(true));
        assert_eq!(StatusUpdate::DisabledChanged(false), StatusUpdate::DisabledChanged(false));
    }

    #[test]
    fn test_component_flags_operations() {
        use status::ComponentFlags;

        let mut flags = ComponentFlags::empty();
        assert!(!flags.contains(ComponentFlags::ACCEPTS_POINTER));
        assert!(!flags.contains(ComponentFlags::ACCEPTS_FOCUS));

        flags.insert(ComponentFlags::ACCEPTS_POINTER);
        flags.insert(ComponentFlags::ACCEPTS_FOCUS);
        assert!(flags.contains(ComponentFlags::ACCEPTS_POINTER));
        assert!(flags.contains(ComponentFlags::ACCEPTS_FOCUS));

        flags.remove(ComponentFlags::ACCEPTS_POINTER);
        assert!(!flags.contains(ComponentFlags::ACCEPTS_POINTER));
        assert!(flags.contains(ComponentFlags::ACCEPTS_FOCUS));

        flags.insert(ComponentFlags::IS_DISABLED);
        assert!(flags.contains(ComponentFlags::IS_DISABLED));

        let combined = ComponentFlags::ACCEPTS_POINTER | ComponentFlags::ACCEPTS_FOCUS;
        assert!(combined.contains(ComponentFlags::ACCEPTS_POINTER));
        assert!(combined.contains(ComponentFlags::ACCEPTS_FOCUS));
    }

    #[test]
    fn test_event_handlers_creation() {
        let handlers: EventHandlers = EventHandlers::default();

        assert!(handlers.get_pointer_down().is_none());
        assert!(handlers.get_pointer_up().is_none());
        assert!(handlers.get_pointer_move().is_none());
        assert!(handlers.get_click().is_none());
        assert!(handlers.get_key_down().is_none());
        assert!(handlers.get_key_up().is_none());
    }

    #[test]
    fn test_handled_enum() {
        use dispatch::Handled;

        assert_eq!(Handled::Yes, Handled::Yes);
        assert_eq!(Handled::No, Handled::No);
        assert_ne!(Handled::Yes, Handled::No);
    }

    #[test]
    fn test_window_event_variants() {
        use types::WindowEvent;

        let resize = WindowEvent::Resize(winit::dpi::PhysicalSize::new(800, 600));
        assert_eq!(format!("{:?}", resize), "Resize(PhysicalSize { width: 800, height: 600 })");

        let close = WindowEvent::CloseRequested;
        assert_eq!(format!("{:?}", close), "CloseRequested");

        let focus = WindowEvent::FocusChange(true);
        assert_eq!(format!("{:?}", focus), "FocusChange(true)");
    }

    #[test]
    fn test_pointer_button_variants() {
        assert_eq!(PointerButton::Primary, PointerButton::Primary);
        assert_eq!(PointerButton::Secondary, PointerButton::Secondary);
        assert_eq!(PointerButton::Middle, PointerButton::Middle);
        assert_eq!(PointerButton::Other(4), PointerButton::Other(4));
        assert_ne!(PointerButton::Other(4), PointerButton::Other(5));
    }

    #[test]
    fn test_scroll_delta_variants() {
        let line = ScrollDelta::Line(3.0);
        let pixel = ScrollDelta::Pixel(10.5, 20.5);

        match line {
            ScrollDelta::Line(y) => assert!((y - 3.0).abs() < f64::EPSILON),
            _ => panic!("Expected Line"),
        }

        match pixel {
            ScrollDelta::Pixel(x, y) => {
                assert!((x - 10.5).abs() < f64::EPSILON);
                assert!((y - 20.5).abs() < f64::EPSILON);
            }
            _ => panic!("Expected Pixel"),
        }
    }

    #[test]
    fn test_text_event_keyboard() {
        use winit::keyboard::{Key, NamedKey, PhysicalKey};

        let key_event = KeyboardEvent {
            key: Key::Named(NamedKey::Enter),
            code: PhysicalKey::Code(winit::keyboard::KeyCode::Enter),
            state: KeyState::Down,
            modifiers: Modifiers { shift: false, ctrl: true, alt: false, logo: false },
            repeat: false,
        };

        assert_eq!(key_event.state, KeyState::Down);
        assert!(key_event.modifiers.ctrl);
        assert!(!key_event.modifiers.shift);
    }

    #[test]
    fn test_focus_event_variants() {
        assert_eq!(status::FocusEvent::Gained, status::FocusEvent::Gained);
        assert_eq!(status::FocusEvent::Lost, status::FocusEvent::Lost);
        assert_ne!(status::FocusEvent::Gained, status::FocusEvent::Lost);
    }

    #[test]
    fn test_input_event() {
        use status::InputEventType;
        let text_input = status::InputEvent {
            value: "Hello".to_string(),
            number_value: 0.0,
            checked: false,
            input_type: InputEventType::Text,
        };
        assert_eq!(text_input.value, "Hello");
        assert_eq!(text_input.input_type, InputEventType::Text);

        let number_input = status::InputEvent {
            value: String::new(),
            number_value: 42.0,
            checked: false,
            input_type: InputEventType::Number,
        };
        assert_eq!(number_input.number_value, 42.0);
        assert_eq!(number_input.input_type, InputEventType::Number);
    }

    #[test]
    fn test_ime_event_variants() {
        let enabled = ImeEvent::Enabled(ImeCause::Other);
        match enabled {
            ImeEvent::Enabled(cause) => assert_eq!(cause, ImeCause::Other),
            _ => panic!("Expected Enabled"),
        }

        let preedit = ImeEvent::Preedit("test".to_string(), 5);
        match preedit {
            ImeEvent::Preedit(text, cursor) => {
                assert_eq!(text, "test");
                assert_eq!(cursor, 5);
            }
            _ => panic!("Expected Preedit"),
        }

        let commit = ImeEvent::Commit("final".to_string());
        match commit {
            ImeEvent::Commit(text) => assert_eq!(text, "final"),
            _ => panic!("Expected Commit"),
        }

        let disabled = ImeEvent::Disabled;
        match disabled {
            ImeEvent::Disabled => {}
            _ => panic!("Expected Disabled"),
        }
    }

    #[test]
    fn test_modifiers_default() {
        let default = Modifiers::default();
        assert!(!default.shift);
        assert!(!default.ctrl);
        assert!(!default.alt);
        assert!(!default.logo);
    }

    #[test]
    fn test_modifiers_with_values() {
        let modifiers = Modifiers { shift: true, ctrl: true, alt: false, logo: true };
        assert!(modifiers.shift);
        assert!(modifiers.ctrl);
        assert!(!modifiers.alt);
        assert!(modifiers.logo);
    }

    #[test]
    fn test_pointer_info_default() {
        let info = PointerInfo::default();
        assert_eq!(info.position, Point::ZERO);
    }

    #[test]
    fn test_access_event() {
        let event = AccessEvent { action: accesskit::Action::Click, data: None };
        assert_eq!(event.action, accesskit::Action::Click);
        assert!(event.data.is_none());
    }
}
