//! Application runner with winit event loop

use crate::component::{Component, ComponentLifecycle};
use crate::event::context::EventContextOps;
use crate::event::dispatch::{
    run_pointer_event_pass, run_text_event_pass, update_cursor_blink_states,
};
use crate::event::handler::ScrollDragState;
use crate::event::hit_test::hit_test;
use crate::event::types::{
    map_scroll_delta, KeyState as RvueKeyState, KeyboardEvent as RvueKeyboardEvent,
    Modifiers as RvueModifiers, PointerButtonEvent, PointerEvent, PointerMoveEvent,
};
use crate::event::update::{run_update_focus_pass, run_update_pointer_pass};
use crate::event::winit_translator::{get_pointer_event_position, WinitTranslator};
use crate::render::Scene as RvueScene;
use crate::style::Stylesheet;
use crate::vello_util::{CreateSurfaceError, RenderContext, RenderSurface};
use crate::view::ViewStruct;
use rudo_gc::{Gc, GcCell};
use std::cell::RefMut;
use std::io::Write;
use std::sync::Arc;
use vello::kurbo::Affine;
use vello::kurbo::{Point, Vec2};
use vello::peniko::Color;
use vello::{AaConfig, AaSupport, Renderer, RendererOptions};
use wgpu::Color as WgpuColor;
use wgpu::SurfaceTexture;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

pub trait AppStateLike {
    fn root_component(&self) -> Gc<Component>;
    fn pointer_capture(&self) -> Option<Gc<Component>>;
    fn pointer_capture_mut(&mut self) -> RefMut<'_, Option<Gc<Component>>>;
    fn last_pointer_pos(&self) -> Option<Point>;
    fn hovered_component(&self) -> Option<Gc<Component>>;
    fn focused(&self) -> Option<Gc<Component>>;
    fn focused_mut(&mut self) -> &mut Option<Gc<Component>>;
    fn fallback(&self) -> Option<Gc<Component>>;
    fn pending_focus(&mut self) -> &mut Option<Gc<Component>>;
    fn active_path(&mut self) -> &mut Vec<Gc<Component>>;
    fn hovered_path(&mut self) -> &mut Vec<Gc<Component>>;
    fn focused_path(&mut self) -> &mut Vec<Gc<Component>>;
    fn set_active_path(&mut self, path: Vec<Gc<Component>>);
    fn set_hovered_path(&mut self, path: Vec<Gc<Component>>);
    fn set_focused_path(&mut self, path: Vec<Gc<Component>>);
    fn set_needs_pointer_pass_update(&mut self, _value: bool);
    fn needs_pointer_pass_update(&self) -> bool;
    fn set_focused(&mut self, focused: Option<Gc<Component>>);
    fn set_needs_cursor_blink_update(&mut self);
    fn clear_pointer_capture(&mut self);
    fn scroll_drag_state(&self) -> Option<ScrollDragState>;
    fn set_scroll_drag_state(&mut self, state: Option<ScrollDragState>);
}

pub struct FocusState {
    pub focused: Option<Gc<Component>>,
    pub fallback: Option<Gc<Component>>,
    pub pending_focus: Option<Gc<Component>>,
    pub focus_anchor: Option<Gc<Component>>,
}

/// Application state
/// Fields are ordered for correct drop order: GC resources first, window last
pub struct AppState<'a> {
    view: Option<ViewStruct>,
    scene: RvueScene,
    pub stylesheet: Option<Stylesheet>,
    pub focus_state: FocusState,
    pub pointer_capture: GcCell<Option<Gc<Component>>>,
    pub last_pointer_pos: Option<Point>,
    pub hovered_component: GcCell<Option<Gc<Component>>>,
    pub active_path: Vec<Gc<Component>>,
    pub hovered_path: Vec<Gc<Component>>,
    pub focused_path: Vec<Gc<Component>>,
    pub needs_pointer_pass_update: bool,
    pub scroll_drag_state: Option<ScrollDragState>,
    pub last_gc_count: usize,
    pub last_anim_duration: Option<u64>,
    pub needs_cursor_blink_update: bool,
    renderer: Option<Renderer>,
    surface: Option<RenderSurface<'a>>,
    render_cx: Option<RenderContext>,
    window: Option<Arc<Window>>,
    event_translator: WinitTranslator,
}

impl<'a> AppStateLike for AppState<'a> {
    fn root_component(&self) -> Gc<Component> {
        self.view
            .as_ref()
            .map(|v| v.root_component.clone())
            .unwrap_or_else(|| panic!("No root component"))
    }

    fn pointer_capture(&self) -> Option<Gc<Component>> {
        self.pointer_capture.borrow().clone()
    }

