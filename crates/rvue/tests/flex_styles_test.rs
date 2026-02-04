//! Unit tests for Flex widget styles

#[allow(deprecated)]
use rudo_gc::Gc;
use rvue::{Component, ComponentProps, ComponentType};
use rvue_style::{
    BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth, Color, ReactiveStyles,
};

#[test]
fn test_flex_with_background_color() {
    let styles = ReactiveStyles::new().set_background_color(BackgroundColor(Color::rgb(255, 0, 0)));
    let computed = styles.compute();

    let flex = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 0.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: Some(computed),
        },
    );

    match &*flex.props.borrow() {
        ComponentProps::Flex { styles: Some(s), .. } => {
            assert!(s.background_color.is_some(), "background_color should be set");
            let bg = s.background_color.as_ref().unwrap();
            assert_eq!(bg.0 .0.r, 255);
            assert_eq!(bg.0 .0.g, 0);
            assert_eq!(bg.0 .0.b, 0);
        }
        _ => panic!("Expected Flex props with styles"),
    };
}

#[test]
fn test_flex_with_border() {
    let styles = ReactiveStyles::new()
        .set_border_color(BorderColor(Color::rgb(0, 123, 255)))
        .set_border_width(BorderWidth(2.0))
        .set_border_style(BorderStyle::Solid)
        .set_border_radius(BorderRadius(4.0));
    let computed = styles.compute();

    let flex = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 0.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: Some(computed),
        },
    );

    match &*flex.props.borrow() {
        ComponentProps::Flex { styles: Some(s), .. } => {
            assert!(s.border_color.is_some(), "border_color should be set");
            assert!(s.border_width.is_some(), "border_width should be set");
            assert!(s.border_style.is_some(), "border_style should be set");
            assert!(s.border_radius.is_some(), "border_radius should be set");

            let border_color = s.border_color.as_ref().unwrap();
            assert_eq!(border_color.0 .0.r, 0);
            assert_eq!(border_color.0 .0.g, 123);
            assert_eq!(border_color.0 .0.b, 255);

            let border_width = s.border_width.as_ref().unwrap();
            assert_eq!(border_width.0, 2.0);

            let border_style = s.border_style.as_ref().unwrap();
            assert_eq!(*border_style, BorderStyle::Solid);

            let border_radius = s.border_radius.as_ref().unwrap();
            assert_eq!(border_radius.0, 4.0);
        }
        _ => panic!("Expected Flex props with styles"),
    };
}

#[test]
fn test_nested_flex_with_styles() {
    let outer_styles =
        ReactiveStyles::new().set_background_color(BackgroundColor(Color::rgb(250, 250, 250)));
    let outer_computed = outer_styles.compute();

    let inner_styles = ReactiveStyles::new()
        .set_background_color(BackgroundColor(Color::rgb(0, 123, 255)))
        .set_border_color(BorderColor(Color::rgb(255, 0, 0)))
        .set_border_width(BorderWidth(2.0))
        .set_border_style(BorderStyle::Solid);
    let inner_computed = inner_styles.compute();

    let outer = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 10.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: Some(outer_computed),
        },
    );

    let inner = Component::new(
        2,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 5.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
            styles: Some(inner_computed),
        },
    );

    outer.add_child(Gc::clone(&inner));

    match &*outer.props.borrow() {
        ComponentProps::Flex { styles: Some(s), .. } => {
            assert!(s.background_color.is_some(), "outer background_color should be set");
        }
        _ => panic!("Expected outer Flex props with styles"),
    };

    match &*inner.props.borrow() {
        ComponentProps::Flex { styles: Some(s), .. } => {
            assert!(s.background_color.is_some(), "inner background_color should be set");
            assert!(s.border_color.is_some(), "inner border_color should be set");
            assert!(s.border_width.is_some(), "inner border_width should be set");
            assert!(s.border_style.is_some(), "inner border_style should be set");
        }
        _ => panic!("Expected inner Flex props with styles"),
    };
}

#[test]
fn test_flex_without_styles() {
    let flex = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 0.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    match &*flex.props.borrow() {
        ComponentProps::Flex { styles: None, .. } => {
            // Expected - no styles
        }
        _ => panic!("Expected Flex props without styles"),
    };
}

#[test]
fn test_multiple_nested_flex_with_different_styles() {
    let parent_styles =
        ReactiveStyles::new().set_background_color(BackgroundColor(Color::rgb(255, 255, 255)));
    let parent_computed = parent_styles.compute();

    let child1_styles = ReactiveStyles::new()
        .set_background_color(BackgroundColor(Color::rgb(255, 0, 0)))
        .set_border_color(BorderColor(Color::rgb(200, 0, 0)))
        .set_border_width(BorderWidth(1.0))
        .set_border_style(BorderStyle::Solid);
    let child1_computed = child1_styles.compute();

    let child2_styles = ReactiveStyles::new()
        .set_background_color(BackgroundColor(Color::rgb(0, 255, 0)))
        .set_border_color(BorderColor(Color::rgb(0, 200, 0)))
        .set_border_width(BorderWidth(1.0))
        .set_border_style(BorderStyle::Solid);
    let child2_computed = child2_styles.compute();

    let parent = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 10.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: Some(parent_computed),
        },
    );

    let child1 = Component::new(
        2,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 5.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
            styles: Some(child1_computed),
        },
    );

    let child2 = Component::new(
        3,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 5.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
            styles: Some(child2_computed),
        },
    );

    parent.add_child(Gc::clone(&child1));
    parent.add_child(Gc::clone(&child2));

    // Verify parent styles
    match &*parent.props.borrow() {
        ComponentProps::Flex { styles: Some(s), .. } => {
            assert!(s.background_color.is_some());
        }
        _ => panic!("Expected parent Flex props with styles"),
    };

    // Verify child1 styles (red)
    match &*child1.props.borrow() {
        ComponentProps::Flex { styles: Some(s), .. } => {
            assert!(s.background_color.is_some());
            assert!(s.border_color.is_some());
            let bg = s.background_color.as_ref().unwrap();
            assert_eq!(bg.0 .0.r, 255); // Red
        }
        _ => panic!("Expected child1 Flex props with styles"),
    };

    // Verify child2 styles (green)
    match &*child2.props.borrow() {
        ComponentProps::Flex { styles: Some(s), .. } => {
            assert!(s.background_color.is_some());
            assert!(s.border_color.is_some());
            let bg = s.background_color.as_ref().unwrap();
            assert_eq!(bg.0 .0.g, 255); // Green
        }
        _ => panic!("Expected child2 Flex props with styles"),
    };
}
