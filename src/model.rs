use crate::categories::{DebtSeniority, DeliveryType, FinalPriceType, FxType, IndexName, IndexTermUnit, OptionExerciseStyle, OptionType, StrikePriceType, TransactionType};
use crate::error::ParseError;
use crate::product::BaseProduct;
use crate::xml_utils::{child_or_none, date_or_none, datetime_or_none, text_or_none, Element};
use chrono::{DateTime, NaiveDate, Utc};
use std::str::FromStr;

pub trait FromXml: Sized {
    
    /// Try to create an instance of `Self` from the given [`Element`].
    fn from_xml(elem: &Element) -> Result<Self, ParseError>;
    
    /// Try to construct an instance of `Self` from the given [`Element`], if provided, otherwise
    /// return `None`. Returns a [`ParseError`] if an `Element` was encountered but could not be
    /// converted to an instance of `Self`.
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
    /// Parse a `Fltg/Term` XML element from FIRDS data into an [`IndexTerm`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let number = elem.get_child("Val")?.text.parse::<i32>()?;
        let unit = IndexTermUnit::try_from(elem.get_child("Unit")?.text.as_str())?;
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
    /// Parse a `DerivInstrmAttrbts/StrkPric` XML element from FIRDS into a [`StrikePrice`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        if let Some(price_elem) = elem.find_child("Pric") {
            let monetary_val_elem = price_elem.find_child("MntryVal");
            let (price_type, val_elem) = if let Some(e) = child_or_none(monetary_val_elem, "Amt") {
                (StrikePriceType::MonetaryValue, e)
            } else if let Some(e) = price_elem.find_child("Pctg") {
                (StrikePriceType::Percentage, e)
            } else if let Some(e) = price_elem.find_child("Yld") {
                (StrikePriceType::Yield, e)
            } else if let Some(e) = price_elem.find_child("BsisPts") {
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
            let no_price_elem = elem.get_child("NoPric")?;
            let pending = no_price_elem.get_child("Pdg")?.text == "PNDG";
            let currency = text_or_none(no_price_elem.find_child("Ccy")).map(ToOwned::to_owned);
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
    /// equivalent XML element from FIRDS data into an [`Index`] struct.
    ///
    /// Specifically, yhe element should be of type `FloatingInterestRate8` as defined in the FULINS
    /// XSD.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let ref_rate_elem = elem.get_child("RefRate")?;
        let name = if let Some(text) = text_or_none(ref_rate_elem.find_child("Indx")) {
            Some(IndexName::from_str(text)?)
        } else if let Some(text) = text_or_none(ref_rate_elem.find_child("Nm")) {
            Some(IndexName::from_str(text)?)
        } else {
            None
        };
        Ok(Self {
            isin: text_or_none(ref_rate_elem.find_child("ISIN")).map(ToOwned::to_owned),
            name,
            term: IndexTerm::from_xml_option(elem.find_child("Term"))?
        })
    }
}

/// Data relating to the trading or admission to trading of a financial instrument on a trading
/// venue.
#[derive(Debug)]
pub struct TradingVenueAttributes {
    /// The Market Identifier Code (ISO 20022) for the trading venue or systemic internaliser.
    pub trading_venue: String,
    /// Whether the issuer has requested or approved the trading or admission to trading of their
    /// financial instruments on a trading venue.
    pub requested_admission: bool,
    /// Date and time the issuer has approved admission to trading or trading in its financial
    /// instruments on a trading venue.
    pub approval_date: Option<DateTime<Utc>>,
    /// Date and time of the request for admission to trading on the trading venue.
    pub request_date: Option<DateTime<Utc>>,
    /// Date and time of the admission to trading on the trading venue or when the instrument was
    /// first traded.
    pub admission_or_first_trade_date: Option<DateTime<Utc>>,
    /// Date and time when the instrument ceases to be traded or admitted to trading on the trading
    /// venue.
    pub termination_date: Option<DateTime<Utc>>,
}

impl FromXml for TradingVenueAttributes {
    /// Parse a `TradgVnRltdAttrbts` XML element from FIRDS into a `TradingVenueAttributes` struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self {
            trading_venue: elem.get_child("Id")?.text.to_owned(),
            requested_admission: elem.get_child("IssrReq")?.text.parse::<bool>()?,
            approval_date: datetime_or_none(elem.find_child("AdmssnApprvlDtByIssr"))?,
            request_date: datetime_or_none(elem.find_child("ReqForAdmssnDt"))?,
            admission_or_first_trade_date: datetime_or_none(elem.find_child("FrstTradDt"))?,
            termination_date: datetime_or_none(elem.find_child("TermntnDt"))?
        })
    }
}

