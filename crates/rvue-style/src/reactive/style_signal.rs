//! Reactive style signal types
//!
//! This module provides types for reactive styling built on top of rvue-signals core types.

use crate::properties::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth,
    Color, ComputedStyles, Cursor, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink,
    FontFamily, FontSize, FontWeight, Gap, Height, JustifyContent, Margin, Opacity, Overflow,
    Padding, TextColor, Visibility, Width, ZIndex,
};
use crate::property::Property;
use bitflags::bitflags;
use rudo_gc::{Gc, GcCell, Trace};
use rvue_signals::{create_signal, ReadSignal, SignalRead, SignalWrite, WriteSignal};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct StyleFlags: u32 {
        const BACKGROUND_COLOR = 1 << 0;
        const COLOR = 1 << 1;
        const TEXT_COLOR = 1 << 2;
        const FONT_SIZE = 1 << 3;
        const FONT_FAMILY = 1 << 4;
        const FONT_WEIGHT = 1 << 5;
        const PADDING = 1 << 6;
        const MARGIN = 1 << 7;
        const WIDTH = 1 << 8;
        const HEIGHT = 1 << 9;
        const DISPLAY = 1 << 10;
        const FLEX_DIRECTION = 1 << 11;
        const JUSTIFY_CONTENT = 1 << 12;
        const ALIGN_ITEMS = 1 << 13;
        const ALIGN_SELF = 1 << 14;
        const FLEX_GROW = 1 << 15;
        const FLEX_SHRINK = 1 << 16;
        const FLEX_BASIS = 1 << 17;
        const GAP = 1 << 18;
        const BORDER_COLOR = 1 << 19;
        const BORDER_WIDTH = 1 << 20;
        const BORDER_RADIUS = 1 << 21;
        const BORDER_STYLE = 1 << 22;
        const OPACITY = 1 << 23;
        const VISIBILITY = 1 << 24;
        const Z_INDEX = 1 << 25;
        const CURSOR = 1 << 26;
        const OVERFLOW_X = 1 << 27;
        const OVERFLOW_Y = 1 << 28;
    }
}

thread_local! {
    static CURRENT_STYLE_EFFECT: RefCell<Option<Gc<StyleEffect>>> = const { RefCell::new(None) };
}

pub type ReactiveReadSignal<T> = ReadSignal<T>;
pub type ReactiveWriteSignal<T> = WriteSignal<T>;

pub fn create_reactive_signal<T: Clone + Trace + 'static>(
    initial_value: T,
) -> (ReactiveReadSignal<T>, ReactiveWriteSignal<T>) {
    create_signal(initial_value)
}

pub struct StyleEffect {
    closure: Rc<dyn Fn()>,
    is_dirty: AtomicU64,
    is_running: std::sync::atomic::AtomicBool,
    cleanups: GcCell<Vec<Box<dyn FnOnce()>>>,
}

unsafe impl Trace for StyleEffect {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

impl StyleEffect {
    fn new<F>(closure: F) -> Gc<Self>
    where
        F: Fn() + 'static,
    {
        let boxed = Rc::new(closure);
        Gc::new(Self {
            closure: boxed,
            is_dirty: AtomicU64::new(1),
            is_running: std::sync::atomic::AtomicBool::new(false),
            cleanups: GcCell::new(Vec::new()),
        })
    }

    fn run(gc_effect: &Gc<Self>) {
        if gc_effect.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let cleanups = {
            let mut cleanups = gc_effect.cleanups.borrow_mut_gen_only();
            std::mem::take(&mut *cleanups)
        };
        for cleanup in cleanups {
            cleanup();
        }

        gc_effect.is_dirty.store(0, Ordering::SeqCst);

        CURRENT_STYLE_EFFECT.with(|cell| {
            let previous = cell.borrow().clone();
            *cell.borrow_mut() = Some(Gc::clone(gc_effect));

            (gc_effect.closure)();

            *cell.borrow_mut() = previous;
        });

        gc_effect.is_running.store(false, Ordering::SeqCst);
    }
}

fn current_style_effect() -> Option<Gc<StyleEffect>> {
    CURRENT_STYLE_EFFECT.with(|cell| cell.borrow().clone())
}

pub fn create_style_effect<F>(closure: F) -> Gc<StyleEffect>
where
    F: Fn() + 'static,
    F: Clone,
{
    let effect = StyleEffect::new(closure.clone());
    StyleEffect::run(&effect);
    effect
}

pub fn on_style_cleanup<F>(f: F)
where
    F: FnOnce() + 'static,
{
    if let Some(effect) = current_style_effect() {
        effect.cleanups.borrow_mut_gen_only().push(Box::new(f));
    }
}

pub trait ReactiveSignal<T: Clone + Trace + 'static> {
    fn get(&self) -> T;
    fn get_untracked(&self) -> T;
}

