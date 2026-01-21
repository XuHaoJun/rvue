use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ComponentFlags: u32 {
        const ACCEPTS_POINTER     = 1 << 0;
        const ACCEPTS_FOCUS       = 1 << 1;
        const ACCEPTS_TEXT_INPUT  = 1 << 2;
        const IS_DISABLED         = 1 << 3;
        const IS_STASHED          = 1 << 4;
        const IS_HOVERED          = 1 << 5;
        const HAS_HOVERED         = 1 << 6;
        const IS_ACTIVE           = 1 << 7;
        const HAS_ACTIVE          = 1 << 8;
        const IS_FOCUSED          = 1 << 9;
        const HAS_FOCUS_TARGET    = 1 << 10;
    }
}

unsafe impl rudo_gc::Trace for ComponentFlags {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {
        // ComponentFlags is a simple bitflag, no GC pointers to trace
    }
}

impl Default for ComponentFlags {
    fn default() -> Self {
        ComponentFlags::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusUpdate {
    Mounted,
    Unmounting,
    HoveredChanged(bool),
    ChildHoveredChanged(bool),
    ActiveChanged(bool),
    ChildActiveChanged(bool),
    FocusChanged(bool),
    ChildFocusChanged(bool),
    DisabledChanged(bool),
    VisibilityChanged(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusEvent {
    Gained,
    Lost,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    TextInput { value: String },
    NumberInput { value: f64 },
}
