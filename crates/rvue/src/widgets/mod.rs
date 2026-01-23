//! Built-in widget components

pub mod button;
pub mod checkbox;
pub mod flex;
pub mod for_loop;
pub mod input;
pub mod radio;
pub mod show;
pub mod text;

// New widget builders
pub use button::ButtonWidget;
pub use checkbox::CheckboxWidget;
pub use flex::FlexWidget;
pub use for_loop::ForWidget;
pub use input::{NumberInputWidget, TextInputWidget};
pub use radio::RadioWidget;
pub use show::ShowWidget;
pub use text::TextWidget;

// Keep old API for backward compatibility
#[allow(deprecated)]
pub use button::Button;
#[allow(deprecated)]
pub use checkbox::Checkbox;
#[allow(deprecated)]
pub use flex::Flex;
#[allow(deprecated)]
pub use for_loop::For;
#[allow(deprecated)]
pub use input::{NumberInput, TextInput};
#[allow(deprecated)]
pub use radio::Radio;
#[allow(deprecated)]
pub use show::Show;
#[allow(deprecated)]
pub use text::Text;
