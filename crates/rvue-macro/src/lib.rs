//! Procedural macros for Rvue framework

mod ast;
mod attrs;
mod codegen;
mod parser;
mod widgets;

use codegen::{convert_rstml_to_rvue, generate_view_code};
use parser::{parse_global_class, parse_view, strip_global_class};
use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

/// The `view!` macro provides HTML-like syntax for creating UI components
///
/// # Basic Usage
///
/// ```ignore
/// use rvue::prelude::*;
///
/// view! {
///     <Text value="Hello, World!" />
///     <Button on_click=|| println!("clicked")>"Click me"</Button>
///     <Flex direction="column" gap={20.0}>
///         <Text value={signal.get()} />
///     </Flex>
/// }
/// ```
///
/// # Supported Widgets
///
/// - `Text` - Display text with optional font_size and color
/// - `Button` - Interactive button with label
/// - `Flex` - Flexbox container with direction, gap, align_items, justify_content
/// - `TextInput` - Text input field
/// - `NumberInput` - Numeric input field
/// - `Checkbox` - Boolean checkbox
/// - `Radio` - Radio button
/// - `Show` - Conditional rendering
/// - `For` - List rendering
///
/// # Attributes
///
/// Static attributes: `attr="value"`
/// Dynamic attributes: `attr={expression}`
/// Event handlers: `on_event=handler`
///
/// # Examples
///
/// Static text widget:
/// ```ignore
/// view! { <Text value="Hello" /> }
/// ```
///
/// Button with click handler:
/// ```ignore
/// view! { <Button on_click=|| println!("clicked")>"Click me"</Button> }
/// ```
///
/// Flex container with children:
/// ```ignore
/// view! {
///     <Flex direction="column" gap={20.0}>
///         <Text value="Child 1" />
///         <Text value="Child 2" />
///     </Flex>
/// }
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.clone().into();

    let global_class = parse_global_class(input2.clone());
    let tokens = strip_global_class(input2, global_class.as_ref());

    let nodes = match parse_view(tokens) {
        Ok(nodes) => nodes,
        Err(e) => return e.into(),
    };

    let rvue_nodes: Vec<_> = nodes.iter().filter_map(|n| convert_rstml_to_rvue(n, None)).collect();

    let output = generate_view_code(rvue_nodes);

    quote::quote! {
        {
            use rvue::prelude::*;
            #output
        }
    }
    .into()
}

/// The `#[component]` macro marks a function as a component
///
/// Components are functions that return `impl View` and can be used
/// within `view!` macro as PascalCase tags.
///
/// # Basic Usage
///
/// ```ignore
/// #[component]
/// fn MyComponent() -> impl View {
///     view! { <Text value="Hello" /> }
/// }
///
/// #[component]
/// fn App() -> impl View {
///     view! {
///         <Flex direction="column">
///             <MyComponent />
///         </Flex>
///     }
/// }
/// ```
///
/// # Component Properties
///
/// Function parameters become component properties that can be passed
/// when using the component in `view!`.
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    quote::quote! {
        #input
    }
    .into()
}
