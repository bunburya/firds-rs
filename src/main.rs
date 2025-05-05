use std::fs::File;
use std::io::BufReader;
use quick_xml::events::Event;
use quick_xml::NsReader;
use crate::error::ParseError;
use crate::model::{FromXml, IndexTerm};

mod xml_utils;
mod error;
mod model;
mod categories;
mod product;

fn main() -> Result<(), ParseError> {
    let file = File::open("/mnt/storage/alan/data/firds/esma/FULINS_D_20250201_01of03.xml").unwrap();
    let reader = BufReader::new(file);
    let mut xml_reader = NsReader::from_reader(reader);

    let mut buf = Vec::new();
    let target_tag = "StrkPric";
    let mut parsed = 0;
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let elem_name = e.name();
                let tag_name = String::from_utf8_lossy(elem_name.as_ref());
                if tag_name == target_tag {
                    // 🧠 Found the tag we're interested in
                    let element = xml_utils::Element::parse_start(&mut xml_reader, e)?;
                    let term = IndexTerm::from_xml(&element);
                    assert!(term.is_ok());
                    //println!("Parsed element tree: {:#?}", element);
                    parsed += 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error: {:?}", e),
            _ => {}
        }
        buf.clear();
    }
    println!("Parsed {parsed} {target_tag} elements.");
    Ok(())
}