pub trait ReactiveSignalWrite<T: Clone + Trace + 'static> {
    fn set(&self, value: T);
    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T);
}

impl<T: Clone + Trace + 'static> ReactiveSignal<T> for ReactiveReadSignal<T> {
    fn get(&self) -> T {
        SignalRead::get(self)
    }

    fn get_untracked(&self) -> T {
        SignalRead::get_untracked(self)
    }
}

impl<T: Clone + Trace + 'static> ReactiveSignalWrite<T> for ReactiveWriteSignal<T> {
    fn set(&self, value: T) {
        SignalWrite::set(self, value);
    }

    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        SignalWrite::update(self, f);
    }
}

/// Wrapper for dynamic value retrieval using a closure
pub struct DynamicValue<T: Clone + 'static> {
    getter: Rc<dyn Fn() -> T>,
}

impl<T: Clone + 'static> DynamicValue<T> {
    pub fn new(getter: Rc<dyn Fn() -> T>) -> Self {
        Self { getter }
    }

    pub fn get(&self) -> T {
        (self.getter)()
    }
}

impl<T: Clone + 'static> Clone for DynamicValue<T> {
    fn clone(&self) -> Self {
        Self { getter: Rc::clone(&self.getter) }
    }
}

#[derive(Clone)]
pub enum ReactiveProperty<T: Clone + Trace + 'static> {
    Static(T),
    Reactive(ReactiveReadSignal<T>),
    Dynamic(Rc<dyn Fn() -> T>),
}

impl<T: Clone + Trace + 'static> ReactiveProperty<T> {
    pub fn static_value(value: T) -> Self {
        ReactiveProperty::Static(value)
    }

    pub fn reactive(signal: ReactiveReadSignal<T>) -> Self {
        ReactiveProperty::Reactive(signal)
    }

    /// Create a ReactiveProperty from a closure that gets the value
    pub fn with_getter(getter: Rc<dyn Fn() -> T>) -> Self {
        ReactiveProperty::Dynamic(getter)
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        match self {
            ReactiveProperty::Static(value) => value.clone(),
            ReactiveProperty::Reactive(signal) => signal.get(),
            ReactiveProperty::Dynamic(getter) => getter(),
        }
    }

    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        match self {
            ReactiveProperty::Static(value) => value.clone(),
            ReactiveProperty::Reactive(signal) => signal.get_untracked(),
            ReactiveProperty::Dynamic(getter) => getter(),
        }
    }

    pub fn is_reactive(&self) -> bool {
        matches!(self, ReactiveProperty::Reactive(_))
    }
}

impl<T: Clone + Trace + 'static> From<T> for ReactiveProperty<T> {
    fn from(value: T) -> Self {
        ReactiveProperty::Static(value)
    }
}

impl<T: Clone + Trace + 'static> From<ReactiveReadSignal<T>> for ReactiveProperty<T> {
    fn from(signal: ReactiveReadSignal<T>) -> Self {
        ReactiveProperty::Reactive(signal)
    }
}

impl<T: Clone + Trace + 'static + Default> Default for ReactiveProperty<T> {
    fn default() -> Self {
        ReactiveProperty::Static(T::default())
    }
}

