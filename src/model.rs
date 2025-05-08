use crate::categories::{DebtSeniority, DeliveryType, FinalPriceType, FxType, IndexName, TermUnit, OptionExerciseStyle, OptionType, StrikePriceType, TransactionType};
use crate::error::ParseError;
use crate::product::BaseProduct;
use crate::xml_utils::{child_or_none, date_or_none, datetime_or_none, parse_or_none, text_or_none, Element};
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
pub struct Term {
    /// The number of weeks, months, etc (as determined by `unit`).
    pub number: i32,
    /// The unit of time in which the term is expressed (days, weeks, months or years).
    pub unit: TermUnit,
}

impl FromXml for Term {
    /// Parse a `Fltg/Term` XML element from FIRDS data into an [`Term`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let number = elem.get_child("Val")?.text.parse::<i32>()?;
        let unit = TermUnit::try_from(elem.get_child("Unit")?.text.as_str())?;
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
pub struct FloatingRate {
    /// The name of the index or benchmark.
    pub name: Option<IndexName>,
    /// The term of the index or benchmark.
    pub term: Option<Term>,
}

impl FromXml for FloatingRate {
    /// Parse an XML element of type `FloatingInterestRate8` into a [`FloatingRate`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let ref_rate_elem = elem.get_child("RefRate").unwrap();
        let name = if let Some(text) = text_or_none(ref_rate_elem.find_child("Indx")) {
            Some(IndexName::from_str(text).unwrap())
        } else if let Some(text) = text_or_none(ref_rate_elem.find_child("Nm")) {
            Some(IndexName::from_str(text).unwrap())
        } else {
            None
        };
        Ok(Self {
            name,
            term: Term::from_xml_option(elem.find_child("Term")).unwrap()
        })
    }
}

/// An index is effectively a [`FloatingRate`], with an optional ISIN code.
#[derive(Debug)]
pub struct Index {
    isin: Option<String>,
    name: FloatingRate
}

impl FromXml for Index {
    /// Parse an XML element of type `FinancialInstrument58` into an [`Index`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self {
            isin: text_or_none(elem.find_child("ISIN")).map(String::from),
            name: FloatingRate::from_xml(elem.get_child("Nm")?)?
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
    /// Floating interest rate, consisting of an [`FloatingRate`] (representing the benchmark) and a spread
    /// expressed as an integer number of basis points (if applicable).
    /// 
    /// In the FIRDS data, some interest rates specify a basis point spread, whereas others specify
    /// the rate only. This variant is intended to represent both, which is why the spread is
    /// optional.
    Floating(FloatingRate, Option<i32>)
}

impl FromXml for InterestRate {

    /// Parse an `IntrstRate` XML element from FIRDS data into an [`InterestRate`] struct.
    /// 
    /// This is designed to work with `IntrstRate` elements of types `InterestRate6Choice` and
    /// `FloatingInterestRate8`. The difference is that the former specifies a basis point spread
    /// whereas the latter does not.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        
        Ok(if let Some(fltg) = elem.find_child("Fltg") {
            Self::Floating(
                FloatingRate::from_xml(fltg)?,
                parse_or_none::<i32>(fltg.find_child("BsisPtSprd"))?
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
    /// Parse a `DerivInstrmAttrbts/AsstClssSpcfcAttrbts/Cmmdty` XML element from FIRDS data into a
    /// [`CommodityDerivativeAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        // Normal structure is `Pdct/<base product>/<sub product>/BasePdct`, but if the base product
        // does not have an associated sub product then structure will be
        // `Pdct/<base product>/BasePdct`. So we first check for `BasePdct` two levels down, if it's
        // not there we check one level down. We also know at that point that there is no sub
        // product (and therefore no further sub product) associated.
        let prod_elem = elem.get_child("Pdct").unwrap();
        let child = prod_elem.get_first_child().unwrap();
        let product = if let Ok(p) = BaseProduct::from_xml(child) {
            p
        } else {
            BaseProduct::from_xml(
                child.get_first_child().unwrap()
            ).unwrap()
        };
        Ok(Self {
            product,
            transaction_type: TransactionType::from_xml_option(elem.find_child("TxTp")).unwrap(),
            final_price_type: FinalPriceType::from_xml_option(elem.find_child("FnlPricTp")).unwrap()
        })
        
    }
}

/// Additional reference data for an interest rate derivative instrument.
#[derive(Debug)]
struct InterestRateDerivativeAttributes {
    /// The reference rate.
    reference_rate: FloatingRate,
    /// The interest rate of leg 1 of the trade, if applicable.
    interest_rate_1: Option<InterestRate>,
    /// In the case of multi-currency or cross-currency swaps the currency
    /// in which leg 2 of the contract is denominated. For swaptions where
    /// the underlying swap is multi-currency, the currency in which leg 2
    /// of the swap is denominated.
    notional_currency_2: Option<String>,
    /// The fixed rate of leg 2 of the trade, if applicable. Expressed as a percentage.
    interest_rate_2: Option<InterestRate>,
}

impl FromXml for InterestRateDerivativeAttributes {
    /// Parse a `DerivInstrmAttrbts/AsstClssSpcfcAttrbts/Intrst` XML element from FIRDS into a
    /// [`InterestRateDerivativeAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self {
            reference_rate: FloatingRate::from_xml(elem.get_child("IntrstRate")?)?,
            interest_rate_1: InterestRate::from_xml_option(elem.find_child("FirstLegIntrstRate"))?,
            notional_currency_2: text_or_none(elem.find_child("OtherNtnlCcy")).map(String::from),
            interest_rate_2: InterestRate::from_xml_option(elem.find_child("OthrLegIntrstRate"))?
        })
    }
}

