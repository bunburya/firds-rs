use crate::enums::{DebtSeniority, DeliveryType, FinalPriceType, FxType, IndexName, OptionExerciseStyle, OptionType, StrikePriceType, TermUnit, TransactionType};
use crate::product_enums::BaseProduct;
use chrono::{DateTime, NaiveDate, Utc};

/// The term of an index or benchmark.
#[derive(Debug, Copy, Clone)]
pub struct Term {
    /// The number of weeks, months, etc (as determined by `unit`).
    pub number: i32,
    /// The unit of time in which the term is expressed (days, weeks, months or years).
    pub unit: TermUnit,
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

/// An index or benchmark rate used in the reference data for certain financial instruments.
#[derive(Debug)]
pub struct FloatingRate {
    /// The name of the index or benchmark.
    pub name: Option<IndexName>,
    /// The term of the index or benchmark.
    pub term: Option<Term>,
}

/// An index is effectively a [`FloatingRate`], with an optional ISIN code.
#[derive(Debug)]
pub struct Index {
    pub isin: Option<String>,
    pub name: FloatingRate
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

/// Data about the interest rate applicable to a debt instrument.
#[derive(Debug)]
pub enum InterestRate {
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

/// The period for which details on a financial instrument were published.
#[derive(Debug)]
pub struct PublicationPeriod {
    /// The date from which details on the financial instrument were published.
    pub from_date: NaiveDate,
    /// The date to which details on the financial instrument were published.
    pub to_date: Option<NaiveDate>,
}

/// The technical attributes of a financial instrument (ie, attributes relating to
/// the submission of details of the financial instrument to FIRDS).
#[derive(Debug)]
pub struct TechnicalAttributes {
    /// The relevant competent authority for the instrument.
    pub relevant_competent_authority: Option<String>,
    /// The period for which these details on the financial instrument was published.
    /// NOTE: `publication_period` is optional as it does not appear in TerminatedRecord
    /// classes, but it should always appear in ReferenceData classes.
    pub publication_period: Option<PublicationPeriod>,
    /// The MIC of the trading venue that reported the record considered as the reference
    /// for the published data.
    pub relevant_trading_venue: Option<String>,
}

/// Reference data for bonds or other forms of securitised debt.
#[derive(Debug)]
pub struct DebtAttributes {
    /// The total issued nominal amount of the financial instrument. Amount is expressed
    /// in the `nominal_currency`.
    pub total_issued_amount: f64,
    /// The maturity date of the financial instrument. Only applies to debt instruments
    /// with defined maturity.
    pub maturity_date: Option<NaiveDate>,
    /// The currency of the nominal value.
    pub nominal_currency: String,
    /// The nominal value of each traded unit. If not available, the minimum traded amount
    /// is included. Amount is expressed in the `nominal_currency`.
    pub nominal_value_per_unit: f64,
    /// Details of the interest rate applicable to the financial instrument.
    pub interest_rate: InterestRate,
    /// The seniority of the financial instrument (senior, mezzanine, subordinated or junior).
    pub seniority: Option<DebtSeniority>,
}

/// Additional reference data for a commodity derivative instrument.
#[derive(Debug)]
pub struct CommodityDerivativeAttributes {
    /// The base product for the underlying asset class.
    pub product: BaseProduct,
    /// The transaction type as specified by the trading venue.
    pub transaction_type: Option<TransactionType>,
    /// The final price type as specified by the trading venue.
    pub final_price_type: Option<FinalPriceType>,
}

/// Additional reference data for an interest rate derivative instrument.
#[derive(Debug)]
pub struct InterestRateDerivativeAttributes {
    /// The reference rate.
    pub reference_rate: FloatingRate,
    /// The interest rate of leg 1 of the trade, if applicable.
    pub interest_rate_1: Option<InterestRate>,
    /// In the case of multi-currency or cross-currency swaps the currency
    /// in which leg 2 of the contract is denominated. For swaptions where
    /// the underlying swap is multi-currency, the currency in which leg 2
    /// of the swap is denominated.
    pub notional_currency_2: Option<String>,
    /// The fixed rate of leg 2 of the trade, if applicable. Expressed as a percentage.
    pub interest_rate_2: Option<InterestRate>,
}

/// Additional reference data for a foreign exchange derivative instrument.
#[derive(Debug)]
pub struct FxDerivativeAttributes {
    /// The second currency of the currency pair.
    pub notional_currency_2: Option<String>,
    /// The type of underlying currency.
    pub fx_type: Option<FxType>,
}

/// Reference data for a single asset which underlies a derivative instrument.
#[derive(Debug)]
pub enum UnderlyingSingle {
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

/// Reference data for a basket of assets which underlie a derivative instrument.
#[derive(Debug)]
pub struct UnderlyingBasket {
    /// A list of ISINs of the financial instruments in the basket.
    pub isin: Vec<String>,
    /// A list of LEIs of issuers in the basket.
    pub issuer_lei: Vec<String>,
}


/// Reference data for the asset underlying a derivative. The underlying may be a single issuer,
/// instrument or index, or may be a basket of instruments or issuers. The relevant parameter
/// will be populated and the rest will be None.
#[derive(Debug)]
pub enum DerivativeUnderlying {
    /// Single instrument, index or issuer underlying a derivative instrument.
    Single(UnderlyingSingle),
    /// Basket of instruments or issuers underlying a derivative instrument.
    Basket(UnderlyingBasket),
}

/// Asset class-specific attributes of a derivative.
#[derive(Debug, Default)]
pub struct AssetClassSpecificAttributes {
    /// If the instrument is a commodity derivative, certain commodity-related attributes.
    pub commodity_attributes: Option<CommodityDerivativeAttributes>,
    /// If the instrument is an interest rate derivative, certain IR-related attributes.
    pub ir_attributes: Option<InterestRateDerivativeAttributes>,
    /// If the instrument is a foreign exchange derivative, certain FX-related attributes.
    pub fx_attributes: Option<FxDerivativeAttributes>,
}

/// Reference data for a derivative instrument.
///
/// Note that some other types of instrument can also have derivative-related attributes,
/// eg, some collective investment scheme (CFI code C) instruments.
#[derive(Debug, Default)]
pub struct DerivativeAttributes {
    /// Expiry date of the instrument.
    pub expiry_date: Option<NaiveDate>,
    /// Number of units of the underlying instrument represented by a single derivative
    /// contract. For a future or option on an index, the amount per index point.
    pub price_multiplier: Option<f64>,
    /// Description of the underlying asset or basket of assets.
    pub underlying: Option<DerivativeUnderlying>,
    /// If the derivative instrument is an option, whether it is a call or a put or whether
    /// it cannot be determined whether it is a call or a put at the time of execution.
    pub option_type: Option<OptionType>,
    /// Predetermined price at which the holder will have to buy or sell the underlying
    /// instrument, or an indication that the price cannot be determined at the time of execution.
    pub strike_price: Option<StrikePrice>,
    /// Indication of whether the option may be exercised only at a fixed date (European and
    /// Asian style), a series of pre-specified dates (Bermudan) or at any time during the
    /// life of the contract (American style).
    pub option_exercise_style: Option<OptionExerciseStyle>,
    /// Whether the financial instrument is cash settled or physically settled or delivery
    /// type cannot be determined at time of execution.
    pub delivery_type: Option<DeliveryType>,
    /// Certain additional attributes which are specific to the asset class of the derivative.
    pub asset_class_specific_attributes: Option<AssetClassSpecificAttributes>
}

/// A base class for financial instrument reference data.
#[derive(Debug)]
pub struct ReferenceData {
    /// The International Securities Identifier Number (ISO 6166) of the financial instrument.
    pub isin: String,
    /// The full name of the financial instrument. This should give a good indication of the
    /// issuer and the particulars of the instrument.
    pub full_name: String,
    /// The Classification of Financial Instruments code (ISO 10962) of the financial instrument.
    pub cfi: String,
    /// Whether the financial instrument falls within the definition of a "commodities derivative"
    /// under Article 2(1)(30) of Regulation (EU) No 600/2014.
    pub is_commodities_derivative: bool,
    /// The Legal Entity Identifier (ISO 17442) for the issuer. In certain cases, eg derivative
    /// instruments issued by the trading venue, this field will be populated with the trading
    /// venue operator's LEI.
    pub issuer_lei: String,
    /// The Financial Instrument Short Name (ISO 18774) for the financial instrument.
    pub fisn: String,
    /// Data relating to the trading or admission to trading of the financial instrument
    /// on a trading venue.
    pub trading_venue_attrs: TradingVenueAttributes,
    /// The currency in which the notional is denominated. For an interest rate or currency
    /// derivative contract, this will be the notional currency of leg 1, or the currency 1,
    /// of the pair. In the case of swaptions where the underlying swap is single currency,
    /// this will be the notional currency of the underlying swap. For swaptions where the
    /// underlying is multi-currency, this will be the notional currency of leg 1 of the swap.
    pub notional_currency: String,
    /// Technical attributes of the financial instrument.
    pub technical_attributes: Option<TechnicalAttributes>,
    /// If the instrument is a debt instrument, certain debt-related attributes.
    pub debt_attributes: Option<DebtAttributes>,
    /// If the instrument is a derivative, certain derivative-related attributes.
    pub derivative_attributes: Option<DerivativeAttributes>,
}

/// Reference data for a newly added financial instrument.
pub struct NewRecord(pub ReferenceData);

/// Modified reference data for a financial instrument.
pub struct ModifiedRecord(pub ReferenceData);

/// Reference data for a financial instrument that has ceased being traded on a trading venue.
pub struct TerminatedRecord(pub ReferenceData);