#[derive(Clone)]
pub struct ReactiveStyles {
    pub background_color: ReactiveProperty<BackgroundColor>,
    pub border_color: ReactiveProperty<BorderColor>,
    pub border_radius: ReactiveProperty<BorderRadius>,
    pub color: ReactiveProperty<Color>,
    pub text_color: ReactiveProperty<TextColor>,
    pub cursor: ReactiveProperty<Cursor>,
    pub display: ReactiveProperty<Display>,
    pub opacity: ReactiveProperty<Opacity>,
    pub visibility: ReactiveProperty<Visibility>,
    pub width: ReactiveProperty<Width>,
    pub height: ReactiveProperty<Height>,
    pub font_family: ReactiveProperty<FontFamily>,
    pub font_size: ReactiveProperty<FontSize>,
    pub font_weight: ReactiveProperty<FontWeight>,
    pub z_index: ReactiveProperty<ZIndex>,
    pub align_items: ReactiveProperty<AlignItems>,
    pub align_self: ReactiveProperty<AlignSelf>,
    pub border_style: ReactiveProperty<BorderStyle>,
    pub border_width: ReactiveProperty<BorderWidth>,
    pub flex_basis: ReactiveProperty<FlexBasis>,
    pub flex_direction: ReactiveProperty<FlexDirection>,
    pub flex_grow: ReactiveProperty<FlexGrow>,
    pub flex_shrink: ReactiveProperty<FlexShrink>,
    pub gap: ReactiveProperty<Gap>,
    pub justify_content: ReactiveProperty<JustifyContent>,
    pub margin: ReactiveProperty<Margin>,
    pub padding: ReactiveProperty<Padding>,
    pub overflow_x: ReactiveProperty<Overflow>,
    pub overflow_y: ReactiveProperty<Overflow>,
    flags: StyleFlags,
}

impl ReactiveStyles {
    pub fn new() -> Self {
        Self {
            background_color: ReactiveProperty::Static(BackgroundColor::default()),
            border_color: ReactiveProperty::Static(BorderColor::default()),
            border_radius: ReactiveProperty::Static(BorderRadius::default()),
            color: ReactiveProperty::Static(Color::default()),
            text_color: ReactiveProperty::Static(TextColor::default()),
            cursor: ReactiveProperty::Static(Cursor::default()),
            display: ReactiveProperty::Static(Display::default()),
            opacity: ReactiveProperty::Static(Opacity::default()),
            visibility: ReactiveProperty::Static(Visibility::default()),
            width: ReactiveProperty::Static(Width::default()),
            height: ReactiveProperty::Static(Height::default()),
            font_family: ReactiveProperty::Static(FontFamily::default()),
            font_size: ReactiveProperty::Static(FontSize::default()),
            font_weight: ReactiveProperty::Static(FontWeight::default()),
            z_index: ReactiveProperty::Static(ZIndex::default()),
            align_items: ReactiveProperty::Static(AlignItems::default()),
            align_self: ReactiveProperty::Static(AlignSelf::default()),
            border_style: ReactiveProperty::Static(BorderStyle::default()),
            border_width: ReactiveProperty::Static(BorderWidth::default()),
            flex_basis: ReactiveProperty::Static(FlexBasis::default()),
            flex_direction: ReactiveProperty::Static(FlexDirection::default()),
            flex_grow: ReactiveProperty::Static(FlexGrow::default()),
            flex_shrink: ReactiveProperty::Static(FlexShrink::default()),
            gap: ReactiveProperty::Static(Gap::default()),
            justify_content: ReactiveProperty::Static(JustifyContent::default()),
            margin: ReactiveProperty::Static(Margin::default()),
            padding: ReactiveProperty::Static(Padding::default()),
            overflow_x: ReactiveProperty::Static(Overflow::Visible),
            overflow_y: ReactiveProperty::Static(Overflow::Visible),
            flags: StyleFlags::empty(),
        }
    }

    pub fn set_background_color(
        mut self,
        value: impl Into<ReactiveProperty<BackgroundColor>>,
    ) -> Self {
        self.background_color = value.into();
        self.flags |= StyleFlags::BACKGROUND_COLOR;
        self
    }

    pub fn set_border_color(mut self, value: impl Into<ReactiveProperty<BorderColor>>) -> Self {
        self.border_color = value.into();
        self.flags |= StyleFlags::BORDER_COLOR;
        self
    }

    pub fn set_border_radius(mut self, value: impl Into<ReactiveProperty<BorderRadius>>) -> Self {
        self.border_radius = value.into();
        self.flags |= StyleFlags::BORDER_RADIUS;
        self
    }

