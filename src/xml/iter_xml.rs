use crate::xml::error::XmlError;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A struct describing an XML element.
#[derive(Debug, Default)]
pub(crate) struct Element {
    pub(crate) local_name: String,
    pub(crate) namespace: Option<String>,
    pub(crate) attributes: HashMap<String, String>,
    pub(crate) children: Vec<Element>,
    pub(crate) text: String,
}

impl Element {
    /// Search for the first immediate child [`Element`] with the given tag name, or return `None`
    /// if no such child is present.
    pub(crate) fn find_child(&self, tag_name: &str) -> Option<&Element> {
        self.children.iter().find(|&child| child.local_name == tag_name)
    }

    /// Find the first descendant of this element with the given tag name, if any, searching
    /// recursively. If `left_only` is true, only the leftmost branch of the tree is searched (ie,
    /// the first child of each element).
    pub(crate) fn find_descendant(&self, tag_name: &str, left_only: bool) -> Option<&Element> {
        if left_only {
            let child = self.children.first()?;
            if child.local_name == tag_name {
                Some(child)
            } else {
                child.find_descendant(tag_name, left_only)
            }
        } else {
            for child in &self.children {
                if let Some(c) = child.find_descendant(tag_name, left_only) {
                    return Some(c);
                }
            }
            None
        }
    }

    /// Return the first immediate child [`Element`] with the given tag name. Return an error if no
    /// such child is present.
    pub(crate) fn get_child(&self, tag_name: &str) -> Result<&Element, XmlError> {
        self.find_child(tag_name).ok_or(XmlError::ElementNotFound)
    }

    /// Search for the given attribute and return it or `None`.
    pub(crate) fn find_attr(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Return the value for the attribute with the given name. Return an error if no such attribute
    /// is present.
    pub(crate) fn get_attr(&self, key: &str) -> Result<&String, XmlError> {
        self.find_attr(key).ok_or(XmlError::AttributeNotFound)
    }

    /// Return the first child element, or `None`, if the element has no children.
    pub(crate) fn find_first_child(&self) -> Option<&Element> {
        self.children.first()
    }

    /// Return the first child element, or an error if the element has no children.
    pub(crate) fn get_first_child(&self) -> Result<&Element, XmlError> {
        self.find_first_child().ok_or(XmlError::ElementNotFound)
    }

    /// Return an iterator over the element's children.
    pub(crate) fn iter_children(&self) -> impl Iterator<Item = &Element> {
        self.children.iter()
    }
}

pub(crate) struct XmlIterator<'a, R> {
    tag_names: HashSet<&'a str>,
    reader: NsReader<R>,
}

impl<'a, R: BufRead> XmlIterator<'a, R> {
    pub fn new(tag_names: impl IntoIterator<Item = &'a str>, reader: R) -> Self {
        XmlIterator {
            tag_names: HashSet::from_iter(tag_names),
            reader: NsReader::from_reader(reader),
        }
    }

    pub fn parse_start(&mut self, start: BytesStart) -> Result<Element, XmlError> {
        let mut buf = Vec::new();
        let (resolve_res, local_name) = self.reader.resolve_element(start.name());
        let namespace = match resolve_res {
            ResolveResult::Bound(ns) => Some(String::from_utf8_lossy(ns.into_inner()).to_string()),
            ResolveResult::Unbound => None,
            ResolveResult::Unknown(_) => None,
        };
        let attributes = start
            .attributes()
            .filter_map(|a| a.ok())
            .map(|a| {
                let key = String::from_utf8_lossy(a.key.as_ref()).to_string();
                let value = a.unescape_value().unwrap_or_else(|_| "".into()).to_string();
                (key, value)
            })
            .collect();

        let mut children = Vec::new();
        let mut text: Option<String> = None;

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let child = self.parse_start(e)?;
                    children.push(child);
                }
                Ok(Event::Text(e)) => {
                    let t = e.unescape().unwrap_or_else(|_| "".into());
                    if let Some(prev) = text {
                        text = Some(prev + t.as_ref())
                    } else {
                        text = Some(t.into_owned());
                    }
                }
                Ok(Event::End(e)) if e.name() == start.name() => break,
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(Element {
            local_name: String::from_utf8_lossy(local_name.into_inner()).to_string(),
            namespace,
            attributes,
            children,
            text: text.unwrap_or_default(),
        })
    }
}

impl<'a, R: BufRead> Iterator for XmlIterator<'a, R> {
    type Item = Result<Element, XmlError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::new();
        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let elem_name = e.name();
                    let tag_name = String::from_utf8_lossy(elem_name.as_ref());
                    if self.tag_names.contains(tag_name.as_ref()) {
                        return Some(self.parse_start(e));
                    }
                }
                Ok(Event::Eof) => return None,
                Err(e) => panic!("Error: {:?}", e),
                _ => {}
            }
        }
    }
}

impl <'a> XmlIterator<'a, BufReader<File>> {
    pub fn from_file(tag_names: impl IntoIterator<Item = &'a str>, fpath: &Path) -> Result<Self, XmlError> {
        let file = File::open(fpath)?;
        let buf_reader = BufReader::new(file);
        Ok(Self::new(tag_names, buf_reader))
    }
}