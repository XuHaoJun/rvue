//! Widget property trait definition.

/// A property that can be stored on a widget.
///
/// This trait extends the CSS-focused `rvue_style::Property` with
/// additional requirements for widget properties (like being traceable
/// by the garbage collector).
///
/// # Example
///
/// ```rust
/// use rvue::properties::WidgetProperty;
///
/// #[derive(Clone, Debug)]
/// pub struct TextContent(pub String);
///
/// impl WidgetProperty for TextContent {
///     fn static_default() -> &'static Self {
///         static DEFAULT: TextContent = TextContent(String::new());
///         &DEFAULT
///     }
/// }
/// ```
pub trait WidgetProperty: Clone + Send + Sync + 'static {
    /// Returns a static reference to the default value for this property.
    ///
    /// This is used when a property is not explicitly set on a widget.
    /// Using a static ensures we don't need to allocate for default values.
    fn static_default() -> &'static Self;
}