    pub fn set_color(mut self, value: impl Into<ReactiveProperty<Color>>) -> Self {
        self.color = value.into();
        self.flags |= StyleFlags::COLOR;
        self
    }

    pub fn set_text_color(mut self, value: impl Into<ReactiveProperty<TextColor>>) -> Self {
        self.text_color = value.into();
        self.flags |= StyleFlags::TEXT_COLOR;
        self
    }

    pub fn set_cursor(mut self, value: impl Into<ReactiveProperty<Cursor>>) -> Self {
        self.cursor = value.into();
        self.flags |= StyleFlags::CURSOR;
        self
    }

    pub fn set_display(mut self, value: impl Into<ReactiveProperty<Display>>) -> Self {
        self.display = value.into();
        self.flags |= StyleFlags::DISPLAY;
        self
    }

    pub fn set_opacity(mut self, value: impl Into<ReactiveProperty<Opacity>>) -> Self {
        self.opacity = value.into();
        self.flags |= StyleFlags::OPACITY;
        self
    }

    pub fn set_visibility(mut self, value: impl Into<ReactiveProperty<Visibility>>) -> Self {
        self.visibility = value.into();
        self.flags |= StyleFlags::VISIBILITY;
        self
    }

    pub fn set_width(mut self, value: impl Into<ReactiveProperty<Width>>) -> Self {
        self.width = value.into();
        self.flags |= StyleFlags::WIDTH;
        self
    }

    pub fn set_height(mut self, value: impl Into<ReactiveProperty<Height>>) -> Self {
        self.height = value.into();
        self.flags |= StyleFlags::HEIGHT;
        self
    }

    pub fn set_font_family(mut self, value: impl Into<ReactiveProperty<FontFamily>>) -> Self {
        self.font_family = value.into();
        self.flags |= StyleFlags::FONT_FAMILY;
        self
    }

    pub fn set_font_size(mut self, value: impl Into<ReactiveProperty<FontSize>>) -> Self {
        self.font_size = value.into();
        self.flags |= StyleFlags::FONT_SIZE;
        self
    }

    pub fn set_font_weight(mut self, value: impl Into<ReactiveProperty<FontWeight>>) -> Self {
        self.font_weight = value.into();
        self.flags |= StyleFlags::FONT_WEIGHT;
        self
    }

    pub fn set_z_index(mut self, value: impl Into<ReactiveProperty<ZIndex>>) -> Self {
        self.z_index = value.into();
        self.flags |= StyleFlags::Z_INDEX;
        self
    }

    pub fn set_align_items(mut self, value: impl Into<ReactiveProperty<AlignItems>>) -> Self {
        self.align_items = value.into();
        self.flags |= StyleFlags::ALIGN_ITEMS;
        self
    }

    pub fn set_align_self(mut self, value: impl Into<ReactiveProperty<AlignSelf>>) -> Self {
        self.align_self = value.into();
        self.flags |= StyleFlags::ALIGN_SELF;
        self
    }

    pub fn set_border_style(mut self, value: impl Into<ReactiveProperty<BorderStyle>>) -> Self {
        self.border_style = value.into();
        self.flags |= StyleFlags::BORDER_STYLE;
        self
    }

    pub fn set_border_width(mut self, value: impl Into<ReactiveProperty<BorderWidth>>) -> Self {
        self.border_width = value.into();
        self.flags |= StyleFlags::BORDER_WIDTH;
        self
    }

    pub fn set_flex_basis(mut self, value: impl Into<ReactiveProperty<FlexBasis>>) -> Self {
        self.flex_basis = value.into();
        self.flags |= StyleFlags::FLEX_BASIS;
        self
    }

    pub fn set_flex_direction(mut self, value: impl Into<ReactiveProperty<FlexDirection>>) -> Self {
        self.flex_direction = value.into();
        self.flags |= StyleFlags::FLEX_DIRECTION;
        self
    }

    pub fn set_flex_grow(mut self, value: impl Into<ReactiveProperty<FlexGrow>>) -> Self {
        self.flex_grow = value.into();
        self.flags |= StyleFlags::FLEX_GROW;
        self
    }

