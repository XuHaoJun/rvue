//! Form example demonstrating all input types

use rvue::prelude::*;
use rvue_macro::view;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create form view
    let form_view = create_form_view();

    // Run the application
    rvue::run_app(|| form_view)?;

    Ok(())
}

fn create_form_view() -> ViewStruct {
    // Create signals for form fields
    let (name, set_name) = create_signal(String::new());
    let (age, set_age) = create_signal(0.0);
    let (email, set_email) = create_signal(String::new());
    let (agree_to_terms, set_agree) = create_signal(false);
    let (selected_option, set_option) = create_signal("option1".to_string());

    let selected_option1 = selected_option.clone();
    let selected_option2 = selected_option.clone();
    let selected_option3 = selected_option.clone();

    let set_name_clone = set_name.clone();
    let set_age_clone = set_age.clone();
    let set_email_clone = set_email.clone();
    let set_agree_clone = set_agree.clone();
    let set_option_clone1 = set_option.clone();
    let set_option_clone2 = set_option.clone();
    let set_option_clone3 = set_option.clone();

    view! {
        <Flex direction="column" gap=15.0 align_items="start" justify_content="start">
            <Text content="User Registration Form" />

            <Flex direction="row" gap=10.0>
                <Text content="Name:" />
                <TextInput value=name.get() on_input={move |e: &rvue::event::status::InputEvent| set_name_clone.set(e.value.clone())} />
            </Flex>

            <Flex direction="row" gap=10.0>
                <Text content="Age:" />
                <NumberInput value=age.get() on_input={move |e: &rvue::event::status::InputEvent| set_age_clone.set(e.number_value)} />
            </Flex>

            <Flex direction="row" gap=10.0>
                <Text content="Email:" />
                <TextInput value=email.get() on_input={move |e: &rvue::event::status::InputEvent| set_email_clone.set(e.value.clone())} />
            </Flex>

            <Flex direction="row" gap=10.0>
                <Checkbox checked=agree_to_terms.get() on_change={move |e: &rvue::event::status::InputEvent| set_agree_clone.set(e.checked)} />
                <Text content="I agree to the terms" />
            </Flex>

            <Flex direction="column" gap=5.0>
                <Text content="Select Option:" />
                <Flex direction="row" gap=10.0>
                    <Radio value="option1" checked=selected_option1.get() == "option1" on_change={move |_e: &rvue::event::status::InputEvent| set_option_clone1.set("option1".to_string())} />
                    <Text content="Option 1" />
                </Flex>
                <Flex direction="row" gap=10.0>
                    <Radio value="option2" checked=selected_option2.get() == "option2" on_change={move |_e: &rvue::event::status::InputEvent| set_option_clone2.set("option2".to_string())} />
                    <Text content="Option 2" />
                </Flex>
                <Flex direction="row" gap=10.0>
                    <Radio value="option3" checked=selected_option3.get() == "option3" on_change={move |_e: &rvue::event::status::InputEvent| set_option_clone3.set("option3".to_string())} />
                    <Text content="Option 3" />
                </Flex>
            </Flex>

            <Button label="Submit" />
        </Flex>
    }
}
