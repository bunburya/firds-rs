//! Code for parsing structs from the XML files published by ESMA or the FCA.

use crate::xml::error::XmlError;
pub(crate) use crate::xml::from_xml::FromXml;
pub(crate) use crate::xml::iter_xml::XmlIterator;
use crate::ReferenceData;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod from_xml;
mod iter_xml;
mod error;
mod parse_utils;

pub struct IterRefData<'a> {
    xml_iterator: XmlIterator<'a, BufReader<File>>,
}

impl<'a> IterRefData<'a> {
    pub(crate) fn new(path: &Path) -> Result<Self, XmlError> {
        Ok(Self {
            xml_iterator: XmlIterator::from_file(vec!["RefData"], path)?,
        })
    }
}

impl Iterator for IterRefData<'_> {
    type Item = Result<ReferenceData, XmlError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.xml_iterator.next()? {
            Ok(elem) => Some(ReferenceData::from_xml(&elem)),
            Err(e) => Some(Err(e)),
        }
    }
}

