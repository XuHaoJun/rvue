//! Reactive style signal types
//!
//! This module provides types for reactive styling built on top of rvue-signals core types.

use crate::properties::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth,
    Color, ComputedStyles, Cursor, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink,
    FontFamily, FontSize, FontWeight, Gap, Height, JustifyContent, Margin, Opacity, Padding,
    TextColor, Visibility, Width, ZIndex,
};
use rudo_gc::{Gc, GcCell, Trace};
use rvue_signals::{create_signal, ReadSignal, SignalRead, SignalWrite, WriteSignal};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

thread_local! {
    static CURRENT_STYLE_EFFECT: RefCell<Option<Gc<StyleEffect>>> = const { RefCell::new(None) };
}

pub type ReactiveReadSignal<T> = ReadSignal<T>;
pub type ReactiveWriteSignal<T> = WriteSignal<T>;

pub fn create_reactive_signal<T: Clone + 'static>(
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
            let mut cleanups = gc_effect.cleanups.borrow_mut();
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
        effect.cleanups.borrow_mut().push(Box::new(f));
    }
}

pub trait ReactiveSignal<T: Clone + 'static> {
    fn get(&self) -> T;
    fn get_untracked(&self) -> T;
}

pub trait ReactiveSignalWrite<T: Clone + 'static> {
    fn set(&self, value: T);
    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T);
}

impl<T: Clone + 'static> ReactiveSignal<T> for ReactiveReadSignal<T> {
    fn get(&self) -> T {
        SignalRead::get(self)
    }

    fn get_untracked(&self) -> T {
        SignalRead::get_untracked(self)
    }
}

impl<T: Clone + 'static> ReactiveSignalWrite<T> for ReactiveWriteSignal<T> {
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

#[derive(Clone)]
pub enum ReactiveProperty<T: Clone + 'static> {
    Static(T),
    Reactive(ReactiveReadSignal<T>),
}

impl<T: Clone + 'static> ReactiveProperty<T> {
    pub fn static_value(value: T) -> Self {
        ReactiveProperty::Static(value)
    }

    pub fn reactive(signal: ReactiveReadSignal<T>) -> Self {
        ReactiveProperty::Reactive(signal)
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        match self {
            ReactiveProperty::Static(value) => value.clone(),
            ReactiveProperty::Reactive(signal) => signal.get(),
        }
    }

    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        match self {
            ReactiveProperty::Static(value) => value.clone(),
            ReactiveProperty::Reactive(signal) => signal.get_untracked(),
        }
    }

    pub fn is_reactive(&self) -> bool {
        matches!(self, ReactiveProperty::Reactive(_))
    }
}

impl<T: Clone + 'static> From<T> for ReactiveProperty<T> {
    fn from(value: T) -> Self {
        ReactiveProperty::Static(value)
    }
}

impl<T: Clone + 'static> From<ReactiveReadSignal<T>> for ReactiveProperty<T> {
    fn from(signal: ReactiveReadSignal<T>) -> Self {
        ReactiveProperty::Reactive(signal)
    }
}

impl<T: Clone + 'static + Default> Default for ReactiveProperty<T> {
    fn default() -> Self {
        ReactiveProperty::Static(T::default())
    }
}

