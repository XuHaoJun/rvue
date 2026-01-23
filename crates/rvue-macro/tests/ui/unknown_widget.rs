use rvue_macro::view;

fn main() {
    // Unknown widget type should fail to compile
    let _view = view! {
        <UnknownWidget />
    };
}
