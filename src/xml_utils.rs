use crate::error::ParseError;
use chrono::{DateTime, NaiveDate, Utc};
use quick_xml::events::{BytesStart, Event};
use quick_xml::NsReader;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub(crate) struct Element {
    pub(crate) name: String,
    attributes: HashMap<String, String>,
    children: Vec<Element>,
    pub(crate) text: String,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            attributes: HashMap::new(),
            children: vec![],
            text: "".to_string(),
        }
    }
}

impl Element {
    pub fn parse_start<R: std::io::BufRead>(
        reader: &mut NsReader<R>,
        start: BytesStart,
    ) -> Result<Self, ParseError> {
        let mut buf = Vec::new();
        let name = String::from_utf8_lossy(start.name().as_ref()).to_string();
        let attributes = start.attributes()
            .filter_map(|a| a.ok())
            .map(|a| {
                let key = String::from_utf8_lossy(a.key.as_ref()).to_string();
                let value = a
                    .unescape_value()
                    .unwrap_or_else(|_| "".into())
                    .to_string();
                (key, value)
            })
            .collect();

        let mut children = Vec::new();
        let mut text: Option<String> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let child = Element::parse_start(reader, e)?;
                    children.push(child);
                },
                Ok(Event::Text(e)) => {
                    let t = e.unescape().unwrap_or_else(|_| "".into());
                    if let Some(prev) = text {
                        text = Some(prev + t.as_ref())
                    } else {
                        text = Some(t.into_owned());
                    }
                },
                Ok(Event::End(e)) if e.name() == start.name() => break,
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(Element {
            name,
            attributes,
            children,
            text: text.unwrap_or_default(),
        })
    }
    
    /// Search for the first immediate child [`Element`] with the given tag name, or return `None`
    /// if no such child is present.
    pub(crate) fn find_child(&self, tag_name: &str) -> Option<&Element> {
        for child in &self.children {
            if child.name == tag_name {
                return Some(child)
            }
        }
        None
    }
    
    /// Find the first descendant of this element with the given tag name, if any, searching
    /// recursively. If `left_only` is true, only the leftmost branch of the tree is searched (ie,
    /// the first child of each element).
    pub(crate) fn find_descendant(&self, tag_name: &str, left_only: bool) -> Option<&Element> {
        if left_only {
            let child = self.children.first()?;
            if child.name == tag_name {
                Some(child)
            } else {
                child.find_descendant(tag_name, left_only)
            }
        } else {
            for child in &self.children {
                if let Some(c) = child.find_descendant(tag_name, left_only) {
                    return Some(c)
                }
            }
            None
        }
    }
    
    /// Return the first immediate child [`Element`] with the given tag name. Return an error if no
    /// such child is present.
    pub(crate) fn get_child(&self, tag_name: &str) -> Result<&Element, ParseError> {
        self.find_child(tag_name).ok_or(ParseError::ElementNotFound)
    }
    
    /// Search for the given attribute and return it or `None`.
    pub(crate) fn find_attr(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }
    
    /// Return the value for the attribute with the given name. Return an error if no such attribute
    /// is present.
    pub(crate) fn get_attr(&self, key: &str) -> Result<&String, ParseError> {
        self.find_attr(key).ok_or(ParseError::AttributeNotFound)
    }
    
    /// Return the first child element, or `None`, if the element has no children.
    pub(crate) fn find_first_child(&self) -> Option<&Element> {
        self.children.first()
    }
    
    /// Return the first child element, or an error if the element has no children.
    pub(crate) fn get_first_child(&self) -> Result<&Element, ParseError> {
        self.find_first_child().ok_or(ParseError::ElementNotFound)
    }
    
    /// Return an iterator over the element's children.
    pub(crate) fn iter_children(&self) -> impl Iterator<Item = &Element> {
        self.children.iter()
    }
}

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