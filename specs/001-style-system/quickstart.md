# Quickstart: Rvue Style System

**Feature**: 001-style-system | **Date**: 2026-01-27

## Introduction

The rvue style system provides type-safe, CSS-compatible styling for GUI widgets. This guide covers basic usage patterns.

## Adding rvue-style Dependency

```toml
# Cargo.toml
[dependencies]
rvue-style = { path = "crates/rvue-style" }
```

## Basic Styling

### Using Property Builders

```rust
use rvue_style::prelude::*;

let style = Properties::new()
    .with(BackgroundColor(Color::rgb(255, 0, 0)))
    .with(Padding::uniform(16.0))
    .with(FontSize(14.0))
    .with(TextColor(Color::rgb(255, 255, 255)));
```

### Using StyledWidgetExt Trait

```rust
use rvue_style::prelude::*;

let button = Button::new("Click Me")
    .style_background(Color::rgb(0, 120, 215))
    .style_padding(Padding::uniform(12.0))
    .style_margin(Margin::all(8.0))
    .style_font_size(16.0)
    .style_color(Color::rgb(255, 255, 255));
```

## CSS Styling

### Creating a Stylesheet

```rust
use rvue_style::stylesheet;

let stylesheet = stylesheet(r#"
    button {
        background-color: #0078d7;
        padding: 12px;
        margin: 8px;
        font-size: 16px;
        color: white;
    }

    button:hover {
        background-color: #005a9e;
    }

    button:disabled {
        background-color: #cccccc;
        color: #666666;
    }

    .primary {
        background-color: #28a745;
    }

    .primary:hover {
        background-color: #218838;
    }

    #submit-btn {
        font-weight: bold;
    }
"#);
```

### Applying Stylesheet to Widget

```rust
let resolver = StyleResolver::new(stylesheet);

fn apply_styles(resolver: &StyleResolver, widget: &mut dyn Widget) {
    let element = RvueElement::new(widget);
    let computed = resolver.resolve(&element);

    // Apply computed styles to widget...
}
```

## State-Based Styling

### CSS Pseudo-Classes

The style system supports standard CSS pseudo-classes:

```css
/* User action pseudo-classes */
button:hover { }
button:focus { }
button:active { }
button:focus-visible { }

/* State pseudo-classes */
input:disabled { }
checkbox:checked { }
input:valid { }
input:invalid { }

/* Structural pseudo-classes */
div:first-child { }
div:last-child { }
div:nth-child(2n) { }
```

### ElementState Tracking

Widgets automatically track state for pseudo-class matching:

```rust
// Widget automatically tracks:
// - HOVER: mouse enters widget bounds
// - FOCUS: widget gains keyboard focus
// - ACTIVE: mouse button pressed over widget
// - DISABLED: widget is disabled
// - CHECKED: checkbox/radio is checked

// State is available for custom pseudo-class matching:
if widget.state().contains(ElementState::HOVER) {
    // Apply hover-specific styling
}
```

## Color Values

### Creating Colors

```rust
use rvue_style::Color;

// RGB
let red = Color::rgb(255, 0, 0);

// RGBA
let transparent_red = Color::rgba(255, 0, 0, 128);

// Hex
let blue = Color::from_hex("#0078d7").unwrap();

// Named colors (CSS spec)
let white = Color::from_str("white").unwrap();
```

### CSS Color Parsing

CSS colors are automatically parsed:

```css
background-color: #ff0000;
background-color: rgb(255, 0, 0);
background-color: rgba(255, 0, 0, 0.5);
background-color: red;
background-color: hsl(120, 100%, 50%);
```

## Layout Properties

### Flexbox

```rust
use rvue_style::{Display, FlexDirection, JustifyContent, AlignItems};

let flex_style = Properties::new()
    .with(Display::Flex)
    .with(FlexDirection::Row)
    .with(JustifyContent::SpaceBetween)
    .with(AlignItems::Center);
```

### Sizing

```rust
use rvue_style::{Width, Height, Size};

let sized = Properties::new()
    .with(Width(Size::pixels(200.0)))
    .with(Height(Size::auto()))
    .with(MinWidth(Size::pixels(100.0)))
    .with(MaxWidth(Size::percent(0.5)));
```

### Spacing

