use std::num::ParseIntError;
use crate::categories::{IndexTermUnit, StrikePriceType};
use crate::error::ParseError;
use crate::xml_utils::Element;

/// The term of an index or benchmark.
#[derive(Debug)]
pub struct IndexTerm {
    /// The number of weeks, months, etc (as determined by `unit`).
    pub number: i32,
    /// The unit of time in which the term is expressed (days, weeks, months or years).
    pub unit: IndexTermUnit,
}

impl IndexTerm {
    /// Parse a `Fltg/Term` XML element from FIRDS data into an `IndexTerm` object.
    ///
    /// # Arguments
    /// * `elem` - The XML element to parse. The tag should be `{urn:iso:std:iso:20022:tech:xsd:auth.017.001.02}Term` or equivalent.
    pub fn from_xml(elem: &Element) -> Result<Self, ParseError> {
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

impl StrikePrice {
    /// Parse a `DerivInstrmAttrbts/StrkPric` XML element from FIRDS into a `StrikePrice` object.
    ///
    /// # Arguments
    /// * `elem` - The XML element to parse. The tag should be `{urn:iso:std:iso:20022:tech:xsd:auth.017.001.02}StrkPric` or equivalent.
    pub fn from_xml(elem: &Element) -> Self {
        todo!() // Placeholder for XML parse logic
    }
}

/// An index or benchmark rate used in the reference data for certain financial instruments.
#[derive(Debug)]
pub struct Index {
    /// The name of the index or benchmark. Should either be an `IndexName` object or a 25-character string.
    pub name: Option<String>,
    /// The ISIN of the index or benchmark.
    pub isin: Option<String>,
    /// The term of the index or benchmark.
    pub term: Option<IndexTerm>,
}

impl Index {
    /// Parse an `IntrstRate/Fltg` or equivalent XML element from FIRDS data into an `Index` object.
    ///
    /// # Arguments
    /// * `elem` - The XML element to parse.
    pub fn from_xml(elem: &Element) -> Self {
        todo!() // Placeholder for XML parse logic
    }
}

/// Data relating to the trading or admission to trading of a financial instrument on a trading venue.
#[derive(Debug)]
pub struct TradingVenueAttributes {
    /// The Market Identifier Code (ISO 20022) for the trading venue or systemic internaliser.
    pub trading_venue: String,
    /// Whether the issuer has requested or approved the trading or admission to trading of their financial instruments on a trading venue.
    pub requested_admission: bool,
    /// Date and time the issuer has approved admission to trading or trading in its financial instruments on a trading venue.
    pub approval_date: Option<DateTime>,
    /// Date and time of the request for admission to trading on the trading venue.
    pub request_date: Option<DateTime>,
    /// Date and time of the admission to trading on the trading venue or when the instrument was first traded.
    pub admission_or_first_trade_date: Option<DateTime>,
    /// Date and time when the instrument ceases to be traded or admitted to trading on the trading venue.
    pub termination_date: Option<DateTime>,
}

impl TradingVenueAttributes {
    /// Parse a `TradgVnRltAttrbts` XML element from FIRDS into a `TradingVenueAttributes` object.
    ///
    /// # Arguments
    /// * `elem` - The XML element to parse. The tag should be `{urn:iso:std:iso:20022:tech:xsd:auth.017.001.02}TradgVnRltAttrbts` or equivalent.
    pub fn from_xml(elem: &Element) -> Self {
        todo!() // Placeholder for XML parse logic
    }
}