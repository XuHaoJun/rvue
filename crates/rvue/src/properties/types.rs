//! Built-in widget properties.
//!
//! These properties represent widget-specific state that is not CSS-styling.
//! They are orthogonal to styling properties and can be combined freely.

use crate::properties::WidgetProperty;
use rvue_style::ComputedStyles;

#[derive(Clone, Debug, PartialEq)]
pub struct TextContent(pub String);

impl WidgetProperty for TextContent {
    fn static_default() -> &'static Self {
        static DEFAULT: TextContent = TextContent(String::new());
        &DEFAULT
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct ShowCondition(pub bool);

impl WidgetProperty for ShowCondition {
    fn static_default() -> &'static Self {
        static DEFAULT: ShowCondition = ShowCondition(true);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForItemCount(pub usize);

impl WidgetProperty for ForItemCount {
    fn static_default() -> &'static Self {
        static DEFAULT: ForItemCount = ForItemCount(0);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputValue(pub String);

impl WidgetProperty for TextInputValue {
    fn static_default() -> &'static Self {
        static DEFAULT: TextInputValue = TextInputValue(String::new());
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NumberInputValue(pub f64);

impl WidgetProperty for NumberInputValue {
    fn static_default() -> &'static Self {
        static DEFAULT: NumberInputValue = NumberInputValue(0.0);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckboxChecked(pub bool);

impl WidgetProperty for CheckboxChecked {
    fn static_default() -> &'static Self {
        static DEFAULT: CheckboxChecked = CheckboxChecked(false);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadioValue(pub String);

impl WidgetProperty for RadioValue {
    fn static_default() -> &'static Self {
        static DEFAULT: RadioValue = RadioValue(String::new());
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadioChecked(pub bool);

impl WidgetProperty for RadioChecked {
    fn static_default() -> &'static Self {
        static DEFAULT: RadioChecked = RadioChecked(false);
        &DEFAULT
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct FlexGap(pub f32);

impl WidgetProperty for FlexGap {
    fn static_default() -> &'static Self {
        static DEFAULT: FlexGap = FlexGap(0.0);
        &DEFAULT
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct FlexJustifyContent(pub String);

impl WidgetProperty for FlexJustifyContent {
    fn static_default() -> &'static Self {
        static DEFAULT: std::sync::LazyLock<FlexJustifyContent> =
            std::sync::LazyLock::new(|| FlexJustifyContent("flex-start".to_string()));
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonLabel(pub String);

impl WidgetProperty for ButtonLabel {
    fn static_default() -> &'static Self {
        static DEFAULT: ButtonLabel = ButtonLabel(String::new());
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderValue(pub f64);

impl WidgetProperty for SliderValue {
    fn static_default() -> &'static Self {
        static DEFAULT: SliderValue = SliderValue(0.0);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderMin(pub f64);

impl WidgetProperty for SliderMin {
    fn static_default() -> &'static Self {
        static DEFAULT: SliderMin = SliderMin(0.0);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderMax(pub f64);

impl WidgetProperty for SliderMax {
    fn static_default() -> &'static Self {
        static DEFAULT: SliderMax = SliderMax(100.0);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderStep(pub f64);

impl WidgetProperty for SliderStep {
    fn static_default() -> &'static Self {
        static DEFAULT: SliderStep = SliderStep(1.0);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwitchChecked(pub bool);

impl WidgetProperty for SwitchChecked {
    fn static_default() -> &'static Self {
        static DEFAULT: SwitchChecked = SwitchChecked(false);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaceholderText(pub String);

impl WidgetProperty for PlaceholderText {
    fn static_default() -> &'static Self {
        static DEFAULT: PlaceholderText = PlaceholderText(String::new());
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageSource(pub String);

impl WidgetProperty for ImageSource {
    fn static_default() -> &'static Self {
        static DEFAULT: ImageSource = ImageSource(String::new());
        &DEFAULT
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressValue(pub f64);

impl WidgetProperty for ProgressValue {
    fn static_default() -> &'static Self {
        static DEFAULT: ProgressValue = ProgressValue(0.0);
        &DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressMax(pub f64);

impl WidgetProperty for ProgressMax {
    fn static_default() -> &'static Self {
        static DEFAULT: ProgressMax = ProgressMax(100.0);
        &DEFAULT
    }
}
