use rvue_macro::view;

fn main() {
    // Invalid attribute syntax should fail
    let _view = view! {
        <Text content= />
    };
}
