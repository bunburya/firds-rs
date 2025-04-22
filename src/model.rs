use crate::categories::{IndexName, IndexTermUnit, StrikePriceType};
use crate::error::ParseError;
use crate::xml_utils::{child_or_none, text_or_none, Element};
use std::io::Read;
use std::str::FromStr;

pub trait FromXml: Sized {
    fn from_xml(elem: &Element) -> Result<Self, ParseError>;
    
    fn from_xml_option(elem: Option<&Element>) -> Result<Option<Self>, ParseError> {
        if let Some(e) = elem {
            Ok(Some(Self::from_xml(e)?))
        } else {
            Ok(None)
        }

    }
}

/// The term of an index or benchmark.
#[derive(Debug)]
pub struct IndexTerm {
    /// The number of weeks, months, etc (as determined by `unit`).
    pub number: i32,
    /// The unit of time in which the term is expressed (days, weeks, months or years).
    pub unit: IndexTermUnit,
}

impl FromXml for IndexTerm {
    /// Parse a `Fltg/Term` XML element from FIRDS data into an `IndexTerm` object.
    ///
    /// # Arguments
    /// * `elem` - The XML element to parse. The tag should be `{urn:iso:std:iso:20022:tech:xsd:auth.017.001.02}Term` or equivalent.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let number = elem.get("Val")?.text.parse::<i32>()?;
        let unit = IndexTermUnit::try_from(elem.get("Unit")?.text.as_str())?;
        Ok(Self {
            number,
            unit
        })
    }
}

/// The strike price of a derivative instrument.
#[derive(Debug)]
pub struct StrikePrice {
    /// How the price is expressed (e.g., monetary value, percentage, yield, or basis points).
    /// Alternatively identifies if no price is available.
    pub price_type: StrikePriceType,
    /// The actual price, expressed according to `price_type`. Will be `None` if no price is available.
    pub price: Option<f64>,
    /// Whether the price is currently not available and is pending.
    pub pending: bool,
    /// The currency in which the price is denominated (if appropriate).
    pub currency: Option<String>,
}

impl FromXml for StrikePrice {
    /// Parse a `DerivInstrmAttrbts/StrkPric` XML element from FIRDS into a `StrikePrice` object.
    ///
    /// # Arguments
    /// * `elem` - The XML element to parse. The tag should be
    /// `{urn:iso:std:iso:20022:tech:xsd:auth.017.001.02}StrkPric` or equivalent.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        if let Some(price_elem) = elem.find("Pric") {
            let monetary_val_elem = price_elem.find("MntryVal");
            let (price_type, val_elem) = if let Some(e) = child_or_none(monetary_val_elem, "Amt") {
                (StrikePriceType::MonetaryValue, e)
            } else if let Some(e) = price_elem.find("Pctg") {
                (StrikePriceType::Percentage, e)
            } else if let Some(e) = price_elem.find("Yld") {
                (StrikePriceType::Yield, e)
            } else if let Some(e) = price_elem.find("BsisPts") {
                (StrikePriceType::BasisPoints, e)
            } else {
                return Err(ParseError::ElementNotFound)
            };
            let price = val_elem.text.parse::<f64>()?;
            let currency = text_or_none(child_or_none(monetary_val_elem, "Ccy"))
                .map(ToOwned::to_owned);
            Ok(Self {
                price_type,
                price: Some(price),
                pending: false,
                currency
            })
        } else {
            let no_price_elem = elem.get("NoPric")?;
            let pending = no_price_elem.get("Pdg")?.text == "PNDG";
            let currency = text_or_none(no_price_elem.find("Ccy")).map(ToOwned::to_owned);
            Ok(Self {
                price_type: StrikePriceType::NoPrice,
                price: None,
                pending,
                currency
            })
        }
        
    }
}

/// An index or benchmark rate used in the reference data for certain financial instruments.
#[derive(Debug)]
pub struct Index {
    /// The name of the index or benchmark.
    pub name: Option<IndexName>,
    /// The ISIN of the index or benchmark.
    pub isin: Option<String>,
    /// The term of the index or benchmark.
    pub term: Option<IndexTerm>,
}

impl FromXml for Index {
    /// Parse an `IntrstRate/Fltg`, `DerivInstrmAttrbts/UndrlygInstrm/Sngl/Indx/Nm/RefRate/Nm` or
    /// equivalent XML element from FIRDS data into an `Index` object.
    ///
    /// # Arguments
    /// * `elem` - The XML element to parse. The element should be of type `FloatingInterestRate8`
    /// as defined in the FULINS XSD.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let ref_rate_elem = elem.get("RefRate")?;
        let name = if let Some(text) = text_or_none(ref_rate_elem.find("Indx")) {
            Some(IndexName::from_str(text)?)
        } else if let Some(text) = text_or_none(ref_rate_elem.find("Nm")) {
            Some(IndexName::from_str(text)?)
        } else {
            None
        };
        Ok(Self {
            isin: text_or_none(ref_rate_elem.find("ISIN")).map(ToOwned::to_owned),
            name,
            term: IndexTerm::from_xml_option(elem.find("Term"))?
        })
    }
}