/// Data about the interest rate applicable to a debt instrument.
#[derive(Debug)]
enum InterestRate {
    /// Fixed interest rate, expressed as a percentage (eg, 7.5 means 7.5%).
    Fixed(f64),
    /// Floating interest rate, insisting of an [`Index`] (representing the benchmark) and a spread
    /// expressed as an integer number of basis points.
    Floating(Index, i32)
}

impl FromXml for InterestRate {

    /// Parse an `IntrstRate` XML element from FIRDS data into an [`InterestRate`] struct.
    ///
    /// The element should be an `IntrstRate` element or equivalent. Specifically it should be an
    /// element of type `InterestRate6Choice` as defined in the FULINS XSD file (not
    /// `FloatingInterestRate8` which appears with the same tag for certain interest rate
    /// derivatives).
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        
        Ok(if let Some(fltg) = elem.find_child("Fltg") {
            Self::Floating(
                Index::from_xml(fltg)?,
                fltg.get_child("BsisPtSprd")?.text.parse::<i32>()?
            )
        } else {
            Self::Fixed(elem.get_child("Fxd")?.text.parse::<f64>()?)
        })
    }
}

/// The period for which details on a financial instrument were published.
#[derive(Debug)]
struct PublicationPeriod {
    /// The date from which details on the financial instrument were published.
    from_date: NaiveDate,
    /// The date to which details on the financial instrument were published.
    to_date: Option<NaiveDate>,
}

impl FromXml for PublicationPeriod {
    
    /// Parse a `PblctnPrd` XML element from FIRDS data into a [`PublicationPeriod`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(if let Some(fdtd) = elem.find_child("FrDtToDt") {
            Self {
                from_date: NaiveDate::parse_from_str(&fdtd.text, "%Y-%m-%d")?,
                to_date: date_or_none(fdtd.find_child("ToDt"))?,
            }
        } else {
            Self {
                from_date: NaiveDate::parse_from_str(&elem.get_child("FrDt")?.text, "%Y-%m-%d")?,
                to_date: None,
            }
        })
    }
}

/// The technical attributes of a financial instrument (ie, attributes relating to
/// the submission of details of the financial instrument to FIRDS).
#[derive(Debug)]
struct TechnicalAttributes {
    /// The relevant competent authority for the instrument.
    relevant_competent_authority: Option<String>,
    /// The period for which these details on the financial instrument was published.
    /// NOTE: `publication_period` is optional as it does not appear in TerminatedRecord
    /// classes, but it should always appear in ReferenceData classes.
    publication_period: Option<PublicationPeriod>,
    /// The MIC of the trading venue that reported the record considered as the reference
    /// for the published data.
    relevant_trading_venue: Option<String>,
}

impl FromXml for TechnicalAttributes {

    /// Parse a `TechAttrbts` XML element from FIRDS data into a [`TechnicalAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self {
            relevant_competent_authority: text_or_none(elem.find_child("RlvntCmptntAuthrty"))
                .map(String::from),
            publication_period: PublicationPeriod::from_xml_option(elem.find_child("PblctnPrd"))?,
            relevant_trading_venue: text_or_none(elem.find_child("RlvntTradgVn")).map(String::from)
        })
    }
}

/// Reference data for bonds or other forms of securitised debt.
#[derive(Debug)]
struct DebtAttributes {
    /// The total issued nominal amount of the financial instrument. Amount is expressed
    /// in the `nominal_currency`.
    total_issued_amount: f64,
    /// The maturity date of the financial instrument. Only applies to debt instruments
    /// with defined maturity.
    maturity_date: Option<NaiveDate>,
    /// The currency of the nominal value.
    nominal_currency: String,
    /// The nominal value of each traded unit. If not available, the minimum traded amount
    /// is included. Amount is expressed in the `nominal_currency`.
    nominal_value_per_unit: f64,
    /// Details of the interest rate applicable to the financial instrument.
    interest_rate: InterestRate,
    /// The seniority of the financial instrument (senior, mezzanine, subordinated or junior).
    seniority: Option<DebtSeniority>,
}

