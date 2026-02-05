//! Overflow properties for content clipping and scrolling.

use crate::property::Property;
use rudo_gc::Trace;

/// CSS overflow behavior - extended for web compatibility.
///
/// This enum mirrors CSS overflow values with an additional `Auto` variant
/// that rvue uses for dynamic scrollbar visibility (Taffy doesn't have Auto).
///
/// # CSS Mapping
/// - `Visible` → Content overflows visibly
/// - `Clip` → Content clipped, no scroll interaction
/// - `Hidden` → Content clipped, no scrollbar
/// - `Scroll` → Content clipped, always show scrollbar
/// - `Auto` → Content clipped, show scrollbar only when needed (rvue-only)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Trace)]
pub enum Overflow {
    /// Default. Content overflows visibly and contributes to parent scroll region.
    #[default]
    Visible,
    /// Content is clipped, doesn't contribute to parent scroll region,
    /// and minimum size is based on content.
    Clip,
    /// Content is clipped, minimum size is 0, no scrollbar shown.
    Hidden,
    /// Content is clipped, minimum size is 0, always reserve space for scrollbar.
    Scroll,
    /// rvue-only: Content is clipped, show scrollbar only when content overflows.
    /// This is resolved at layout time to either Hidden or Scroll.
    Auto,
}

impl Overflow {
    /// Whether this overflow mode creates a scroll container.
    /// A scroll container clips content and allows scroll interaction.
    #[inline]
    pub fn is_scroll_container(self) -> bool {
        matches!(self, Self::Hidden | Self::Scroll | Self::Auto)
    }

    /// Whether content should be clipped.
    #[inline]
    pub fn should_clip(self) -> bool {
        !matches!(self, Self::Visible)
    }
}

impl Property for Overflow {
    fn initial_value() -> Self {
        Self::Visible
    }
}