///// Data relating to the trading or admission to trading of a financial instrument on a trading venue.
// #[derive(Debug)]
// pub struct TradingVenueAttributes {
//     /// The Market Identifier Code (ISO 20022) for the trading venue or systemic internaliser.
//     pub trading_venue: String,
//     /// Whether the issuer has requested or approved the trading or admission to trading of their financial instruments on a trading venue.
//     pub requested_admission: bool,
//     /// Date and time the issuer has approved admission to trading or trading in its financial instruments on a trading venue.
//     pub approval_date: Option<DateTime>,
//     /// Date and time of the request for admission to trading on the trading venue.
//     pub request_date: Option<DateTime>,
//     /// Date and time of the admission to trading on the trading venue or when the instrument was first traded.
//     pub admission_or_first_trade_date: Option<DateTime>,
//     /// Date and time when the instrument ceases to be traded or admitted to trading on the trading venue.
//     pub termination_date: Option<DateTime>,
// }
// 
// impl TradingVenueAttributes {
//     /// Parse a `TradgVnRltAttrbts` XML element from FIRDS into a `TradingVenueAttributes` object.
//     ///
//     /// # Arguments
//     /// * `elem` - The XML element to parse. The tag should be `{urn:iso:std:iso:20022:tech:xsd:auth.017.001.02}TradgVnRltAttrbts` or equivalent.
//     pub fn from_xml(elem: &Element) -> Self {
//         todo!() // Placeholder for XML parse logic
//     }
// }

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use crate::model::{FromXml, Index, StrikePrice};
    use quick_xml::events::Event;
    use quick_xml::NsReader;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::{Path, PathBuf};
    
    fn get_firds_data_dir() -> PathBuf {
        current_dir().unwrap().join("test_data").join("firds_data")
    }

    fn test_parsing_xml<T: FromXml>(tag: &str, files: Vec<(&str, i32)>) {
        for (fname, count) in files {
            let path = get_firds_data_dir().join("esma").join(fname);
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            let mut xml_reader = NsReader::from_reader(reader);
            let mut buf = Vec::new();
            let mut parsed = 0;
            loop {
                match xml_reader.read_event_into(&mut buf) {
                    Ok(Event::Start(e)) => {
                        let elem_name = e.name();
                        let tag_name = String::from_utf8_lossy(elem_name.as_ref());
                        if tag_name == tag {
                            // 🧠 Found the tag we're interested in
                            let element_res = crate::xml_utils::Element::parse(&mut xml_reader, e);
                            assert!(element_res.is_ok());
                            let element = element_res.unwrap();
                            let from_xml_res = T::from_xml(&element);
                            assert!(from_xml_res.is_ok());
                            parsed += 1;
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => panic!("Error: {:?}", e),
                    _ => {}
                }
                buf.clear();
            }
            println!("{fname}: {parsed}");
            assert_eq!(parsed, count);
        }
    }

    #[test]
    fn test_parse_strike_price() {
        test_parsing_xml::<StrikePrice>(
            "StrkPric",
            vec![
                ("FULINS_D_20250201_02of03.xml", 0),
                ("FULINS_O_20250201_01of03.xml", 500000),
                ("FULINS_C_20250201_01of01.xml", 0),
                ("FULINS_S_20250201_01of05.xml", 0),
                ("FULINS_D_20250201_03of03.xml", 0),
                ("FULINS_S_20250201_04of05.xml", 0),
                ("FULINS_S_20250201_03of05.xml", 0),
                ("FULINS_H_20250201_01of02.xml", 284971),
                ("FULINS_R_20250201_08of08.xml", 495128),
                ("FULINS_R_20250201_02of08.xml", 23289),
                ("FULINS_R_20250201_07of08.xml", 500000),
                ("FULINS_O_20250201_03of03.xml", 52304),
                ("FULINS_F_20250201_01of01.xml", 0),
                ("FULINS_E_20250201_01of02.xml", 0),
                ("FULINS_O_20250201_02of03.xml", 500000),
                ("FULINS_R_20250201_03of08.xml", 23290),
                ("FULINS_R_20250201_05of08.xml", 156758),
                ("FULINS_E_20250201_02of02.xml", 0),
                ("FULINS_R_20250201_04of08.xml", 20027),
                ("FULINS_I_20250201_01of01.xml", 0),
                ("FULINS_S_20250201_02of05.xml", 0),
                ("FULINS_D_20250201_01of03.xml", 0),
                ("FULINS_J_20250201_01of01.xml", 0),
                ("FULINS_R_20250201_06of08.xml", 500000),
                ("FULINS_H_20250201_02of02.xml", 179),
                ("FULINS_S_20250201_05of05.xml", 0),
                ("FULINS_R_20250201_01of08.xml", 15415),
            ]
        )
    }
    
    #[test]
    fn test_parse_index() {
        test_parsing_xml::<Index>(
            "Fltg",
            vec![
                ("FULINS_D_20250201_02of03.xml", 7602),
                ("FULINS_O_20250201_01of03.xml", 0),
                ("FULINS_C_20250201_01of01.xml", 0),
                ("FULINS_S_20250201_01of05.xml", 0),
                ("FULINS_D_20250201_03of03.xml", 20814),
                ("FULINS_S_20250201_04of05.xml", 873),
                ("FULINS_S_20250201_03of05.xml", 72378),
                ("FULINS_H_20250201_01of02.xml", 0),
                ("FULINS_R_20250201_08of08.xml", 0),
                ("FULINS_R_20250201_02of08.xml", 0),
                ("FULINS_R_20250201_07of08.xml", 0),
                ("FULINS_O_20250201_03of03.xml", 0),
                ("FULINS_F_20250201_01of01.xml", 0),
            ]
        )
    }
}