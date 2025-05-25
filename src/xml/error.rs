use crate::xml::error::ProductParseError::BadSubProduct;
use quick_xml::events::attributes::AttrError;
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

#[derive(Debug)]
pub enum ProductParseError {
    NoSubProduct,
    BadSubProduct,
    BadProduct
}

impl From<strum::ParseError> for ProductParseError {
    fn from(_: strum::ParseError) -> Self {
        BadSubProduct
    }
}

#[derive(Debug)]
pub enum ParseError {
    /// Error parsing XML attributes.
    Attr(AttrError),
    /// Error parsing XML using the `quick_xml` crate.
    QuickXml(quick_xml::Error),
    /// Error parsing an integer from a string.
    Int(ParseIntError),
    /// Error parsing a float from a string.
    Float(ParseFloatError),
    /// Error parsing a bool from a string.
    Bool(ParseBoolError),
    /// Error parsing a [`chrono::DateTime`] from a string.
    DateTime(chrono::ParseError),
    /// Could not find the desired [`Element`].
    ElementNotFound,
    /// Found an element we did not expect.
    UnexpectedElement,
    /// Element did not have the expected attribute
    AttributeNotFound,
    /// Element had no text where some was expected.
    TextNotFound,
    /// Something returned [`None`] when we expected [`Some`].
    NoneFound,
    /// Error constructing a `firds` struct.
    Firds(crate::ParseError)
}

impl From<AttrError> for ParseError {
    fn from(e: AttrError) -> Self {
        Self::Attr(e)
    }
}

impl From<quick_xml::Error> for ParseError {
    fn from(e: quick_xml::Error) -> Self {
        Self::QuickXml(e)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> Self {
        Self::Int(e)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(e: ParseFloatError) -> Self {
        Self::Float(e)
    }
}

impl From<ParseBoolError> for ParseError {
    fn from(e: ParseBoolError) -> Self {
        Self::Bool(e)
    }
}

impl From<chrono::ParseError> for ParseError {
    fn from(e: chrono::ParseError) -> Self {
        Self::DateTime(e)
    }
}

impl From<crate::ParseError> for ParseError {
    fn from(e: crate::ParseError) -> Self {
        Self::Firds(e)
    }
}

impl From<crate::ProductError> for ParseError {
    fn from(e: crate::ProductError) -> Self {
        Self::Firds(crate::ParseError::from(e))
    }
}

impl From<strum::ParseError> for ParseError {
    fn from(e: strum::ParseError) -> Self {
        Self::Firds(crate::ParseError::from(e))
    }
}