    fn pointer_capture_mut(&mut self) -> RefMut<'_, Option<Gc<Component>>> {
        self.pointer_capture.borrow_mut()
    }

    fn last_pointer_pos(&self) -> Option<Point> {
        self.last_pointer_pos
    }

    fn hovered_component(&self) -> Option<Gc<Component>> {
        self.hovered_component.borrow().clone()
    }

    fn focused(&self) -> Option<Gc<Component>> {
        self.focus_state.focused.clone()
    }

    fn focused_mut(&mut self) -> &mut Option<Gc<Component>> {
        &mut self.focus_state.focused
    }

    fn fallback(&self) -> Option<Gc<Component>> {
        self.focus_state.fallback.clone()
    }

    fn pending_focus(&mut self) -> &mut Option<Gc<Component>> {
        &mut self.focus_state.pending_focus
    }

    fn active_path(&mut self) -> &mut Vec<Gc<Component>> {
        &mut self.active_path
    }

    fn hovered_path(&mut self) -> &mut Vec<Gc<Component>> {
        &mut self.hovered_path
    }

    fn focused_path(&mut self) -> &mut Vec<Gc<Component>> {
        &mut self.focused_path
    }

    fn set_active_path(&mut self, path: Vec<Gc<Component>>) {
        self.active_path = path;
    }

    fn set_hovered_path(&mut self, path: Vec<Gc<Component>>) {
        self.hovered_path = path;
    }

    fn set_focused_path(&mut self, path: Vec<Gc<Component>>) {
        self.focused_path = path;
    }

    fn set_needs_pointer_pass_update(&mut self, value: bool) {
        self.needs_pointer_pass_update = value;
    }

    fn needs_pointer_pass_update(&self) -> bool {
        self.needs_pointer_pass_update
    }

    fn set_focused(&mut self, focused: Option<Gc<Component>>) {
        self.focus_state.focused = focused;
    }

    fn clear_pointer_capture(&mut self) {
        *self.pointer_capture.borrow_mut() = None;
    }

    fn scroll_drag_state(&self) -> Option<ScrollDragState> {
        self.scroll_drag_state
    }

    fn set_scroll_drag_state(&mut self, state: Option<ScrollDragState>) {
        self.scroll_drag_state = state;
    }

    fn set_needs_cursor_blink_update(&mut self) {
        self.needs_cursor_blink_update = true;
    }
}

impl EventContextOps for AppState<'_> {
    fn request_paint(&mut self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn request_layout(&mut self) {
        if let Some(view) = &self.view {
            view.root_component.mark_dirty();
        }
        self.request_paint();
    }

    fn capture_pointer(&mut self, component: Gc<Component>) {
        *self.pointer_capture.borrow_mut() = Some(component);
    }

    fn release_pointer(&mut self) {
        *self.pointer_capture.borrow_mut() = None;
    }

    fn request_focus(&mut self) {
        // Focus will be applied in the next update pass
        // The actual focus target is set by EventContext using self.target
    }

    fn resign_focus(&mut self) {
        self.focus_state.focused = None;
    }

    fn set_handled(&mut self) {}

    fn is_handled(&self) -> bool {
        false
    }

    fn target(&self) -> Gc<Component> {
        self.root_component()
    }

    fn local_position(&self, window_pos: Point) -> Point {
        window_pos
    }

    fn has_pointer_capture(&self) -> bool {
        false
    }

    fn set_pending_focus(&mut self, component: Gc<Component>) {
        self.focus_state.pending_focus = Some(component);
    }

    fn set_needs_cursor_blink_update(&mut self) {
        self.needs_cursor_blink_update = true;
    }
}

impl<'a> AppState<'a> {
    fn new() -> Self {
        Self {
            renderer: None,
            surface: None,
            render_cx: None,
            window: None,
            view: None,
            scene: RvueScene::new(),
            stylesheet: None,
            focus_state: FocusState {
                focused: None,
                fallback: None,
                pending_focus: None,
                focus_anchor: None,
            },
            pointer_capture: GcCell::new(None),
            last_pointer_pos: None,
            hovered_component: GcCell::new(None),
            active_path: Vec::new(),
            hovered_path: Vec::new(),
            focused_path: Vec::new(),
            needs_pointer_pass_update: false,
            scroll_drag_state: None,
            last_gc_count: 0,
            last_anim_duration: None,
            needs_cursor_blink_update: false,
            event_translator: WinitTranslator::new(),
        }
    }

    fn handle_translated_pointer_event(
        &mut self,
        event: &ui_events::pointer::PointerEvent,
        scale_factor: f64,
    ) {
        if let Some(pos) = get_pointer_event_position(event) {
            let logical_x = pos.x / scale_factor;
            let logical_y = pos.y / scale_factor;
            let logical_pos = Point::new(logical_x, logical_y);
            self.last_pointer_pos = Some(logical_pos);

            let new_hovered = hit_test(&self.root_component(), logical_pos);
            *self.hovered_component.borrow_mut() = new_hovered;
        }

        self.scene.update();

        let converted_event =
            crate::event::types::convert_pointer_event_from_ui_events(event, scale_factor);

        run_pointer_event_pass(self, &converted_event);
        self.request_redraw_if_dirty();
    }
}

