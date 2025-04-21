use std::fs::File;
use std::io::BufReader;
use quick_xml::events::Event;
use quick_xml::NsReader;

mod xml_utils;

fn main() {
    let file = File::open("/mnt/storage/alan/data/firds/esma/FULINS_C_20250201_01of01.xml").unwrap();
    let reader = BufReader::new(file);
    let mut xml_reader = NsReader::from_reader(reader);

    let mut buf = Vec::new();
    let target_tag = "RefData";
    let mut parsed = 0;
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let elem_name = e.name();
                let tag_name = String::from_utf8_lossy(elem_name.as_ref());
                if tag_name == target_tag {
                    // 🧠 Found the tag we're interested in
                    let element = crate::xml_utils::Element::parse(&mut xml_reader, e);
                    //println!("Parsed element tree: {:#?}", element);
                    parsed += 1;

                    // ⚡ Do your processing here...
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error: {:?}", e),
            _ => {}
        }
        buf.clear();
    }
    println!("Parsed {parsed} {target_tag} elements.");
}