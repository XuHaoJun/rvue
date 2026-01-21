use std::time::Duration;
use vello::kurbo::{Point, Vec2};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{Key, ModifiersState, PhysicalKey};

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    Rescale(f64),
    Resize(PhysicalSize<u32>),
    AnimFrame(Duration),
    FocusChange(bool),
    CloseRequested,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
}

impl From<MouseButton> for PointerButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => PointerButton::Primary,
            MouseButton::Right => PointerButton::Secondary,
            MouseButton::Middle => PointerButton::Middle,
            MouseButton::Back => PointerButton::Other(4),
            MouseButton::Forward => PointerButton::Other(5),
            MouseButton::Other(n) => PointerButton::Other(n),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerButtonEvent {
    pub button: PointerButton,
    pub position: Point,
    pub click_count: u32,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerMoveEvent {
    pub position: Point,
    pub delta: Vec2,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollDelta {
    Line(f64),
    Pixel(f64, f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerScrollEvent {
    pub delta: ScrollDelta,
    pub position: Point,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerInfo {
    pub position: Point,
}

impl Default for PointerInfo {
    fn default() -> Self {
        PointerInfo { position: Point::ZERO }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    Down(PointerButtonEvent),
    Up(PointerButtonEvent),
    Move(PointerMoveEvent),
    Enter(PointerInfo),
    Leave(PointerInfo),
    Cancel(PointerInfo),
    Scroll(PointerScrollEvent),
}

impl PointerEvent {
    pub fn position(&self) -> Option<Point> {
        match self {
            PointerEvent::Down(e) => Some(e.position),
            PointerEvent::Up(e) => Some(e.position),
            PointerEvent::Move(e) => Some(e.position),
            PointerEvent::Enter(e) => Some(e.position),
            PointerEvent::Leave(e) => Some(e.position),
            PointerEvent::Cancel(e) => Some(e.position),
            PointerEvent::Scroll(e) => Some(e.position),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyState {
    Down,
    Up,
}

impl From<ElementState> for KeyState {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => KeyState::Down,
            ElementState::Released => KeyState::Up,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImeCause {
    Focus,
    KeyboardInput,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImeEvent {
    Enabled(ImeCause),
    Preedit(String, usize),
    Commit(String),
    Disabled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyboardEvent {
    pub key: Key,
    pub code: PhysicalKey,
    pub state: KeyState,
    pub modifiers: Modifiers,
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

impl From<ModifiersState> for Modifiers {
    fn from(state: ModifiersState) -> Self {
        Modifiers {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            logo: state.super_key(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextEvent {
    Keyboard(KeyboardEvent),
    Ime(ImeEvent),
    Paste(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccessEvent {
    pub action: accesskit::Action,
    pub data: Option<accesskit::ActionData>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RvueEvent {
    Window(WindowEvent),
    Pointer(PointerEvent),
    Text(TextEvent),
    Access(AccessEvent),
}

pub trait IntoPoint {
    fn into_point(self) -> Point;
}

impl IntoPoint for PhysicalPosition<f64> {
    fn into_point(self) -> Point {
        Point::new(self.x, self.y)
    }
}

impl IntoPoint for PhysicalPosition<i32> {
    fn into_point(self) -> Point {
        Point::new(self.x as f64, self.y as f64)
    }
}

pub fn map_scroll_delta(delta: winit::event::MouseScrollDelta) -> ScrollDelta {
    match delta {
        winit::event::MouseScrollDelta::LineDelta(_x, y) => ScrollDelta::Line(y as f64),
        winit::event::MouseScrollDelta::PixelDelta(pos) => ScrollDelta::Pixel(pos.x, pos.y),
    }
}