impl FromXml for DebtAttributes {
    /// Parse a `DebtInstrmAttrbts` XML element from FIRDS data into a [`DebtAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let issued_amount_elem = elem.get_child("TtlIssdNmnlAmt")?;
        Ok(Self {
            total_issued_amount: issued_amount_elem.text.parse()?,
            maturity_date: date_or_none(elem.find_child("MtrtyDt"))?,
            nominal_currency: issued_amount_elem.get_attr("Ccy")?.to_owned(),
            nominal_value_per_unit: elem.get_child("NmnlValPerUnit")?.text.parse()?,
            interest_rate: InterestRate::from_xml(elem.get_child("IntrstRate")?)?,
            seniority: DebtSeniority::from_xml_option(elem.find_child("DebtSnrty"))?
        })
    }
}

/// Additional reference data for a commodity derivative instrument.
#[derive(Debug)]
struct CommodityDerivativeAttributes {
    /// The base product for the underlying asset class.
    product: BaseProduct,
    /// The transaction type as specified by the trading venue.
    transaction_type: Option<TransactionType>,
    /// The final price type as specified by the trading venue.
    final_price_type: Option<FinalPriceType>,
}

impl FromXml for CommodityDerivativeAttributes {
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        // Normal structure is `Pdct/<base product>/<sub product>/BasePdct`, but if the base product
        // does not have an associated sub product then structure will be
        // `Pdct/<base product>/BasePdct`. So we first check for `BasePdct` two levels down, if it's
        // not there we check one level down. We also know at that point that there is no sub
        // product (and therefore no further sub product) associated.
        let prod_elem = elem.get_child("Pdct")?;
        let child = prod_elem.get_first_child()?;
        let product = if let Ok(p) = BaseProduct::from_xml(child) {
            p
        } else {
            BaseProduct::from_xml(
                child.get_first_child()?
            )?
        };
        Ok(Self {
            product,
            transaction_type: TransactionType::from_xml_option(elem.find_child("TxTp"))?,
            final_price_type: FinalPriceType::from_xml_option(elem.find_child("FnlPricTp"))?  
        })
        
    }
}

/// Additional reference data for an interest rate derivative instrument.
#[derive(Debug)]
struct InterestRateDerivativeAttributes {
    /// The reference rate.
    reference_rate: Index,
    /// In the case of multi-currency or cross-currency swaps the currency
    /// in which leg 2 of the contract is denominated. For swaptions where
    /// the underlying swap is multi-currency, the currency in which leg 2
    /// of the swap is denominated.
    notional_currency_2: Option<String>,
    /// The fixed rate of leg 1 of the trade, if applicable. Expressed as a percentage.
    fixed_rate_1: Option<f64>,
    /// The fixed rate of leg 2 of the trade, if applicable. Expressed as a percentage.
    fixed_rate_2: Option<f64>,
    /// The floating rate of leg 2 of the trade, if applicable.
    floating_rate_2: Option<Index>,
}

/// Additional reference data for a foreign exchange derivative instrument.
#[derive(Debug)]
struct FxDerivativeAttributes {
    /// The second currency of the currency pair.
    notional_currency_2: String,
    /// The type of underlying currency.
    fx_type: FxType,
}

/// Reference data for a single asset which underlies a derivative instrument.
#[derive(Debug)]
struct UnderlyingSingle {
    /// The ISIN of the underlying financial instrument.
    /// - For ADRs, GDRs and similar instruments, the ISIN code of the financial instrument
    ///   on which those instruments are based. For convertible bonds, the ISIN code of the
    ///   instrument in which the bond can be converted.
    /// - For derivatives or other instruments which have an underlying, the underlying
    ///   instrument ISIN code, when the underlying is admitted to trading, or traded on a
    ///   trading venue. Where the underlying is a stock dividend, then the ISIN code of the
    ///   related share entitling the underlying dividend shall be provided.
    /// - For Credit Default Swaps, the ISIN of the reference obligation shall be provided.
    isin: Option<String>,
    /// The ISIN, or an `Index` object, representing the underlying index.
    index: Option<Index>,
    /// The LEI of the underlying issuer.
    issuer_lei: Option<String>,
}

/// Reference data for a basket of assets which underlie a derivative instrument.
#[derive(Debug)]
struct UnderlyingBasket {
    /// A list of ISINs of the financial instruments in the basket.
    isin: Option<Vec<String>>,
    /// A list of LEIs of issuers in the basket.
    issuer_lei: Option<Vec<String>>,
}

