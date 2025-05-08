//! Some enum types that are used as building blocks in the main [`ReferenceData`] structs.

use std::str::FromStr;
use strum_macros::{EnumString, Display};
use crate::error::ParseError;
use crate::model::FromXml;
use crate::xml_utils::Element;

/// Represents the unit of time in which the term is expressed (days, weeks, months, or years).
#[derive(Debug, EnumString, Display)]
pub enum TermUnit {
    #[strum(serialize = "DAYS")]
    Days,
    #[strum(serialize = "WEEK")]
    Week,
    #[strum(serialize = "MNTH")]
    Month,
    #[strum(serialize = "YEAR")]
    Year,
}

/// A four-letter code representing an index or benchmark.
#[derive(Debug, EnumString, Display)]
pub enum IndexCode {
    #[strum(serialize = "EONA")]
    Eonia,
    #[strum(serialize = "EONS")]
    EoniaSwap,
    #[strum(serialize = "EURO")]
    Euribor,
    #[strum(serialize = "EUCH")]
    EuroSwiss,
    #[strum(serialize = "GCFR")]
    GcfRepo,
    #[strum(serialize = "ISDA")]
    Isdafix,
    #[strum(serialize = "LIBI")]
    Libid,
    #[strum(serialize = "LIBO")]
    Libor,
    #[strum(serialize = "MAAA")]
    MuniAaa,
    #[strum(serialize = "PFAN")]
    Pfandbriefe,
    #[strum(serialize = "TIBO")]
    Tibor,
    #[strum(serialize = "STBO")]
    Stibor,
    #[strum(serialize = "BBSW")]
    Bbsw,
    #[strum(serialize = "JIBA")]
    Jibar,
    #[strum(serialize = "BUBO")]
    Bubor,
    #[strum(serialize = "CDOR")]
    Cdor,
    #[strum(serialize = "CIBO")]
    Cibor,
    #[strum(serialize = "MOSP")]
    Mosprim,
    #[strum(serialize = "NIBO")]
    Nibor,
    #[strum(serialize = "PRBO")]
    Pribor,
    #[strum(serialize = "TLBO")]
    Telbor,
    #[strum(serialize = "WIBO")]
    Wibor,
    #[strum(serialize = "TREA")]
    Treasury,
    #[strum(serialize = "SWAP")]
    Swap,
    #[strum(serialize = "FUSW")]
    FutureSwap,
}

/// The name of an index or benchmark.
#[derive(Debug)]
pub enum IndexName {
    /// A four-letter code representing the index or benchmark.
    Code(IndexCode),
    /// Free text describing the name of the index or benchmark.
    Text(String),
}

impl FromStr for IndexName {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(code) = IndexCode::from_str(s) {
            Ok(IndexName::Code(code))
        } else {
            Ok(IndexName::Text(s.to_string()))
        }
    }
}

/// Represents the seniority of a debt instrument.
#[derive(Debug, EnumString, Display)]
pub enum DebtSeniority {
    #[strum(serialize = "SNDB")]
    Senior,
    #[strum(serialize = "MZZD")]
    Mezzanine,
    #[strum(serialize = "SBOD")]
    Subordinated,
    #[strum(serialize = "JUND")]
    Junior,
}

impl FromXml for DebtSeniority {

    /// Parse a `DebtSnrty` XML element from FIRDS data into a [`DebtSeniority`] enum.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self::from_str(&elem.text)?)
    } 
}

/// Represents the type of an option (put, call, or other).
#[derive(Debug, EnumString, Display)]
pub enum OptionType {
    #[strum(serialize = "PUTO")]
    Put,
    #[strum(serialize = "CALL")]
    Call,
    #[strum(serialize = "OTHR")]
    Other,
}

/// Represents the exercise style of an option (European, American, etc.).
#[derive(Debug, EnumString, Display)]
pub enum OptionExerciseStyle {
    #[strum(serialize = "EURO")]
    European,
    #[strum(serialize = "AMER")]
    American,
    #[strum(serialize = "ASIA")]
    Asian,
    #[strum(serialize = "BERM")]
    Bermudan,
    #[strum(serialize = "OTHR")]
    Other,
}

/// Represents the delivery type of a financial instrument (physical, cash, etc.).
#[derive(Debug, EnumString, Display)]
pub enum DeliveryType {
    #[strum(serialize = "PHYS")]
    Physical,
    #[strum(serialize = "CASH")]
    Cash,
    #[strum(serialize = "OPTL")]
    Optional,
}

/// Represents the type of transaction.
#[derive(Debug, EnumString, Display)]
pub enum TransactionType {
    #[strum(serialize = "FUTR")]
    Futures,
    #[strum(serialize = "OPTN")]
    Options,
    #[strum(serialize = "TAPO")]
    Tapos,
    #[strum(serialize = "SWAP")]
    Swaps,
    #[strum(serialize = "MINI")]
    Minis,
    #[strum(serialize = "OTCT")]
    OverTheCounter,
    #[strum(serialize = "ORIT")]
    Outright,
    #[strum(serialize = "CRCK")]
    Crack,
    #[strum(serialize = "DIFF")]
    Differential,
    #[strum(serialize = "OTHR")]
    Other,
}
impl FromXml for TransactionType {
    /// Parse a `TxTp` XML element from FIRDS data into a [`TransactionType`] enum.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self::from_str(&elem.text)?)
    }
}

/// Represents the final price type of a derivative.
#[derive(Debug, EnumString, Display)]
pub enum FinalPriceType {
    #[strum(serialize = "ARGM")]
    ArgusMcCloskey,
    #[strum(serialize = "BLTC")]
    Baltic,
    #[strum(serialize = "EXOF")]
    Exchange,
    #[strum(serialize = "GBCL")]
    GlobalCoal,
    #[strum(serialize = "IHSM")]
    IHSMarkit,
    #[strum(serialize = "PLAT")]
    Platts,
    #[strum(serialize = "OTHR")]
    Other,
}

impl FromXml for FinalPriceType {
    /// Parse a `FnlPricTp` XML element from FIRDS data into a [`FinalPriceType`] enum.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self::from_str(&elem.text)?)
    }
}

/// Represents the type of FX.
#[derive(Debug, EnumString, Display)]
pub enum FxType {
    #[strum(serialize = "FXCR")]
    CrossRates,
    #[strum(serialize = "FXEM")]
    EmergingMarkets,
    #[strum(serialize = "FXMJ")]
    Majors,
}

impl FromXml for FxType {
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self::from_str(&elem.text)?)
    }
}

/// Represents the type of strike price.
#[derive(Debug, EnumString, Display)]
pub enum StrikePriceType {
    #[strum(serialize = "MONETARY_VALUE")]
    MonetaryValue,
    #[strum(serialize = "PERCENTAGE")]
    Percentage,
    #[strum(serialize = "YIELD")]
    Yield,
    #[strum(serialize = "BASIS_POINTS")]
    BasisPoints,
    #[strum(serialize = "NO_PRICE")]
    NoPrice,
}