/// Additional reference data for a foreign exchange derivative instrument.
#[derive(Debug)]
struct FxDerivativeAttributes {
    /// The second currency of the currency pair.
    notional_currency_2: Option<String>,
    /// The type of underlying currency.
    fx_type: Option<FxType>,
}

impl FromXml for FxDerivativeAttributes {
    /// Parse a `DerivInstrmAttrbts/AsstClssSpcfcAttrbts/FX` XML element from FIRDS into a
    /// [`FxDerivativeAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        Ok(Self {
            notional_currency_2: text_or_none(elem.find_child("OthrNtnlCcy")).map(String::from),
            fx_type: FxType::from_xml_option(elem.find_child("FxTp"))?
        })
    }
}

/// Reference data for a single asset which underlies a derivative instrument.
#[derive(Debug)]
enum UnderlyingSingle {
    /// The ISIN of a financial instrument underlying a derivative.
    /// - For ADRs, GDRs and similar instruments, the ISIN code of the financial instrument
    ///   on which those instruments are based. For convertible bonds, the ISIN code of the
    ///   instrument in which the bond can be converted.
    /// - For derivatives or other instruments which have an underlying, the underlying
    ///   instrument ISIN code, when the underlying is admitted to trading, or traded on a
    ///   trading venue. Where the underlying is a stock dividend, then the ISIN code of the
    ///   related share entitling the underlying dividend shall be provided.
    /// - For Credit Default Swaps, the ISIN of the reference obligation shall be provided.
    Isin(String),
    /// An index underlying a derivative.
    Index(Index),
    /// The LEI of an issuer underlying a derivative.
    Lei(String),
}

impl FromXml for UnderlyingSingle {
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        if let Some(child) = elem.find_first_child() {
            match child.name.as_str() {
                "ISIN" => Ok(Self::Isin(child.text.to_owned())),
                "LEI" => Ok(Self::Lei(child.text.to_owned())),
                "Indx" => Ok(Self::Index(Index::from_xml(child).unwrap())),
                _ => Err(ParseError::Enum)
            }
        } else {
            Err(ParseError::ElementNotFound)
        }
    }
}

/// Reference data for a basket of assets which underlie a derivative instrument.
#[derive(Debug)]
struct UnderlyingBasket {
    /// A list of ISINs of the financial instruments in the basket.
    isin: Vec<String>,
    /// A list of LEIs of issuers in the basket.
    issuer_lei: Vec<String>,
}

impl FromXml for UnderlyingBasket {
    /// Parse an XML element of type `FinancialInstrument53` into an [`UnderlyingBasket`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let mut isin = vec![];
        let mut issuer_lei = vec![];
        for c in elem.iter_children() {
            match c.name.as_str() {
                "ISIN" => isin.push(c.text.to_owned()),
                "LEI" => issuer_lei.push(c.text.to_owned()),
                _ => return Err(ParseError::UnexpectedElement)
            }
        }
        Ok(Self {
            isin,
            issuer_lei,
        })
    }
}