/// Application handler for winit event loop
impl ApplicationHandler for AppState<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = winit::window::Window::default_attributes()
                .with_title("Rvue Application")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            let window = event_loop.create_window(window_attributes).unwrap();
            self.window = Some(Arc::new(window));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let scale_factor =
            self.window.as_ref().map(|w: &Arc<Window>| w.scale_factor()).unwrap_or(1.0);

        // Check if this is a MouseWheel event BEFORE translation
        use winit::event::WindowEvent;
        let is_wheel_event = matches!(event, WindowEvent::MouseWheel { .. });
        if is_wheel_event {
            if let WindowEvent::MouseWheel { delta, .. } = event {
                let scroll_event = PointerEvent::Scroll(crate::event::types::PointerScrollEvent {
                    delta: map_scroll_delta(delta),
                    position: self.last_pointer_pos.unwrap_or_default(),
                    modifiers: self.current_modifiers(),
                });
                run_pointer_event_pass(self, &scroll_event);
                self.request_redraw_if_dirty();
                return;
            }
        }

        let translated = self.event_translator.translate(scale_factor, &event);

        if let Some(ref translated_event) = translated {
            match translated_event {
                ui_events_winit::WindowEventTranslation::Pointer(pointer_event) => {
                    self.handle_translated_pointer_event(pointer_event, scale_factor);
                    return;
                }
                ui_events_winit::WindowEventTranslation::Keyboard(key_event) => {
                    // Process pending focus before handling keyboard events
                    run_update_focus_pass(self);

                    // Convert ui-events keyboard event to rvue keyboard event
                    // Note: code field conversion is simplified since text input mainly uses 'key'
                    let physical_code = match key_event.code {
                        keyboard_types::Code::KeyA => winit::keyboard::KeyCode::KeyA,
                        keyboard_types::Code::KeyB => winit::keyboard::KeyCode::KeyB,
                        keyboard_types::Code::KeyC => winit::keyboard::KeyCode::KeyC,
                        keyboard_types::Code::KeyD => winit::keyboard::KeyCode::KeyD,
                        keyboard_types::Code::KeyE => winit::keyboard::KeyCode::KeyE,
                        keyboard_types::Code::KeyF => winit::keyboard::KeyCode::KeyF,
                        keyboard_types::Code::KeyG => winit::keyboard::KeyCode::KeyG,
                        keyboard_types::Code::KeyH => winit::keyboard::KeyCode::KeyH,
                        keyboard_types::Code::KeyI => winit::keyboard::KeyCode::KeyI,
                        keyboard_types::Code::KeyJ => winit::keyboard::KeyCode::KeyJ,
                        keyboard_types::Code::KeyK => winit::keyboard::KeyCode::KeyK,
                        keyboard_types::Code::KeyL => winit::keyboard::KeyCode::KeyL,
                        keyboard_types::Code::KeyM => winit::keyboard::KeyCode::KeyM,
                        keyboard_types::Code::KeyN => winit::keyboard::KeyCode::KeyN,
                        keyboard_types::Code::KeyO => winit::keyboard::KeyCode::KeyO,
                        keyboard_types::Code::KeyP => winit::keyboard::KeyCode::KeyP,
                        keyboard_types::Code::KeyQ => winit::keyboard::KeyCode::KeyQ,
                        keyboard_types::Code::KeyR => winit::keyboard::KeyCode::KeyR,
                        keyboard_types::Code::KeyS => winit::keyboard::KeyCode::KeyS,
                        keyboard_types::Code::KeyT => winit::keyboard::KeyCode::KeyT,
                        keyboard_types::Code::KeyU => winit::keyboard::KeyCode::KeyU,
                        keyboard_types::Code::KeyV => winit::keyboard::KeyCode::KeyV,
                        keyboard_types::Code::KeyW => winit::keyboard::KeyCode::KeyW,
                        keyboard_types::Code::KeyX => winit::keyboard::KeyCode::KeyX,
                        keyboard_types::Code::KeyY => winit::keyboard::KeyCode::KeyY,
                        keyboard_types::Code::KeyZ => winit::keyboard::KeyCode::KeyZ,
                        keyboard_types::Code::Digit1 => winit::keyboard::KeyCode::Digit1,
                        keyboard_types::Code::Digit2 => winit::keyboard::KeyCode::Digit2,
                        keyboard_types::Code::Digit3 => winit::keyboard::KeyCode::Digit3,
                        keyboard_types::Code::Digit4 => winit::keyboard::KeyCode::Digit4,
                        keyboard_types::Code::Digit5 => winit::keyboard::KeyCode::Digit5,
                        keyboard_types::Code::Digit6 => winit::keyboard::KeyCode::Digit6,
                        keyboard_types::Code::Digit7 => winit::keyboard::KeyCode::Digit7,
                        keyboard_types::Code::Digit8 => winit::keyboard::KeyCode::Digit8,
                        keyboard_types::Code::Digit9 => winit::keyboard::KeyCode::Digit9,
                        keyboard_types::Code::Digit0 => winit::keyboard::KeyCode::Digit0,
                        keyboard_types::Code::Enter => winit::keyboard::KeyCode::Enter,
                        keyboard_types::Code::Escape => winit::keyboard::KeyCode::Escape,
                        keyboard_types::Code::Backspace => winit::keyboard::KeyCode::Backspace,
                        keyboard_types::Code::Tab => winit::keyboard::KeyCode::Tab,
                        keyboard_types::Code::Space => winit::keyboard::KeyCode::Space,
                        keyboard_types::Code::Minus => winit::keyboard::KeyCode::Minus,
                        keyboard_types::Code::Equal => winit::keyboard::KeyCode::Equal,
                        keyboard_types::Code::BracketLeft => winit::keyboard::KeyCode::BracketLeft,
                        keyboard_types::Code::BracketRight => {
                            winit::keyboard::KeyCode::BracketRight
                        }
                        keyboard_types::Code::Backslash => winit::keyboard::KeyCode::Backslash,
                        keyboard_types::Code::Semicolon => winit::keyboard::KeyCode::Semicolon,
                        keyboard_types::Code::Quote => winit::keyboard::KeyCode::Quote,
                        keyboard_types::Code::Backquote => winit::keyboard::KeyCode::Backquote,
                        keyboard_types::Code::Comma => winit::keyboard::KeyCode::Comma,
                        keyboard_types::Code::Period => winit::keyboard::KeyCode::Period,
                        keyboard_types::Code::Slash => winit::keyboard::KeyCode::Slash,
                        keyboard_types::Code::CapsLock => winit::keyboard::KeyCode::CapsLock,
                        keyboard_types::Code::F1 => winit::keyboard::KeyCode::F1,
                        keyboard_types::Code::F2 => winit::keyboard::KeyCode::F2,
                        keyboard_types::Code::F3 => winit::keyboard::KeyCode::F3,
                        keyboard_types::Code::F4 => winit::keyboard::KeyCode::F4,
                        keyboard_types::Code::F5 => winit::keyboard::KeyCode::F5,
                        keyboard_types::Code::F6 => winit::keyboard::KeyCode::F6,
                        keyboard_types::Code::F7 => winit::keyboard::KeyCode::F7,
                        keyboard_types::Code::F8 => winit::keyboard::KeyCode::F8,
                        keyboard_types::Code::F9 => winit::keyboard::KeyCode::F9,
                        keyboard_types::Code::F10 => winit::keyboard::KeyCode::F10,
                        keyboard_types::Code::F11 => winit::keyboard::KeyCode::F11,
                        keyboard_types::Code::F12 => winit::keyboard::KeyCode::F12,
                        keyboard_types::Code::Insert => winit::keyboard::KeyCode::Insert,
                        keyboard_types::Code::Home => winit::keyboard::KeyCode::Home,
                        keyboard_types::Code::PageUp => winit::keyboard::KeyCode::PageUp,
                        keyboard_types::Code::Delete => winit::keyboard::KeyCode::Delete,
                        keyboard_types::Code::End => winit::keyboard::KeyCode::End,
                        keyboard_types::Code::PageDown => winit::keyboard::KeyCode::PageDown,
                        keyboard_types::Code::ArrowRight => winit::keyboard::KeyCode::ArrowRight,
                        keyboard_types::Code::ArrowLeft => winit::keyboard::KeyCode::ArrowLeft,
                        keyboard_types::Code::ArrowDown => winit::keyboard::KeyCode::ArrowDown,
                        keyboard_types::Code::ArrowUp => winit::keyboard::KeyCode::ArrowUp,
                        keyboard_types::Code::Numpad1 => winit::keyboard::KeyCode::Numpad1,
                        keyboard_types::Code::Numpad2 => winit::keyboard::KeyCode::Numpad2,
                        keyboard_types::Code::Numpad3 => winit::keyboard::KeyCode::Numpad3,
                        keyboard_types::Code::Numpad4 => winit::keyboard::KeyCode::Numpad4,
                        keyboard_types::Code::Numpad5 => winit::keyboard::KeyCode::Numpad5,
                        keyboard_types::Code::Numpad6 => winit::keyboard::KeyCode::Numpad6,
                        keyboard_types::Code::Numpad7 => winit::keyboard::KeyCode::Numpad7,
                        keyboard_types::Code::Numpad8 => winit::keyboard::KeyCode::Numpad8,
                        keyboard_types::Code::Numpad9 => winit::keyboard::KeyCode::Numpad9,
                        keyboard_types::Code::Numpad0 => winit::keyboard::KeyCode::Numpad0,
                        keyboard_types::Code::NumpadDecimal => {
                            winit::keyboard::KeyCode::NumpadDecimal
                        }
                        keyboard_types::Code::NumpadEnter => winit::keyboard::KeyCode::NumpadEnter,
                        keyboard_types::Code::NumpadAdd => winit::keyboard::KeyCode::NumpadAdd,
                        keyboard_types::Code::NumpadSubtract => {
                            winit::keyboard::KeyCode::NumpadSubtract
                        }
                        keyboard_types::Code::NumpadMultiply => {
                            winit::keyboard::KeyCode::NumpadMultiply
                        }
                        keyboard_types::Code::NumpadDivide => {
                            winit::keyboard::KeyCode::NumpadDivide
                        }
                        // For unknown codes, use Backquote as a fallback
                        _ => winit::keyboard::KeyCode::Backquote,
                    };

                    let key_event = RvueKeyboardEvent {
                        key: match &key_event.key {
                            ui_events::keyboard::Key::Character(c) => {
                                winit::keyboard::Key::Character(c.as_str().into())
                            }
                            ui_events::keyboard::Key::Named(named) => {
                                winit::keyboard::Key::Named(match named {
                                    ui_events::keyboard::NamedKey::Backspace => {
                                        winit::keyboard::NamedKey::Backspace
                                    }
                                    ui_events::keyboard::NamedKey::Tab => {
                                        winit::keyboard::NamedKey::Tab
                                    }
                                    ui_events::keyboard::NamedKey::Enter => {
                                        winit::keyboard::NamedKey::Enter
                                    }
                                    ui_events::keyboard::NamedKey::Escape => {
                                        winit::keyboard::NamedKey::Escape
                                    }
                                    ui_events::keyboard::NamedKey::ArrowDown => {
                                        winit::keyboard::NamedKey::ArrowDown
                                    }
                                    ui_events::keyboard::NamedKey::ArrowLeft => {
                                        winit::keyboard::NamedKey::ArrowLeft
                                    }
                                    ui_events::keyboard::NamedKey::ArrowRight => {
                                        winit::keyboard::NamedKey::ArrowRight
                                    }
                                    ui_events::keyboard::NamedKey::ArrowUp => {
                                        winit::keyboard::NamedKey::ArrowUp
                                    }
                                    ui_events::keyboard::NamedKey::Delete => {
                                        winit::keyboard::NamedKey::Delete
                                    }
                                    ui_events::keyboard::NamedKey::Home => {
                                        winit::keyboard::NamedKey::Home
                                    }
                                    ui_events::keyboard::NamedKey::End => {
                                        winit::keyboard::NamedKey::End
                                    }
                                    ui_events::keyboard::NamedKey::PageUp => {
                                        winit::keyboard::NamedKey::PageUp
                                    }
                                    ui_events::keyboard::NamedKey::PageDown => {
                                        winit::keyboard::NamedKey::PageDown
                                    }
                                    ui_events::keyboard::NamedKey::Insert => {
                                        winit::keyboard::NamedKey::Insert
                                    }
                                    ui_events::keyboard::NamedKey::F1 => {
                                        winit::keyboard::NamedKey::F1
                                    }
                                    ui_events::keyboard::NamedKey::F2 => {
                                        winit::keyboard::NamedKey::F2
                                    }
                                    ui_events::keyboard::NamedKey::F3 => {
                                        winit::keyboard::NamedKey::F3
                                    }
                                    ui_events::keyboard::NamedKey::F4 => {
                                        winit::keyboard::NamedKey::F4
                                    }
                                    ui_events::keyboard::NamedKey::F5 => {
                                        winit::keyboard::NamedKey::F5
                                    }
                                    ui_events::keyboard::NamedKey::F6 => {
                                        winit::keyboard::NamedKey::F6
                                    }
                                    ui_events::keyboard::NamedKey::F7 => {
                                        winit::keyboard::NamedKey::F7
                                    }
                                    ui_events::keyboard::NamedKey::F8 => {
                                        winit::keyboard::NamedKey::F8
                                    }
                                    ui_events::keyboard::NamedKey::F9 => {
                                        winit::keyboard::NamedKey::F9
                                    }
                                    ui_events::keyboard::NamedKey::F10 => {
                                        winit::keyboard::NamedKey::F10
                                    }
                                    ui_events::keyboard::NamedKey::F11 => {
                                        winit::keyboard::NamedKey::F11
                                    }
                                    ui_events::keyboard::NamedKey::F12 => {
                                        winit::keyboard::NamedKey::F12
                                    }
                                    ui_events::keyboard::NamedKey::CapsLock => {
                                        winit::keyboard::NamedKey::CapsLock
                                    }
                                    ui_events::keyboard::NamedKey::Shift => {
                                        winit::keyboard::NamedKey::Shift
                                    }
                                    ui_events::keyboard::NamedKey::Control => {
                                        winit::keyboard::NamedKey::Control
                                    }
                                    ui_events::keyboard::NamedKey::Alt => {
                                        winit::keyboard::NamedKey::Alt
                                    }
                                    ui_events::keyboard::NamedKey::Meta => {
                                        winit::keyboard::NamedKey::Meta
                                    }
                                    _ => winit::keyboard::NamedKey::Cancel,
                                })
                            }
                        },
                        code: winit::keyboard::PhysicalKey::Code(physical_code),
                        state: match key_event.state {
                            ui_events::keyboard::KeyState::Down => RvueKeyState::Down,
                            ui_events::keyboard::KeyState::Up => RvueKeyState::Up,
                        },
                        modifiers: RvueModifiers {
                            shift: key_event.modifiers.shift(),
                            ctrl: key_event.modifiers.ctrl(),
                            alt: key_event.modifiers.alt(),
                            logo: key_event.modifiers.meta(),
                        },
                        repeat: key_event.repeat,
                    };

                    run_text_event_pass(self, &crate::event::types::TextEvent::Keyboard(key_event));
                    self.request_redraw_if_dirty();
                    return;
                }
            }
        }

        std::io::stderr().flush().ok();

        match event {
            WindowEvent::CloseRequested => {
                // Clear GC resources before exiting to prevent double-free
                self.active_path.clear();
                self.hovered_path.clear();
                self.focused_path.clear();
                *self.pointer_capture.borrow_mut() = None;
                *self.hovered_component.borrow_mut() = None;
                self.scene.root_components.clear();
                self.scene.vello_scene = None;
                self.view = None;
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.handle_resize(size);
            }
            WindowEvent::RedrawRequested => {
                self.run_update_passes();
                self.render_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale_factor = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);
                let logical_x = position.x / scale_factor;
                let logical_y = position.y / scale_factor;
                let point = Point::new(logical_x, logical_y);
                self.last_pointer_pos = Some(point);

                // Ensure layout is up to date before hit testing
                self.scene.update();

                let new_hovered = hit_test(&self.root_component(), point);
                *self.hovered_component.borrow_mut() = new_hovered;

                let event = PointerEvent::Move(PointerMoveEvent {
                    position: point,
                    delta: Vec2::ZERO,
                    modifiers: self.current_modifiers(),
                });
                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let position = self.last_pointer_pos.unwrap_or_default();

                let event = match state {
                    ElementState::Pressed => PointerEvent::Down(PointerButtonEvent {
                        button: button.into(),
                        position,
                        click_count: 1,
                        modifiers: self.current_modifiers(),
                    }),
                    ElementState::Released => PointerEvent::Up(PointerButtonEvent {
                        button: button.into(),
                        position,
                        click_count: 1,
                        modifiers: self.current_modifiers(),
                    }),
                };

                // Ensure layout is up to date before event dispatch (which includes hit testing)
                self.scene.update();

                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::CursorEntered { .. } => {
                run_pointer_event_pass(self, &PointerEvent::Enter(Default::default()));
            }
            WindowEvent::CursorLeft { .. } => {
                *self.hovered_component.borrow_mut() = None;
                run_pointer_event_pass(self, &PointerEvent::Leave(Default::default()));
            }
            WindowEvent::KeyboardInput { event: input, .. } => {
                // Process pending focus before handling keyboard events
                run_update_focus_pass(self);

                let key_event = RvueKeyboardEvent {
                    key: input.logical_key,
                    code: input.physical_key,
                    state: input.state.into(),
                    modifiers: self.current_modifiers(),
                    repeat: input.repeat,
                };
                run_text_event_pass(self, &crate::event::types::TextEvent::Keyboard(key_event));
                self.request_redraw_if_dirty();
            }
            WindowEvent::Ime(ime_event) => {
                // Process pending focus before handling IME events
                run_update_focus_pass(self);

                let ime = match ime_event {
                    winit::event::Ime::Enabled => {
                        crate::event::types::ImeEvent::Enabled(crate::event::types::ImeCause::Other)
                    }
                    winit::event::Ime::Preedit(text, cursor) => {
                        crate::event::types::ImeEvent::Preedit(text, cursor.map_or(0, |c| c.0))
                    }
                    winit::event::Ime::Commit(text) => crate::event::types::ImeEvent::Commit(text),
                    winit::event::Ime::Disabled => crate::event::types::ImeEvent::Disabled,
                };
                run_text_event_pass(self, &crate::event::types::TextEvent::Ime(ime));
                self.request_redraw_if_dirty();
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    *self.pointer_capture.borrow_mut() = None;
                    *self.hovered_component.borrow_mut() = None;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let event = PointerEvent::Scroll(crate::event::types::PointerScrollEvent {
                    delta: map_scroll_delta(delta),
                    position: self.last_pointer_pos.unwrap_or_default(),
                    modifiers: self.current_modifiers(),
                });
                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::AxisMotion { axis, value, .. } => {
                if axis == 1 && value != 0.0 {
                    let scroll_delta = crate::event::types::ScrollDelta::Line(value);
                    let event = PointerEvent::Scroll(crate::event::types::PointerScrollEvent {
                        delta: scroll_delta,
                        position: self.last_pointer_pos.unwrap_or_default(),
                        modifiers: self.current_modifiers(),
                    });
                    run_pointer_event_pass(self, &event);
                    self.request_redraw_if_dirty();
                }
            }
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let (Some(ref render_cx), Some(ref surface)) = (&self.render_cx, &self.surface) {
            let dev_id = surface.dev_id;
            let device = &render_cx.devices[dev_id].device;
            let _ = device.poll(wgpu::PollType::Poll);
        }
    }
}

