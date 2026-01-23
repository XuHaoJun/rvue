use rvue_macro::view;

fn main() {
    // Unclosed tag should fail
    let _view = view! {
        <Text content="Hello"
    };
}