/// Reference data for the asset underlying a derivative. The underlying may be a single issuer,
/// instrument or index, or may be a basket of instruments or issuers. The relevant parameter
/// will be populated and the rest will be None.
#[derive(Debug)]
struct DerivativeUnderlying {
    /// Data for a single instrument, index or issuer underlying a derivative instrument,
    /// or None if the underlying is a basket.
    single: Option<UnderlyingSingle>,
    /// Data for a basket of instruments or issuers underlying a derivative instrument,
    /// or None if the underlying is a single instrument, index or issuer.
    basket: Option<UnderlyingBasket>,
}

/// Reference data for a derivative instrument.
///
/// Note that some other types of instrument can also have derivative-related attributes,
/// eg, some collective investment scheme (CFI code C) instruments.
#[derive(Debug)]
struct DerivativeAttributes {
    /// Expiry date of the instrument.
    expiry_date: Option<chrono::NaiveDate>,
    /// Number of units of the underlying instrument represented by a single derivative
    /// contract. For a future or option on an index, the amount per index point.
    price_multiplier: Option<f64>,
    /// Description of the underlying asset or basket of assets.
    underlying: Option<DerivativeUnderlying>,
    /// If the derivative instrument is an option, whether it is a call or a put or whether
    /// it cannot be determined whether it is a call or a put at the time of execution.
    option_type: Option<OptionType>,
    /// Predetermined price at which the holder will have to buy or sell the underlying
    /// instrument, or an indication that the price cannot be determined at the time of execution.
    strike_price: Option<StrikePrice>,
    /// Indication of whether the option may be exercised only at a fixed date (European and
    /// Asian style), a series of pre-specified dates (Bermudan) or at any time during the
    /// life of the contract (American style).
    option_exercise_style: Option<OptionExerciseStyle>,
    /// Whether the financial instrument is cash settled or physically settled or delivery
    /// type cannot be determined at time of execution.
    delivery_type: Option<DeliveryType>,
    /// If the instrument is a commodity derivative, certain commodity-related attributes.
    commodity_attributes: Option<CommodityDerivativeAttributes>,
    /// If the instrument is an interest rate derivative, certain IR-related attributes.
    ir_attributes: Option<InterestRateDerivativeAttributes>,
    /// If the instrument is a foreign exchange derivative, certain FX-related attributes.
    fx_attributes: Option<FxDerivativeAttributes>,
}

