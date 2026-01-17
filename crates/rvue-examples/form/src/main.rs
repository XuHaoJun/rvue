//! Form example demonstrating all input types

use rvue::prelude::*;
use rvue::{Component, ComponentType, ComponentProps, ViewStruct, Flex, FlexDirection, AlignItems, JustifyContent};
use rvue::{TextInput, NumberInput, Checkbox, Radio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create form view
    let form_view = create_form_view();
    
    // Run the application
    rvue::run_app(|| form_view)?;
    
    Ok(())
}

fn create_form_view() -> ViewStruct {
    // Create signals for form fields
    let (name, _set_name) = create_signal(String::new());
    let (age, _set_age) = create_signal(0.0);
    let (email, _set_email) = create_signal(String::new());
    let (agree_to_terms, _set_agree) = create_signal(false);
    let (selected_option, _set_option) = create_signal("option1".to_string());
    
    // Create root component with column layout
    let root = Flex::new(
        0,
        FlexDirection::Column,
        15.0,
        AlignItems::Start,
        JustifyContent::Start,
    );
    
    // Create form title
    let _title = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text {
            content: "User Registration Form".to_string(),
        },
    );
    
    // Create name input
    let _name_input = TextInput::new(
        2,
        name.get(),
    );
    
    // Create age input
    let _age_input = NumberInput::new(
        3,
        age.get(),
    );
    
    // Create email input
    let _email_input = TextInput::new(
        4,
        email.get(),
    );
    
    // Create checkbox for terms agreement
    let _terms_checkbox = Checkbox::new(
        5,
        agree_to_terms.get(),
    );
    
    // Create radio buttons for options
    let _radio1 = Radio::new(6, "option1".to_string(), selected_option.get() == "option1");
    let _radio2 = Radio::new(7, "option2".to_string(), selected_option.get() == "option2");
    let _radio3 = Radio::new(8, "option3".to_string(), selected_option.get() == "option3");
    
    // Create submit button
    let _submit_button = Component::new(
        9,
        ComponentType::Button,
        ComponentProps::Button {
            label: "Submit".to_string(),
        },
    );
    
    // Note: In a full implementation, we would:
    // 1. Use the view! macro to create components declaratively
    // 2. Connect event handlers (on_input, on_change) to update signals
    // 3. Use effects to update input values when signals change
    // 4. Add components to the root component's children
    // For MVP, this demonstrates the basic structure
    
    // Create view
    let view = ViewStruct::new(root);
    
    // Add effects to track form field changes
    let _name_effect = create_effect({
        let name = name.clone();
        move || {
            let _ = name.get();
            println!("Name changed: {}", name.get());
        }
    });
    
    let _age_effect = create_effect({
        let age = age.clone();
        move || {
            let _ = age.get();
            println!("Age changed: {}", age.get());
        }
    });
    
    view
}
