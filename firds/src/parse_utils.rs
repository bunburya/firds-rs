use crate::error::ParseError;
use crate::iter_xml::Element;
use chrono::{DateTime, NaiveDate, Utc};
use std::str::FromStr;

/// Searches for an immediate child element with the given name. Returns a reference to the element
/// if present. If `elem` is `None` or the child element is not found, returns `None`.
pub(crate) fn child_or_none<'a>(elem: Option<&'a Element>, child_name: &str) -> Option<&'a Element> {
    elem?.find_child(child_name)
}

pub(crate) fn text_or_none(elem: Option<&Element>) -> Option<&str> {
    if let Some(elem) = elem {
        Some(&elem.text)
    } else {
        None
    }
}

pub(crate) fn datetime_or_none(elem: Option<&Element>) -> Result<Option<DateTime<Utc>>, ParseError> {
    if let Some(elem) = elem {
        match DateTime::parse_from_rfc3339(&elem.text) {
            Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
            Err(e) => Err(ParseError::DateTime(e))
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn date_or_none(elem: Option<&Element>) -> Result<Option<NaiveDate>, ParseError> {
    if let Some(elem) = elem {
        match NaiveDate::parse_from_str(&elem.text[..10], "%Y-%m-%d") {
            Ok(date) => Ok(Some(date)),
            Err(e) => Err(ParseError::DateTime(e))
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn parse_or_none<T: FromStr>(elem: Option<&Element>) -> Result<Option<T>, ParseError> 
where ParseError: From<<T as FromStr>::Err> {
    if let Some(elem) = elem {
        Ok(Some(elem.text.parse::<T>()?))
    } else {
        Ok(None)
    }
}