impl<'a> Drop for AppState<'a> {
    fn drop(&mut self) {
        // Clear all component paths first - these hold Gc references
        self.active_path.clear();
        self.hovered_path.clear();
        self.focused_path.clear();

        // Clear pointer capture and hovered component
        *self.pointer_capture.borrow_mut() = None;
        *self.hovered_component.borrow_mut() = None;

        // Clear the scene's root components to break the reference chain
        self.scene.root_components.clear();
        self.scene.vello_scene = None;

        // Clear the view to break the reference to root component
        self.view = None;

        // GC is disabled during app lifetime, so no need to run cleanup
    }
}

impl<'a> AppState<'a> {
    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if let (Some(ref mut render_cx), Some(ref mut surface)) =
            (&mut self.render_cx, &mut self.surface)
        {
            render_cx.resize_surface(surface, size.width, size.height);
        }
    }

    fn run_update_passes(&mut self) {
        if self.needs_pointer_pass_update {
            run_update_pointer_pass(self);
            self.needs_pointer_pass_update = false;
        }
        run_update_focus_pass(self);

        // Update cursor blink animation
        if self.needs_cursor_blink_update {
            if let Some(duration) = self.last_anim_duration {
                update_cursor_blink_states(&self.root_component(), duration);
            }
            self.needs_cursor_blink_update = false;
        }

        // Request redraw after focus changes to ensure cursor renders
        self.request_redraw_if_dirty();

        // Process component lifecycle updates and effects
        self.root_component().update();

        // Monitor GC performance
        self.monitor_gc();
    }

    fn monitor_gc(&mut self) {
        let metrics = rudo_gc::last_gc_metrics();
        if metrics.total_collections > self.last_gc_count {
            self.last_gc_count = metrics.total_collections;

            let duration_ms = metrics.duration.as_millis();
            if duration_ms > 16 {
                log::warn!("GC pause of {}ms exceeded frame budget (16ms)!", duration_ms);
            }
        }
    }

    fn request_redraw_if_dirty(&self) {
        let root_dirty = self.view.as_ref().map(|v| v.root_component.is_dirty()).unwrap_or(false);
        if root_dirty {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn current_modifiers(&self) -> crate::event::types::Modifiers {
        ModifiersState::default().into()
    }

    fn render_frame(&mut self) {
        let (scale_factor, size) =
            match self.window.as_ref().map(|w| (w.scale_factor(), w.inner_size())) {
                Some((sf, s)) if s.width != 0 && s.height != 0 => (sf, s),
                _ => return,
            };

        let surface_texture = match self.get_or_create_surface(size) {
            Ok(Some(st)) => st,
            Ok(None) => return,
            Err(e) => {
                log::error!("Rendering initialization failed: {}", e);
                return;
            }
        };

        let (render_cx, surface) = match (self.render_cx.as_mut(), self.surface.as_mut()) {
            (Some(cx), Some(s)) => (cx, s),
            _ => return,
        };

        let dev_id = surface.dev_id;
        let device = &render_cx.devices[dev_id].device;
        let queue = &render_cx.devices[dev_id].queue;
        let _surface_format = surface.format;

        let render_params = vello::RenderParams {
            base_color: Color::WHITE,
            width: size.width,
            height: size.height,
            antialiasing_method: AaConfig::Area,
        };

        let renderer = self.renderer.get_or_insert_with(|| {
            let options = RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            };
            Renderer::new(device, options).expect("Failed to create Vello renderer")
        });

        // Populate scene from view if not already done
        if self.scene.root_components.is_empty() {
            if let Some(view) = &self.view {
                self.scene.add_fragment(view.root_component.clone());
            }
        }

        // Set stylesheet if not already set
        if self.scene.stylesheet.is_none() {
            if let Some(stylesheet) = &self.stylesheet {
                self.scene.set_stylesheet(stylesheet.clone());
            }
        }

        // Update scene (regenerates the underlying vello::Scene if dirty)
        self.scene.update();

        let scene = self.scene.vello_scene();

        let transformed_scene = if scale_factor == 1.0 {
            None
        } else {
            let mut new_scene = vello::Scene::new();
            new_scene.append(scene, Some(Affine::scale(scale_factor)));
            Some(new_scene)
        };
        let scene_ref = transformed_scene.as_ref().unwrap_or(scene);

        // Clear intermediate texture before rendering
        {
            let mut clear_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Clear Texture"),
                });
            {
                let _clear_pass = clear_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface.target_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(WgpuColor::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }
            queue.submit([clear_encoder.finish()]);
        }

        // Render to intermediate texture
        if let Err(e) = renderer.render_to_texture(
            device,
            queue,
            scene_ref,
            &surface.target_view,
            &render_params,
        ) {
            log::error!("Vello render to texture failed: {}", e);
            return;
        }

        // Blit to surface
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surface Blit"),
        });

        // Intermediate texture format now matches the surface format
        encoder.copy_texture_to_texture(
            surface.target_texture.as_image_copy(),
            surface_texture.texture.as_image_copy(),
            wgpu::Extent3d { width: size.width, height: size.height, depth_or_array_layers: 1 },
        );

        queue.submit([encoder.finish()]);

        if let Some(window) = &self.window {
            window.pre_present_notify();
        }
        surface_texture.present();

        // GPU synchronization
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
    }

    fn get_or_create_surface(
        &mut self,
        size: PhysicalSize<u32>,
    ) -> Result<Option<SurfaceTexture>, CreateSurfaceError> {
        if self.render_cx.is_none() {
            self.render_cx = Some(RenderContext::new());
        }

        let render_cx = self.render_cx.as_mut().unwrap();

        if let Some(surface) = self.surface.as_mut() {
            if surface.config.width != size.width || surface.config.height != size.height {
                render_cx.resize_surface(surface, size.width, size.height);
            }
        } else {
            let window = match &self.window {
                Some(w) => w.clone(),
                None => return Ok(None),
            };
            let new_surface = pollster::block_on(render_cx.create_surface(
                window,
                size.width,
                size.height,
                wgpu::PresentMode::AutoVsync,
            ))?;
            self.surface = Some(new_surface);
        }

        let surface = self.surface.as_mut().unwrap();
        match surface.surface.get_current_texture() {
            Ok(texture) => Ok(Some(texture)),
            Err(wgpu::SurfaceError::Outdated) => {
                let new_size = self.window.as_ref().map(|w| w.inner_size()).unwrap_or(size);
                render_cx.resize_surface(surface, new_size.width, new_size.height);
                match surface.surface.get_current_texture() {
                    Ok(texture) => Ok(Some(texture)),
                    Err(e) => {
                        log::error!("Failed to get surface texture after resize: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get surface texture: {}", e);
                Ok(None)
            }
        }
    }
}

/// Run the application with the given view
///
/// # Arguments
///
/// * `view_fn` - A function that returns the root view of the application
///
/// # Example
///
/// ```ignore
/// use rvue::prelude::*;
///
/// fn main() {
///     rvue::run_app(|| {
///         view! {
///             <Text value="Hello, Rvue!" />
///         }
///     });
/// }
/// ```
pub fn run_app<F>(view_fn: F) -> Result<(), AppError>
where
    F: FnOnce() -> ViewStruct + 'static,
{
    // Disable automatic GC collection during app lifetime to prevent
    // race conditions with component drops
    // The final cleanup in AppState::drop will re-enable and run GC
    rudo_gc::set_collect_condition(|_| false);

    let view = view_fn();

    let event_loop = EventLoop::with_user_event()
        .build()
        .map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    let mut app_state = AppState::new();
    app_state.view = Some(view);

    // Add default stylesheet for component sizing (buttons, inputs, etc.)
    app_state.stylesheet = Some(Stylesheet::with_defaults());

    // Run the event loop - AppState::drop will handle cleanup
    event_loop
        .run_app(&mut app_state)
        .map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    Ok(())
}

/// Run the application with a stylesheet for CSS selector matching.
///
/// # Arguments
///
/// * `view_fn` - A function that returns the root view of the application
/// * `stylesheet` - An optional stylesheet for CSS selector-based styling
///
/// # Example
///
/// ```ignore
/// use rvue::prelude::*;
/// use rvue_style::{Stylesheet, BackgroundColor, Color};
///
/// fn main() {
///     let mut stylesheet = Stylesheet::new();
///     stylesheet.add_rule("button.primary", Properties::with(
///         BackgroundColor(Color::rgb(0, 123, 255))
///     ));
///     stylesheet.add_rule("button:hover", Properties::with(
///         BackgroundColor(Color::rgb(0, 86, 179))
///     ));
///
///     rvue::run_app_with_stylesheet(|| {
///         view! {
///             <Button class="primary">
///                 <Text>Primary</Text>
///             </Button>
///         }
///     }, Some(stylesheet));
/// }
/// ```
pub fn run_app_with_stylesheet<F>(
    view_fn: F,
    stylesheet: Option<Stylesheet>,
) -> Result<(), AppError>
where
    F: FnOnce() -> ViewStruct + 'static,
{
    rudo_gc::set_collect_condition(|_| false);

    let view = view_fn();

    let event_loop = EventLoop::with_user_event()
        .build()
        .map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    let mut app_state = AppState::new();
    app_state.view = Some(view);

    let merged_stylesheet = match stylesheet {
        Some(user_sheet) => {
            let mut defaults = Stylesheet::with_defaults();
            defaults.merge(&user_sheet);
            defaults
        }
        None => Stylesheet::with_defaults(),
    };
    app_state.stylesheet = Some(merged_stylesheet);

    event_loop
        .run_app(&mut app_state)
        .map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    Ok(())
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    WindowCreationFailed(String),
    RendererInitializationFailed(String),
    ComponentCreationFailed(String),
    LayoutCalculationFailed(String),
    GcError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::WindowCreationFailed(msg) => write!(f, "Window creation failed: {}", msg),
            AppError::RendererInitializationFailed(msg) => {
                write!(f, "Renderer initialization failed: {}", msg)
            }
            AppError::ComponentCreationFailed(msg) => {
                write!(f, "Component creation failed: {}", msg)
            }
            AppError::LayoutCalculationFailed(msg) => {
                write!(f, "Layout calculation failed: {}", msg)
            }
            AppError::GcError(msg) => write!(f, "GC error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}