/// Reference data for the asset underlying a derivative. The underlying may be a single issuer,
/// instrument or index, or may be a basket of instruments or issuers. The relevant parameter
/// will be populated and the rest will be None.
#[derive(Debug)]
enum DerivativeUnderlying {
    /// Single instrument, index or issuer underlying a derivative instrument.
    Single(UnderlyingSingle),
    /// Basket of instruments or issuers underlying a derivative instrument.
    Basket(UnderlyingBasket),
}

impl FromXml for DerivativeUnderlying {
    /// Parse an XML element of type `FinancialInstrumentIdentification5Choice` into a
    /// [`DerivativeUnderlying`] struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        if let Some(child) = elem.find_first_child() {
            match child.name.as_str() {
                "Sngl" => Ok(Self::Single(UnderlyingSingle::from_xml(child).unwrap())),
                "Bskt" => Ok(Self::Basket(UnderlyingBasket::from_xml(child).unwrap())),
                _ => Err(ParseError::UnexpectedElement)
            }
        } else {
            Err(ParseError::ElementNotFound)
        }
    }
}

/// Asset class-specific attributes of a derivative.
#[derive(Debug, Default)]
struct AssetClassSpecificAttributes {
    /// If the instrument is a commodity derivative, certain commodity-related attributes.
    commodity_attributes: Option<CommodityDerivativeAttributes>,
    /// If the instrument is an interest rate derivative, certain IR-related attributes.
    ir_attributes: Option<InterestRateDerivativeAttributes>,
    /// If the instrument is a foreign exchange derivative, certain FX-related attributes.
    fx_attributes: Option<FxDerivativeAttributes>,
}

impl FromXml for AssetClassSpecificAttributes {
    /// Parse an XML element of type `AssetClass2__1` into an [`AssetClassSpecificAttributes`]
    /// struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let mut attrs = Self::default();
        for c in elem.iter_children() {
            match c.name.as_str() {
                "Cmmdty" => attrs.commodity_attributes = Some(CommodityDerivativeAttributes::from_xml(c).unwrap()),
                "Intrst" => attrs.ir_attributes = Some(InterestRateDerivativeAttributes::from_xml(c).unwrap()),
                "FX" => attrs.fx_attributes = Some(FxDerivativeAttributes::from_xml(c).unwrap()),
                _ => return Err(ParseError::UnexpectedElement)
            }
        }
        Ok(attrs)
    }
}

/// Reference data for a derivative instrument.
///
/// Note that some other types of instrument can also have derivative-related attributes,
/// eg, some collective investment scheme (CFI code C) instruments.
#[derive(Debug, Default)]
struct DerivativeAttributes {
    /// Expiry date of the instrument.
    expiry_date: Option<NaiveDate>,
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
    /// Certain additional attributes which are specific to the asset class of the derivative.
    asset_class_specific_attributes: Option<AssetClassSpecificAttributes>
}

