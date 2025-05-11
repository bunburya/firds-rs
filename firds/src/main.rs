mod parse_utils;
mod error;
mod model;
mod categories;
mod product;
mod iter_xml;

use crate::error::ParseError;
use std::env;
use std::fs::{read_dir, File};
use std::io::BufReader;
use std::time::SystemTime;
use crate::iter_xml::XmlIterator;
use crate::model::{ReferenceData, FromXml};

fn main() -> Result<(), ParseError> {
    let args: Vec<String> = env::args().collect();
    let t1 = SystemTime::now();
    let files = read_dir("/mnt/storage/alan/data/firds/esma/")
        .expect("Cannot read directory");
    let lei = args.get(1).expect("No LEI provided");
    let mut count = 0;
    for f in files {
        let dir_entry = f.expect("Cannot access file.");
        let reader = BufReader::new(File::open(dir_entry.path()).expect("Cannot open file"));
        for elem in XmlIterator::new("RefData", reader) {
            let ref_data = ReferenceData::from_xml(&elem?)?;
            if &ref_data.issuer_lei == lei {
                count += 1;
            }
        } 
    }
    println!("Found {count} entries with issuer LEI {lei}. Note: Securities may be counted multiple times.");
    let t2 = SystemTime::now();
    println!("Took {:?}", t2.duration_since(t1));
    Ok(())
}
