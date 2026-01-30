//! Background properties.

use super::color::Color;
use crate::property::Property;
use rudo_gc::Trace;

/// Background color property.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct BackgroundColor(pub Color);

impl Property for BackgroundColor {}
