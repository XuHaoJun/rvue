//! Element state tracking for pseudo-class matching.

use bitflags::bitflags;

bitflags! {
    /// Represents the state of a widget element for pseudo-class matching.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct ElementState: u64 {
        const HOVER = 1 << 0;
        const FOCUS = 1 << 1;
        const ACTIVE = 1 << 2;
        const DISABLED = 1 << 3;
        const CHECKED = 1 << 4;
        const EMPTY = 0;
    }
}
