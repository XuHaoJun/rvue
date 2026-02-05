//! Built-in widget properties.
//!
//! These properties represent widget-specific state that is not CSS-styling.
//! They are orthogonal to styling properties and can be combined freely.

use crate::properties::WidgetProperty;
use rudo_gc::{Trace, Visitor};
use rvue_style::ComputedStyles;

#[derive(Clone, Debug, PartialEq)]
pub struct TextContent(pub String);

impl WidgetProperty for TextContent {
    fn static_default() -> &'static Self {
        static DEFAULT: TextContent = TextContent(String::new());
        &DEFAULT
    }
}

unsafe impl Trace for TextContent {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetStyles(pub ComputedStyles);

impl WidgetProperty for WidgetStyles {
    fn static_default() -> &'static Self {
        static DEFAULT: std::sync::LazyLock<WidgetStyles> =
            std::sync::LazyLock::new(|| WidgetStyles(ComputedStyles::default()));
        &DEFAULT
    }
}

unsafe impl Trace for WidgetStyles {
    fn trace(&self, visitor: &mut impl Visitor) {
        self.0.trace(visitor);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShowCondition(pub bool);

impl WidgetProperty for ShowCondition {
    fn static_default() -> &'static Self {
        static DEFAULT: ShowCondition = ShowCondition(true);
        &DEFAULT
    }
}

unsafe impl Trace for ShowCondition {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForItemCount(pub usize);

impl WidgetProperty for ForItemCount {
    fn static_default() -> &'static Self {
        static DEFAULT: ForItemCount = ForItemCount(0);
        &DEFAULT
    }
}

unsafe impl Trace for ForItemCount {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputValue(pub String);

impl WidgetProperty for TextInputValue {
    fn static_default() -> &'static Self {
        static DEFAULT: TextInputValue = TextInputValue(String::new());
        &DEFAULT
    }
}

unsafe impl Trace for TextInputValue {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct NumberInputValue(pub f64);

impl WidgetProperty for NumberInputValue {
    fn static_default() -> &'static Self {
        static DEFAULT: NumberInputValue = NumberInputValue(0.0);
        &DEFAULT
    }
}

unsafe impl Trace for NumberInputValue {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckboxChecked(pub bool);

impl WidgetProperty for CheckboxChecked {
    fn static_default() -> &'static Self {
        static DEFAULT: CheckboxChecked = CheckboxChecked(false);
        &DEFAULT
    }
}

unsafe impl Trace for CheckboxChecked {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadioValue(pub String);

impl WidgetProperty for RadioValue {
    fn static_default() -> &'static Self {
        static DEFAULT: RadioValue = RadioValue(String::new());
        &DEFAULT
    }
}

unsafe impl Trace for RadioValue {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadioChecked(pub bool);

impl WidgetProperty for RadioChecked {
    fn static_default() -> &'static Self {
        static DEFAULT: RadioChecked = RadioChecked(false);
        &DEFAULT
    }
}

unsafe impl Trace for RadioChecked {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlexDirection(pub String);

impl WidgetProperty for FlexDirection {
    fn static_default() -> &'static Self {
        static DEFAULT: std::sync::LazyLock<FlexDirection> =
            std::sync::LazyLock::new(|| FlexDirection("row".to_string()));
        &DEFAULT
    }
}

unsafe impl Trace for FlexDirection {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlexGap(pub f32);

impl WidgetProperty for FlexGap {
    fn static_default() -> &'static Self {
        static DEFAULT: FlexGap = FlexGap(0.0);
        &DEFAULT
    }
}

unsafe impl Trace for FlexGap {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlexAlignItems(pub String);

impl WidgetProperty for FlexAlignItems {
    fn static_default() -> &'static Self {
        static DEFAULT: std::sync::LazyLock<FlexAlignItems> =
            std::sync::LazyLock::new(|| FlexAlignItems("stretch".to_string()));
        &DEFAULT
    }
}

unsafe impl Trace for FlexAlignItems {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlexJustifyContent(pub String);

impl WidgetProperty for FlexJustifyContent {
    fn static_default() -> &'static Self {
        static DEFAULT: std::sync::LazyLock<FlexJustifyContent> =
            std::sync::LazyLock::new(|| FlexJustifyContent("flex-start".to_string()));
        &DEFAULT
    }
}

unsafe impl Trace for FlexJustifyContent {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonLabel(pub String);

impl WidgetProperty for ButtonLabel {
    fn static_default() -> &'static Self {
        static DEFAULT: ButtonLabel = ButtonLabel(String::new());
        &DEFAULT
    }
}

unsafe impl Trace for ButtonLabel {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderValue(pub f64);

impl WidgetProperty for SliderValue {
    fn static_default() -> &'static Self {
        static DEFAULT: SliderValue = SliderValue(0.0);
        &DEFAULT
    }
}

unsafe impl Trace for SliderValue {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderMin(pub f64);

impl WidgetProperty for SliderMin {
    fn static_default() -> &'static Self {
        static DEFAULT: SliderMin = SliderMin(0.0);
        &DEFAULT
    }
}

unsafe impl Trace for SliderMin {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderMax(pub f64);

impl WidgetProperty for SliderMax {
    fn static_default() -> &'static Self {
        static DEFAULT: SliderMax = SliderMax(100.0);
        &DEFAULT
    }
}

unsafe impl Trace for SliderMax {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderStep(pub f64);

impl WidgetProperty for SliderStep {
    fn static_default() -> &'static Self {
        static DEFAULT: SliderStep = SliderStep(1.0);
        &DEFAULT
    }
}

unsafe impl Trace for SliderStep {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwitchChecked(pub bool);

impl WidgetProperty for SwitchChecked {
    fn static_default() -> &'static Self {
        static DEFAULT: SwitchChecked = SwitchChecked(false);
        &DEFAULT
    }
}

unsafe impl Trace for SwitchChecked {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaceholderText(pub String);

impl WidgetProperty for PlaceholderText {
    fn static_default() -> &'static Self {
        static DEFAULT: PlaceholderText = PlaceholderText(String::new());
        &DEFAULT
    }
}

unsafe impl Trace for PlaceholderText {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageSource(pub String);

impl WidgetProperty for ImageSource {
    fn static_default() -> &'static Self {
        static DEFAULT: ImageSource = ImageSource(String::new());
        &DEFAULT
    }
}

unsafe impl Trace for ImageSource {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageFit(pub String);

impl WidgetProperty for ImageFit {
    fn static_default() -> &'static Self {
        static DEFAULT: std::sync::LazyLock<ImageFit> =
            std::sync::LazyLock::new(|| ImageFit("fill".to_string()));
        &DEFAULT
    }
}

unsafe impl Trace for ImageFit {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressValue(pub f64);

impl WidgetProperty for ProgressValue {
    fn static_default() -> &'static Self {
        static DEFAULT: ProgressValue = ProgressValue(0.0);
        &DEFAULT
    }
}

unsafe impl Trace for ProgressValue {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressMax(pub f64);

impl WidgetProperty for ProgressMax {
    fn static_default() -> &'static Self {
        static DEFAULT: ProgressMax = ProgressMax(100.0);
        &DEFAULT
    }
}

unsafe impl Trace for ProgressMax {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}