#[derive(Clone, Default)]
pub struct ReactiveStyles {
    background_color: ReactiveProperty<BackgroundColor>,
    border_color: ReactiveProperty<BorderColor>,
    border_radius: ReactiveProperty<BorderRadius>,
    color: ReactiveProperty<Color>,
    text_color: ReactiveProperty<TextColor>,
    cursor: ReactiveProperty<Cursor>,
    display: ReactiveProperty<Display>,
    opacity: ReactiveProperty<Opacity>,
    visibility: ReactiveProperty<Visibility>,
    width: ReactiveProperty<Width>,
    height: ReactiveProperty<Height>,
    font_family: ReactiveProperty<FontFamily>,
    font_size: ReactiveProperty<FontSize>,
    font_weight: ReactiveProperty<FontWeight>,
    z_index: ReactiveProperty<ZIndex>,
    align_items: ReactiveProperty<AlignItems>,
    align_self: ReactiveProperty<AlignSelf>,
    border_style: ReactiveProperty<BorderStyle>,
    border_width: ReactiveProperty<BorderWidth>,
    flex_basis: ReactiveProperty<FlexBasis>,
    flex_direction: ReactiveProperty<FlexDirection>,
    flex_grow: ReactiveProperty<FlexGrow>,
    flex_shrink: ReactiveProperty<FlexShrink>,
    gap: ReactiveProperty<Gap>,
    justify_content: ReactiveProperty<JustifyContent>,
    margin: ReactiveProperty<Margin>,
    padding: ReactiveProperty<Padding>,
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
        }
    }

    pub fn set_background_color(
        mut self,
        value: impl Into<ReactiveProperty<BackgroundColor>>,
    ) -> Self {
        self.background_color = value.into();
        self
    }

    pub fn set_border_color(mut self, value: impl Into<ReactiveProperty<BorderColor>>) -> Self {
        self.border_color = value.into();
        self
    }

    pub fn set_border_radius(mut self, value: impl Into<ReactiveProperty<BorderRadius>>) -> Self {
        self.border_radius = value.into();
        self
    }

    pub fn set_color(mut self, value: impl Into<ReactiveProperty<Color>>) -> Self {
        self.color = value.into();
        self
    }

    pub fn set_text_color(mut self, value: impl Into<ReactiveProperty<TextColor>>) -> Self {
        self.text_color = value.into();
        self
    }

    pub fn set_cursor(mut self, value: impl Into<ReactiveProperty<Cursor>>) -> Self {
        self.cursor = value.into();
        self
    }

    pub fn set_display(mut self, value: impl Into<ReactiveProperty<Display>>) -> Self {
        self.display = value.into();
        self
    }

    pub fn set_opacity(mut self, value: impl Into<ReactiveProperty<Opacity>>) -> Self {
        self.opacity = value.into();
        self
    }

    pub fn set_visibility(mut self, value: impl Into<ReactiveProperty<Visibility>>) -> Self {
        self.visibility = value.into();
        self
    }

    pub fn set_width(mut self, value: impl Into<ReactiveProperty<Width>>) -> Self {
        self.width = value.into();
        self
    }

    pub fn set_height(mut self, value: impl Into<ReactiveProperty<Height>>) -> Self {
        self.height = value.into();
        self
    }

    pub fn set_font_family(mut self, value: impl Into<ReactiveProperty<FontFamily>>) -> Self {
        self.font_family = value.into();
        self
    }

    pub fn set_font_size(mut self, value: impl Into<ReactiveProperty<FontSize>>) -> Self {
        self.font_size = value.into();
        self
    }

    pub fn set_font_weight(mut self, value: impl Into<ReactiveProperty<FontWeight>>) -> Self {
        self.font_weight = value.into();
        self
    }

    pub fn set_z_index(mut self, value: impl Into<ReactiveProperty<ZIndex>>) -> Self {
        self.z_index = value.into();
        self
    }

    pub fn set_align_items(mut self, value: impl Into<ReactiveProperty<AlignItems>>) -> Self {
        self.align_items = value.into();
        self
    }

    pub fn set_align_self(mut self, value: impl Into<ReactiveProperty<AlignSelf>>) -> Self {
        self.align_self = value.into();
        self
    }

    pub fn set_border_style(mut self, value: impl Into<ReactiveProperty<BorderStyle>>) -> Self {
        self.border_style = value.into();
        self
    }

    pub fn set_border_width(mut self, value: impl Into<ReactiveProperty<BorderWidth>>) -> Self {
        self.border_width = value.into();
        self
    }

    pub fn set_flex_basis(mut self, value: impl Into<ReactiveProperty<FlexBasis>>) -> Self {
        self.flex_basis = value.into();
        self
    }

    pub fn set_flex_direction(mut self, value: impl Into<ReactiveProperty<FlexDirection>>) -> Self {
        self.flex_direction = value.into();
        self
    }

    pub fn set_flex_grow(mut self, value: impl Into<ReactiveProperty<FlexGrow>>) -> Self {
        self.flex_grow = value.into();
        self
    }

    pub fn set_flex_shrink(mut self, value: impl Into<ReactiveProperty<FlexShrink>>) -> Self {
        self.flex_shrink = value.into();
        self
    }

    pub fn set_gap(mut self, value: impl Into<ReactiveProperty<Gap>>) -> Self {
        self.gap = value.into();
        self
    }

    pub fn set_justify_content(
        mut self,
        value: impl Into<ReactiveProperty<JustifyContent>>,
    ) -> Self {
        self.justify_content = value.into();
        self
    }

    pub fn set_margin(mut self, value: impl Into<ReactiveProperty<Margin>>) -> Self {
        self.margin = value.into();
        self
    }

    pub fn set_padding(mut self, value: impl Into<ReactiveProperty<Padding>>) -> Self {
        self.padding = value.into();
        self
    }

    pub fn compute(&self) -> ComputedStyles {
        let mut styles = ComputedStyles::new();
        styles.background_color = Some(self.background_color.get_untracked());
        styles.color = Some(self.color.get_untracked());
        styles.text_color = Some(self.text_color.get_untracked());
        styles.font_size = Some(self.font_size.get_untracked());
        styles.font_family = Some(self.font_family.get_untracked());
        styles.font_weight = Some(self.font_weight.get_untracked());
        styles.padding = Some(self.padding.get_untracked());
        styles.margin = Some(self.margin.get_untracked());
        styles.width = Some(self.width.get_untracked());
        styles.height = Some(self.height.get_untracked());
        styles.display = Some(self.display.get_untracked());
        styles.flex_direction = Some(self.flex_direction.get_untracked());
        styles.justify_content = Some(self.justify_content.get_untracked());
        styles.align_items = Some(self.align_items.get_untracked());
        styles.flex_grow = Some(self.flex_grow.get_untracked());
        styles.flex_shrink = Some(self.flex_shrink.get_untracked());
        styles.gap = Some(self.gap.get_untracked());
        styles.border_color = Some(self.border_color.get_untracked());
        styles.border_width = Some(self.border_width.get_untracked());
        styles.border_radius = Some(self.border_radius.get_untracked());
        styles.border_style = Some(self.border_style.get_untracked());
        styles.opacity = Some(self.opacity.get_untracked());
        styles.visibility = Some(self.visibility.get_untracked());
        styles.z_index = Some(self.z_index.get_untracked());
        styles
    }
}

impl From<ReactiveStyles> for ComputedStyles {
    fn from(styles: ReactiveStyles) -> Self {
        styles.compute()
    }
}