impl FromXml for DerivativeAttributes {
    /// Parse an XML element of type `DerivativeInstrument5__1` into a [`DerivativeAttributes`]
    /// struct.
    fn from_xml(elem: &Element) -> Result<Self, ParseError> {
        let mut attrs = DerivativeAttributes::default();
        for c in elem.iter_children() {
            match c.name.as_str() {
                "XpryDt" =>
                    attrs.expiry_date = Some(NaiveDate::parse_from_str(&c.text, "%Y-%m-%d").unwrap()),
                "PricMltplr" =>
                    attrs.price_multiplier = Some(c.text.parse().unwrap()),
                "UndrlygInstrm" =>
                    attrs.underlying = Some(DerivativeUnderlying::from_xml(c).unwrap()),
                "OptnTp" =>
                    attrs.option_type = Some(OptionType::from_str(&c.text).unwrap()),
                "StrkPric" =>
                    attrs.strike_price = Some(StrikePrice::from_xml(c).unwrap()),
                "OptnExrcStyle" =>
                    attrs.option_exercise_style = Some(OptionExerciseStyle::from_str(&c.text).unwrap()),
                "DlvryTp" =>
                    attrs.delivery_type = Some(DeliveryType::from_str(&c.text).unwrap()),
                "AsstClssSpcfcAttrbts" =>
                    attrs.asset_class_specific_attributes
                        = Some(AssetClassSpecificAttributes::from_xml(c).unwrap()),
                _ => return Err(ParseError::UnexpectedElement)
            }
        }
        Ok(attrs)
    }
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
    use crate::model::{CommodityDerivativeAttributes, DebtAttributes, DerivativeAttributes, FromXml, FxDerivativeAttributes, FloatingRate, InterestRate, InterestRateDerivativeAttributes, PublicationPeriod, StrikePrice, TechnicalAttributes, TradingVenueAttributes};
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
        test_parsing_xml::<FloatingRate>("Fltg", vec![
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
    
    #[test]
    fn test_parse_ir_attrs() {
        test_parsing_xml::<InterestRateDerivativeAttributes>("Intrst", vec![
            ("FULINS_D_20250201_02of03.xml", 0),
            ("FULINS_O_20250201_01of03.xml", 0),
            ("FULINS_C_20250201_01of01.xml", 0),
            ("FULINS_S_20250201_01of05.xml", 26),
            ("FULINS_D_20250201_03of03.xml", 0),
            ("FULINS_S_20250201_04of05.xml", 500000),
            ("FULINS_S_20250201_03of05.xml", 361917),
            ("FULINS_H_20250201_01of02.xml", 211932),
            ("FULINS_R_20250201_08of08.xml", 0),
            ("FULINS_R_20250201_02of08.xml", 0),
            ("FULINS_R_20250201_07of08.xml", 0),
        ])
    }

    #[test]
    fn test_parse_fx_attrs() {
        test_parsing_xml::<FxDerivativeAttributes>("FX", vec![
            ("FULINS_D_20250201_02of03.xml", 0),
            ("FULINS_O_20250201_01of03.xml", 13173),
            ("FULINS_C_20250201_01of01.xml", 0),
            ("FULINS_S_20250201_01of05.xml", 1),
            ("FULINS_D_20250201_03of03.xml", 0),
            ("FULINS_S_20250201_04of05.xml", 0),
            ("FULINS_S_20250201_03of05.xml", 138079),
            ("FULINS_H_20250201_01of02.xml", 3461),
            ("FULINS_R_20250201_08of08.xml", 0),
            ("FULINS_R_20250201_02of08.xml", 0),
        ])
    }
    
    #[test]
    fn test_deriv_attrs() {
        test_parsing_xml::<DerivativeAttributes>("DerivInstrmAttrbts", vec![
            ("FULINS_D_20250201_02of03.xml", 353815),
            ("FULINS_O_20250201_01of03.xml", 500000),
            ("FULINS_C_20250201_01of01.xml", 1),
            ("FULINS_S_20250201_01of05.xml", 500000),
            ("FULINS_D_20250201_03of03.xml", 51612),
            ("FULINS_S_20250201_04of05.xml", 500000),
            ("FULINS_S_20250201_03of05.xml", 500000),
            ("FULINS_H_20250201_01of02.xml", 500000),
            ("FULINS_R_20250201_08of08.xml", 495128),
            ("FULINS_R_20250201_02of08.xml", 498019),
            ("FULINS_R_20250201_07of08.xml", 500000),
            ("FULINS_O_20250201_03of03.xml", 52304),
            ("FULINS_F_20250201_01of01.xml", 47878),
            ("FULINS_E_20250201_01of02.xml", 319156),
            ("FULINS_O_20250201_02of03.xml", 500000),
            ("FULINS_R_20250201_03of08.xml", 498372),
            ("FULINS_R_20250201_05of08.xml", 494300),
            ("FULINS_E_20250201_02of02.xml", 54708),
            ("FULINS_R_20250201_04of08.xml", 498946),
            ("FULINS_I_20250201_01of01.xml", 3),
            ("FULINS_S_20250201_02of05.xml", 500000),
            ("FULINS_D_20250201_01of03.xml", 178802),
            ("FULINS_J_20250201_01of01.xml", 112078),
            ("FULINS_R_20250201_06of08.xml", 500000),
            ("FULINS_H_20250201_02of02.xml", 222360),
            ("FULINS_S_20250201_05of05.xml", 128400),
            ("FULINS_R_20250201_01of08.xml", 499663),
        ])
    }
}