    pub fn set_flex_shrink(mut self, value: impl Into<ReactiveProperty<FlexShrink>>) -> Self {
        self.flex_shrink = value.into();
        self.flags |= StyleFlags::FLEX_SHRINK;
        self
    }

    pub fn set_gap(mut self, value: impl Into<ReactiveProperty<Gap>>) -> Self {
        self.gap = value.into();
        self.flags |= StyleFlags::GAP;
        self
    }

    pub fn set_justify_content(
        mut self,
        value: impl Into<ReactiveProperty<JustifyContent>>,
    ) -> Self {
        self.justify_content = value.into();
        self.flags |= StyleFlags::JUSTIFY_CONTENT;
        self
    }

    pub fn set_margin(mut self, value: impl Into<ReactiveProperty<Margin>>) -> Self {
        self.margin = value.into();
        self.flags |= StyleFlags::MARGIN;
        self
    }

    pub fn set_padding(mut self, value: impl Into<ReactiveProperty<Padding>>) -> Self {
        self.padding = value.into();
        self.flags |= StyleFlags::PADDING;
        self
    }

    pub fn set_overflow_x(mut self, value: impl Into<ReactiveProperty<Overflow>>) -> Self {
        self.overflow_x = value.into();
        self.flags |= StyleFlags::OVERFLOW_X;
        self
    }

    pub fn set_overflow_y(mut self, value: impl Into<ReactiveProperty<Overflow>>) -> Self {
        self.overflow_y = value.into();
        self.flags |= StyleFlags::OVERFLOW_Y;
        self
    }

    /// Set overflow_y with a dynamic getter closure
    pub fn set_overflow_y_dynamic(mut self, getter: Rc<dyn Fn() -> Overflow>) -> Self {
        self.overflow_y = ReactiveProperty::with_getter(getter);
        self.flags |= StyleFlags::OVERFLOW_Y;
        self
    }