```rust
use rvue_style::{Padding, Margin};

let spaced = Properties::new()
    .with(Padding::uniform(16.0))          // All sides 16px
    .with(Padding::symmetric(8.0, 16.0))   // Vertical 8px, Horizontal 16px
    .with(Padding::all(1, 2, 3, 4))        // top, right, bottom, left
    .with(Margin::auto());                 // Auto margin
```

## Reactive Styles

### Signal-Based Properties

```rust
use rvue_style::ReactiveProperty;
use rvue::signal::{create_signal, ReadSignal};

// Create a signal for a color
let (color, set_color) = create_signal(Color::rgb(255, 0, 0));

// Use as reactive property
let reactive_style = Properties::new()
    .with(ReactiveProperty::Signal(color))
    .with(Padding::uniform(16.0));

// Change the signal to update the style
set_color(Color::rgb(0, 255, 0));  // Widget will update automatically
```

### Derived Styles

```rust
use rvue_style::derive_style;
use rvue::create_memo;

let base_style = ComputedStyles::default();
let derived_style = derive_style(base_style, |styles| {
    let mut derived = styles.clone();
    // Modify based on base style
    if styles.background_color == Some(BackgroundColor(Color::rgb(255, 0, 0))) {
        derived.insert(TextColor(Color::rgb(255, 255, 255)));
    }
    derived
});
```

## Widget Integration

### Implementing StyledWidget

```rust
use rvue_style::{Property, StyledWidget, Properties};

pub struct MyWidget {
    style: Properties,
    // ... other fields
}

impl StyledWidget for MyWidget {
    type Style = MyWidgetStyle;  // Marker type

    fn style(&self) -> &Properties {
        &self.style
    }

    fn set_style(&mut self, style: Properties) {
        self.style = style;
    }
}

pub struct MyWidgetStyle;

impl Property for MyWidgetStyle {
    fn static_default() -> &'static Self {
        static DEFAULT: MyWidgetStyle = MyWidgetStyle;
        &DEFAULT
    }
}
```

### Using in Views

```rust
use rvue::prelude::*;
use rvue_style::prelude::*;

fn my_view() -> impl View {
    let count = create_signal(0);

    v_stack![
        Button::new("Increment")
            .style_background(if *count.get() > 5 {
                Color::rgb(255, 0, 0)
            } else {
                Color::rgb(0, 120, 215)
            })
            .on_click(move || count.set(count.get() + 1))
    ]
}
```

## Best Practices

### 1. Use Static Defaults

Property types should use `static_default()` for zero-cost defaults:

```rust
impl Property for BackgroundColor {
    fn static_default() -> &'static Self {
        static DEFAULT: BackgroundColor = BackgroundColor(Color::TRANSPARENT);
        &DEFAULT
    }
}
```

### 2. Prefer Builder Pattern

Use builder-style API for fluent property setting:

```rust
widget
    .style_padding(Padding::uniform(8.0))
    .style_margin(Margin::vertical(4.0))
    .style_background(Color::rgb(255, 255, 255));
```

### 3. Use Named Colors for Accessibility

```rust
// Good - semantic color names
Button::new("Submit").style_background(Color::from_str("primary").unwrap());

// Avoid - magic numbers
Button::new("Submit").style_background(Color::rgb(0, 120, 215));
```

### 4. Group Related Properties

```rust
// Group typography properties
let text_style = Properties::new()
    .with(FontSize(16.0))
    .with(FontFamily("Inter".to_string()))
    .with(FontWeight::Bold)
    .with(TextColor(Color::rgb(0, 0, 0)));
```

## Performance Tips

### 1. Share ComputedStyles

```rust
// Create once, share across many widgets
let shared_style = ComputedStyles::default()
    .with(BackgroundColor(Color::rgb(240, 240, 240)))
    .shared();

let widget1 = Button::new("Button 1").with_base_style(shared_style.clone());
let widget2 = Button::new("Button 2").with_base_style(shared_style.clone());
```

### 2. Use Selective Updates

```rust
// Only update when specific properties change
impl MyWidget {
    fn update_style(&mut self, ctx: &mut UpdateCtx, changed: &Properties) {
        if changed.contains::<BackgroundColor>() {
            ctx.request_paint_only();
        }
        if changed.contains::<Width>() || changed.contains::<Height>() {
            ctx.request_layout();
        }
    }
}
```

## Next Steps

- See [API Reference](../api-reference.md) for complete documentation
- See [Examples](../examples/) for complete working examples
- See [data-model.md](data-model.md) for entity definitions