/// A base class for financial instrument reference data.
#[derive(Debug)]
struct ReferenceData {
    /// The International Securities Identifier Number (ISO 6166) of the financial instrument.
    isin: String,
    /// The full name of the financial instrument. This should give a good indication of the
    /// issuer and the particulars of the instrument.
    full_name: String,
    /// The Classification of Financial Instruments code (ISO 10962) of the financial instrument.
    cfi: String,
    /// Whether the financial instrument falls within the definition of a "commodities derivative"
    /// under Article 2(1)(30) of Regulation (EU) No 600/2014.
    is_commodities_derivative: bool,
    /// The Legal Entity Identifier (ISO 17442) for the issuer. In certain cases, eg derivative
    /// instruments issued by the trading venue, this field will be populated with the trading
    /// venue operator's LEI.
    issuer_lei: String,
    /// The Financial Instrument Short Name (ISO 18774) for the financial instrument.
    fisn: String,
    /// Data relating to the trading or admission to trading of the financial instrument
    /// on a trading venue.
    trading_venue_attrs: TradingVenueAttributes,
    /// The currency in which the notional is denominated. For an interest rate or currency
    /// derivative contract, this will be the notional currency of leg 1, or the currency 1,
    /// of the pair. In the case of swaptions where the underlying swap is single currency,
    /// this will be the notional currency of the underlying swap. For swaptions where the
    /// underlying is multi-currency, this will be the notional currency of leg 1 of the swap.
    notional_currency: String,
    /// Technical attributes of the financial instrument.
    technical_attributes: Option<TechnicalAttributes>,
    /// If the instrument is a debt instrument, certain debt-related attributes.
    debt_attributes: Option<DebtAttributes>,
    /// If the instrument is a derivative, certain derivative-related attributes.
    derivative_attributes: Option<DerivativeAttributes>,
}
#[cfg(test)]
mod tests {
    use crate::model::{CommodityDerivativeAttributes, DebtAttributes, FromXml, Index, InterestRate, PublicationPeriod, StrikePrice, TechnicalAttributes, TradingVenueAttributes};
    use crate::xml_utils::Element;
    use quick_xml::events::Event;
    use quick_xml::NsReader;
    use std::env::current_dir;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

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
                            let element_res = Element::parse_start(&mut xml_reader, e);
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
        test_parsing_xml::<StrikePrice>("StrkPric", vec![
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
        ])
    }
    
    #[test]
    fn test_parse_index() {
        test_parsing_xml::<Index>("Fltg", vec![
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
        ])
    }
    
    #[test]
    fn test_parse_trading_venue_attrs() {
        test_parsing_xml::<TradingVenueAttributes>("TradgVnRltdAttrbts", vec![
            ("FULINS_D_20250201_02of03.xml", 500000),
            ("FULINS_O_20250201_01of03.xml", 500000),
            ("FULINS_C_20250201_01of01.xml", 125816),
            ("FULINS_S_20250201_01of05.xml", 500000),
            ("FULINS_D_20250201_03of03.xml", 193982),
            ("FULINS_S_20250201_04of05.xml", 500000),
            ("FULINS_S_20250201_03of05.xml", 500000),
            ("FULINS_H_20250201_01of02.xml", 500000),
            ("FULINS_R_20250201_08of08.xml", 495128),
            ("FULINS_R_20250201_02of08.xml", 500000),
            ("FULINS_R_20250201_07of08.xml", 500000),
            ("FULINS_O_20250201_03of03.xml", 52304),
            ("FULINS_F_20250201_01of01.xml", 47878),
            ("FULINS_E_20250201_01of02.xml", 500000),
            ("FULINS_O_20250201_02of03.xml", 500000),
            ("FULINS_R_20250201_03of08.xml", 500000),
            ("FULINS_R_20250201_05of08.xml", 500000),
            ("FULINS_E_20250201_02of02.xml", 55790),
            ("FULINS_R_20250201_04of08.xml", 500000),
            ("FULINS_I_20250201_01of01.xml", 3),
            ("FULINS_S_20250201_02of05.xml", 500000),
            ("FULINS_D_20250201_01of03.xml", 500000),
            ("FULINS_J_20250201_01of01.xml", 112078),
            ("FULINS_R_20250201_06of08.xml", 500000),
            ("FULINS_H_20250201_02of02.xml", 222360),
            ("FULINS_S_20250201_05of05.xml", 128400),
            ("FULINS_R_20250201_01of08.xml", 500000),
        ])
    }
    
    #[test]
    fn test_parse_interest_rate() {
        test_parsing_xml::<InterestRate>(
            "IntrstRate",
                // NB: Don't include derivatives files here as the "IntrstRate" tag appears there
                // with a different type.
            vec![
                ("FULINS_D_20250201_02of03.xml", 500000),
                ("FULINS_O_20250201_01of03.xml", 0),
                ("FULINS_C_20250201_01of01.xml", 0),
                ("FULINS_D_20250201_03of03.xml", 193982),
                ("FULINS_R_20250201_08of08.xml", 0),
                ("FULINS_R_20250201_02of08.xml", 0),
                ("FULINS_R_20250201_07of08.xml", 0),
                ("FULINS_O_20250201_03of03.xml", 0),
                ("FULINS_E_20250201_01of02.xml", 0),
                ("FULINS_O_20250201_02of03.xml", 0),
            ]
        )
    }
    
    #[test]
    fn test_parse_publication_period() {
        test_parsing_xml::<PublicationPeriod>("PblctnPrd", vec![
            ("FULINS_D_20250201_02of03.xml", 500000),
            ("FULINS_O_20250201_01of03.xml", 500000),
            ("FULINS_C_20250201_01of01.xml", 125816),
            ("FULINS_S_20250201_01of05.xml", 500000),
            ("FULINS_D_20250201_03of03.xml", 193982),
            ("FULINS_S_20250201_04of05.xml", 500000),
            ("FULINS_S_20250201_03of05.xml", 500000),
            ("FULINS_H_20250201_01of02.xml", 500000),
            ("FULINS_R_20250201_08of08.xml", 495128),
            ("FULINS_R_20250201_02of08.xml", 500000),
            ("FULINS_R_20250201_07of08.xml", 500000),
            ("FULINS_O_20250201_03of03.xml", 52304),
            ("FULINS_F_20250201_01of01.xml", 47878),
            ("FULINS_E_20250201_01of02.xml", 500000),
            ("FULINS_O_20250201_02of03.xml", 500000),
            ("FULINS_R_20250201_03of08.xml", 500000),
            ("FULINS_R_20250201_05of08.xml", 500000),
            ("FULINS_E_20250201_02of02.xml", 55790),
            ("FULINS_R_20250201_04of08.xml", 500000),
            ("FULINS_I_20250201_01of01.xml", 3),
            ("FULINS_S_20250201_02of05.xml", 500000),
            ("FULINS_D_20250201_01of03.xml", 500000),
            ("FULINS_J_20250201_01of01.xml", 112078),
            ("FULINS_R_20250201_06of08.xml", 500000),
            ("FULINS_H_20250201_02of02.xml", 222360),
            ("FULINS_S_20250201_05of05.xml", 128400),
            ("FULINS_R_20250201_01of08.xml", 500000),
        ])
    }
    
    #[test]
    fn test_parse_tech_attr() {
        test_parsing_xml::<TechnicalAttributes>("TechAttrbts", vec![
            ("FULINS_D_20250201_02of03.xml", 500000),
            ("FULINS_O_20250201_01of03.xml", 500000),
            ("FULINS_C_20250201_01of01.xml", 125816),
            ("FULINS_S_20250201_01of05.xml", 500000),
            ("FULINS_D_20250201_03of03.xml", 193982),
            ("FULINS_S_20250201_04of05.xml", 500000),
            ("FULINS_S_20250201_03of05.xml", 500000),
            ("FULINS_H_20250201_01of02.xml", 500000),
            ("FULINS_R_20250201_08of08.xml", 495128),
            ("FULINS_R_20250201_02of08.xml", 500000),
            ("FULINS_R_20250201_07of08.xml", 500000),
            ("FULINS_O_20250201_03of03.xml", 52304),
            ("FULINS_F_20250201_01of01.xml", 47878),
            ("FULINS_E_20250201_01of02.xml", 500000),
            ("FULINS_O_20250201_02of03.xml", 500000),
            ("FULINS_R_20250201_03of08.xml", 500000),
            ("FULINS_R_20250201_05of08.xml", 500000),
            ("FULINS_E_20250201_02of02.xml", 55790),
            ("FULINS_R_20250201_04of08.xml", 500000),
            ("FULINS_I_20250201_01of01.xml", 3),
            ("FULINS_S_20250201_02of05.xml", 500000),
            ("FULINS_D_20250201_01of03.xml", 500000),
            ("FULINS_J_20250201_01of01.xml", 112078),
            ("FULINS_R_20250201_06of08.xml", 500000),
            ("FULINS_H_20250201_02of02.xml", 222360),
            ("FULINS_S_20250201_05of05.xml", 128400),
            ("FULINS_R_20250201_01of08.xml", 500000),
        ])
    }
    
    #[test]
    fn test_parse_debt_attrs() {
        test_parsing_xml::<DebtAttributes>("DebtInstrmAttrbts", vec![
            ("FULINS_D_20250201_02of03.xml", 500000),
            ("FULINS_O_20250201_01of03.xml", 0),
            ("FULINS_C_20250201_01of01.xml", 0),
            ("FULINS_S_20250201_01of05.xml", 0),
            ("FULINS_D_20250201_03of03.xml", 193982),
            ("FULINS_S_20250201_04of05.xml", 0),
            ("FULINS_S_20250201_03of05.xml", 0),
            ("FULINS_H_20250201_01of02.xml", 0),
            ("FULINS_R_20250201_08of08.xml", 0),
        ])
    }
    
    #[test]
    fn test_parse_commodity_attrs() {
        test_parsing_xml::<CommodityDerivativeAttributes>("Cmmdty", vec![
            ("FULINS_D_20250201_02of03.xml", 2679),
            ("FULINS_O_20250201_01of03.xml", 0),
            ("FULINS_C_20250201_01of01.xml", 0),
            ("FULINS_S_20250201_01of05.xml", 0),
            ("FULINS_D_20250201_03of03.xml", 0),
            ("FULINS_S_20250201_04of05.xml", 0),
            ("FULINS_S_20250201_03of05.xml", 0),
            ("FULINS_H_20250201_01of02.xml", 0),
            ("FULINS_R_20250201_08of08.xml", 47465),
            ("FULINS_R_20250201_02of08.xml", 56826),
        ])
    }
}