    /// Computes the final computed styles for rendering.
    ///
    /// This method returns a complete `ComputedStyles` with all properties populated:
    /// - Properties explicitly set via `set_*()` methods use their set value
    /// - Unset properties use their CSS initial value (never `None`)
    ///
    /// This follows CSS cascade semantics where unspecified properties use initial values.
    /// The result is always a complete style object suitable for rendering.
    pub fn compute(&self) -> ComputedStyles {
        let mut styles = ComputedStyles::new();
        let flags = self.flags;

        styles.background_color = Some(if flags.contains(StyleFlags::BACKGROUND_COLOR) {
            self.background_color.get_untracked()
        } else {
            BackgroundColor::initial_value()
        });
        styles.color = Some(if flags.contains(StyleFlags::COLOR) {
            self.color.get_untracked()
        } else {
            Color::initial_value()
        });
        styles.text_color = Some(if flags.contains(StyleFlags::TEXT_COLOR) {
            self.text_color.get_untracked()
        } else {
            TextColor::initial_value()
        });
        styles.font_size = Some(if flags.contains(StyleFlags::FONT_SIZE) {
            self.font_size.get_untracked()
        } else {
            FontSize::initial_value()
        });
        styles.font_family = Some(if flags.contains(StyleFlags::FONT_FAMILY) {
            self.font_family.get_untracked()
        } else {
            FontFamily::initial_value()
        });
        styles.font_weight = Some(if flags.contains(StyleFlags::FONT_WEIGHT) {
            self.font_weight.get_untracked()
        } else {
            FontWeight::initial_value()
        });
        styles.padding = Some(if flags.contains(StyleFlags::PADDING) {
            self.padding.get_untracked()
        } else {
            Padding::initial_value()
        });
        styles.margin = Some(if flags.contains(StyleFlags::MARGIN) {
            self.margin.get_untracked()
        } else {
            Margin::initial_value()
        });
        styles.width = Some(if flags.contains(StyleFlags::WIDTH) {
            self.width.get_untracked()
        } else {
            Width::initial_value()
        });
        styles.height = Some(if flags.contains(StyleFlags::HEIGHT) {
            self.height.get_untracked()
        } else {
            Height::initial_value()
        });
        styles.display = Some(if flags.contains(StyleFlags::DISPLAY) {
            self.display.get_untracked()
        } else {
            Display::initial_value()
        });
        styles.flex_direction = Some(if flags.contains(StyleFlags::FLEX_DIRECTION) {
            self.flex_direction.get_untracked()
        } else {
            FlexDirection::initial_value()
        });
        styles.justify_content = Some(if flags.contains(StyleFlags::JUSTIFY_CONTENT) {
            self.justify_content.get_untracked()
        } else {
            JustifyContent::initial_value()
        });
        styles.align_items = Some(if flags.contains(StyleFlags::ALIGN_ITEMS) {
            self.align_items.get_untracked()
        } else {
            AlignItems::initial_value()
        });
        styles.align_self = Some(if flags.contains(StyleFlags::ALIGN_SELF) {
            self.align_self.get_untracked()
        } else {
            AlignSelf::initial_value()
        });
        styles.flex_grow = Some(if flags.contains(StyleFlags::FLEX_GROW) {
            self.flex_grow.get_untracked()
        } else {
            FlexGrow::initial_value()
        });
        styles.flex_shrink = Some(if flags.contains(StyleFlags::FLEX_SHRINK) {
            self.flex_shrink.get_untracked()
        } else {
            FlexShrink::initial_value()
        });
        styles.flex_basis = Some(if flags.contains(StyleFlags::FLEX_BASIS) {
            self.flex_basis.get_untracked()
        } else {
            FlexBasis::initial_value()
        });
        styles.gap = Some(if flags.contains(StyleFlags::GAP) {
            self.gap.get_untracked()
        } else {
            Gap::initial_value()
        });
        styles.border_color = Some(if flags.contains(StyleFlags::BORDER_COLOR) {
            self.border_color.get_untracked()
        } else {
            BorderColor::initial_value()
        });
        styles.border_width = Some(if flags.contains(StyleFlags::BORDER_WIDTH) {
            self.border_width.get_untracked()
        } else {
            BorderWidth::initial_value()
        });
        styles.border_radius = Some(if flags.contains(StyleFlags::BORDER_RADIUS) {
            self.border_radius.get_untracked()
        } else {
            BorderRadius::initial_value()
        });
        styles.border_style = Some(if flags.contains(StyleFlags::BORDER_STYLE) {
            self.border_style.get_untracked()
        } else {
            BorderStyle::initial_value()
        });
        styles.opacity = Some(if flags.contains(StyleFlags::OPACITY) {
            self.opacity.get_untracked()
        } else {
            Opacity::initial_value()
        });
        styles.visibility = Some(if flags.contains(StyleFlags::VISIBILITY) {
            self.visibility.get_untracked()
        } else {
            Visibility::initial_value()
        });
        styles.z_index = Some(if flags.contains(StyleFlags::Z_INDEX) {
            self.z_index.get_untracked()
        } else {
            ZIndex::initial_value()
        });
        styles.cursor = Some(if flags.contains(StyleFlags::CURSOR) {
            self.cursor.get_untracked()
        } else {
            Cursor::initial_value()
        });
        styles.overflow_x = Some(if flags.contains(StyleFlags::OVERFLOW_X) {
            self.overflow_x.get_untracked()
        } else {
            Overflow::Visible
        });
        styles.overflow_y = Some(if flags.contains(StyleFlags::OVERFLOW_Y) {
            self.overflow_y.get_untracked()
        } else {
            Overflow::Visible
        });

        styles
    }
}

impl Default for ReactiveStyles {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ReactiveStyles> for ComputedStyles {
    fn from(styles: ReactiveStyles) -> Self {
        styles.compute()
    }
}

unsafe impl<T: Clone + Trace + 'static> Trace for ReactiveProperty<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        match self {
            ReactiveProperty::Static(value) => {
                // T is Trace, so trace it to mark any GC pointers it contains
                T::trace(value, visitor);
            }
            ReactiveProperty::Reactive(signal) => {
                // Trace the signal
                signal.trace(visitor);
            }
            ReactiveProperty::Dynamic(_) => {
                // Closures are conservatively scanned, nothing to trace here
            }
        }
    }
}

unsafe impl Trace for ReactiveStyles {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}
