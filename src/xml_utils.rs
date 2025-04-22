use crate::error::ParseError;
use quick_xml::events::{BytesStart, Event};
use quick_xml::NsReader;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Element {
    name: String,
    attributes: HashMap<String, String>,
    children: Vec<Element>,
    pub(crate) text: String,
}

impl Element {
    pub fn parse<R: std::io::BufRead>(
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
                    let child = Element::parse(reader, e)?;
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
    
    pub(crate) fn find(&self, tag_name: &str) -> Option<&Element> {
        for child in &self.children {
            if child.name == tag_name {
                return Some(child)
            }
        }
        None
    }
    
    pub(crate) fn get(&self, tag_name: &str) -> Result<&Element, ParseError> {
        self.find(tag_name).ok_or(ParseError::ElementNotFound)
    }
}

/// Searches for an immediate child element with the given name. Returns a reference to the element
/// if present. If `elem` is `None` or the child element is not found, returns `None`.
pub(crate) fn child_or_none<'a>(elem: Option<&'a Element>, child_name: &str) -> Option<&'a Element> {
    elem?.find(child_name)
}

pub(crate) fn text_or_none(elem: Option<&Element>) -> Option<&str> {
    if let Some(elem) = elem {
        Some(&elem.text)
    } else {
        None
    }
}