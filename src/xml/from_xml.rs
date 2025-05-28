use std::str::FromStr;
use chrono::NaiveDate;
use crate::*;
use crate::xml::error::XmlError;
use crate::xml::iter_xml::Element;
use crate::xml::parse_utils::{child_or_none, date_or_none, datetime_or_none, parse_or_none, text_or_none};

pub trait FromXml: Sized {

    /// Try to create an instance of `Self` from the given [`Element`].
    fn from_xml(elem: &Element) -> Result<Self, XmlError>;

    /// Try to construct an instance of `Self` from the given [`Element`], if provided, otherwise
    /// return `None`. Returns a [`XmlError`] if an `Element` was encountered but could not be
    /// converted to an instance of `Self`.
    fn from_xml_option(elem: Option<&Element>) -> Result<Option<Self>, XmlError> {
        if let Some(e) = elem {
            Ok(Some(Self::from_xml(e)?))
        } else {
            Ok(None)
        }

    }
}

impl FromXml for Term {
    /// Parse a `Fltg/Term` XML element from FIRDS data into an [`Term`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        let number = elem.get_child("Val")?.text.parse::<i32>()?;
        let unit = TermUnit::try_from(elem.get_child("Unit")?.text.as_str())?;
        Ok(Self {
            number,
            unit
        })
    }
}

impl FromXml for StrikePrice {
    /// Parse a `DerivInstrmAttrbts/StrkPric` XML element from FIRDS into a [`StrikePrice`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
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
                return Err(XmlError::ElementNotFound)
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

impl FromXml for FloatingRate {
    /// Parse an XML element of type `FloatingInterestRate8` into a [`FloatingRate`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        let ref_rate_elem = elem.get_child("RefRate")?;
        let name = if let Some(text) = text_or_none(ref_rate_elem.find_child("Indx")) {
            Some(IndexName::from_str(text)?)
        } else if let Some(text) = text_or_none(ref_rate_elem.find_child("Nm")) {
            Some(IndexName::from_str(text)?)
        } else {
            None
        };
        Ok(Self {
            name,
            term: Term::from_xml_option(elem.find_child("Term"))?
        })
    }
}

impl FromXml for Index {
    /// Parse an XML element of type `FinancialInstrument58` into an [`Index`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self {
            isin: text_or_none(elem.find_child("ISIN")).map(String::from),
            name: FloatingRate::from_xml(elem.get_child("Nm")?)?
        })
    }
}

impl FromXml for TradingVenueAttributes {
    /// Parse a `TradgVnRltdAttrbts` XML element from FIRDS into a `TradingVenueAttributes` struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
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

impl FromXml for InterestRate {

    /// Parse an `IntrstRate` XML element from FIRDS data into an [`InterestRate`] struct.
    ///
    /// This is designed to work with `IntrstRate` elements of types `InterestRate6Choice` and
    /// `FloatingInterestRate8`. The difference is that the former specifies a basis point spread
    /// whereas the latter does not.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {

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

impl FromXml for PublicationPeriod {

    /// Parse a `PblctnPrd` XML element from FIRDS data into a [`PublicationPeriod`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
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

impl FromXml for TechnicalAttributes {

    /// Parse a `TechAttrbts` XML element from FIRDS data into a [`TechnicalAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self {
            relevant_competent_authority: text_or_none(elem.find_child("RlvntCmptntAuthrty"))
                .map(String::from),
            publication_period: PublicationPeriod::from_xml_option(elem.find_child("PblctnPrd"))?,
            relevant_trading_venue: text_or_none(elem.find_child("RlvntTradgVn")).map(String::from)
        })
    }
}

impl FromXml for DebtAttributes {
    /// Parse a `DebtInstrmAttrbts` XML element from FIRDS data into a [`DebtAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
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

impl FromXml for CommodityDerivativeAttributes {
    /// Parse a `DerivInstrmAttrbts/AsstClssSpcfcAttrbts/Cmmdty` XML element from FIRDS data into a
    /// [`CommodityDerivativeAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
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

impl FromXml for InterestRateDerivativeAttributes {
    /// Parse a `DerivInstrmAttrbts/AsstClssSpcfcAttrbts/Intrst` XML element from FIRDS into a
    /// [`InterestRateDerivativeAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self {
            reference_rate: FloatingRate::from_xml(elem.get_child("IntrstRate")?)?,
            interest_rate_1: InterestRate::from_xml_option(elem.find_child("FirstLegIntrstRate"))?,
            notional_currency_2: text_or_none(elem.find_child("OtherNtnlCcy")).map(String::from),
            interest_rate_2: InterestRate::from_xml_option(elem.find_child("OthrLegIntrstRate"))?
        })
    }
}

impl FromXml for FxDerivativeAttributes {
    /// Parse a `DerivInstrmAttrbts/AsstClssSpcfcAttrbts/FX` XML element from FIRDS into a
    /// [`FxDerivativeAttributes`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self {
            notional_currency_2: text_or_none(elem.find_child("OthrNtnlCcy")).map(String::from),
            fx_type: FxType::from_xml_option(elem.find_child("FxTp"))?
        })
    }
}

impl FromXml for UnderlyingSingle {
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        if let Some(child) = elem.find_first_child() {
            match child.local_name.as_str() {
                "ISIN" => Ok(Self::Isin(child.text.to_owned())),
                "LEI" => Ok(Self::Lei(child.text.to_owned())),
                "Indx" => Ok(Self::Index(Index::from_xml(child)?)),
                _ => Err(XmlError::Firds(crate::ParseError::Enum))
            }
        } else {
            Err(XmlError::ElementNotFound)
        }
    }
}

impl FromXml for UnderlyingBasket {
    /// Parse an XML element of type `FinancialInstrument53` into an [`UnderlyingBasket`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        let mut isin = vec![];
        let mut issuer_lei = vec![];
        for c in elem.iter_children() {
            match c.local_name.as_str() {
                "ISIN" => isin.push(c.text.to_owned()),
                "LEI" => issuer_lei.push(c.text.to_owned()),
                _ => return Err(XmlError::UnexpectedElement)
            }
        }
        Ok(Self {
            isin,
            issuer_lei,
        })
    }
}

impl FromXml for DerivativeUnderlying {
    /// Parse an XML element of type `FinancialInstrumentIdentification5Choice` into a
    /// [`DerivativeUnderlying`] struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        if let Some(child) = elem.find_first_child() {
            match child.local_name.as_str() {
                "Sngl" => Ok(Self::Single(UnderlyingSingle::from_xml(child)?)),
                "Bskt" => Ok(Self::Basket(UnderlyingBasket::from_xml(child)?)),
                _ => Err(XmlError::UnexpectedElement)
            }
        } else {
            Err(XmlError::ElementNotFound)
        }
    }
}

impl FromXml for AssetClassSpecificAttributes {
    /// Parse an XML element of type `AssetClass2__1` into an [`AssetClassSpecificAttributes`]
    /// struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        let mut attrs = Self::default();
        for c in elem.iter_children() {
            match c.local_name.as_str() {
                "Cmmdty" => attrs.commodity_attributes = Some(CommodityDerivativeAttributes::from_xml(c)?),
                "Intrst" => attrs.ir_attributes = Some(InterestRateDerivativeAttributes::from_xml(c)?),
                "FX" => attrs.fx_attributes = Some(FxDerivativeAttributes::from_xml(c)?),
                _ => return Err(XmlError::UnexpectedElement)
            }
        }
        Ok(attrs)
    }
}

impl FromXml for DerivativeAttributes {
    /// Parse an XML element of type `DerivativeInstrument5__1` into a [`DerivativeAttributes`]
    /// struct.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        let mut attrs = DerivativeAttributes::default();
        for c in elem.iter_children() {
            match c.local_name.as_str() {
                "XpryDt" =>
                    attrs.expiry_date = Some(NaiveDate::parse_from_str(&c.text, "%Y-%m-%d")?),
                "PricMltplr" =>
                    attrs.price_multiplier = Some(c.text.parse()?),
                "UndrlygInstrm" =>
                    attrs.underlying = Some(DerivativeUnderlying::from_xml(c)?),
                "OptnTp" =>
                    attrs.option_type = Some(OptionType::from_str(&c.text)?),
                "StrkPric" =>
                    attrs.strike_price = Some(StrikePrice::from_xml(c)?),
                "OptnExrcStyle" =>
                    attrs.option_exercise_style = Some(OptionExerciseStyle::from_str(&c.text)?),
                "DlvryTp" =>
                    attrs.delivery_type = Some(DeliveryType::from_str(&c.text)?),
                "AsstClssSpcfcAttrbts" =>
                    attrs.asset_class_specific_attributes
                        = Some(AssetClassSpecificAttributes::from_xml(c)?),
                _ => return Err(XmlError::UnexpectedElement)
            }
        }
        Ok(attrs)
    }
}

impl FromXml for ReferenceData {
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        let gen_attrs = elem.get_child("FinInstrmGnlAttrbts")?;
        Ok(Self {
            isin: gen_attrs.get_child("Id")?.text.to_owned(),
            full_name: gen_attrs.get_child("FullNm")?.text.to_owned(),
            cfi: gen_attrs.get_child("ClssfctnTp")?.text.to_owned(),
            is_commodities_derivative: gen_attrs.get_child("CmmdtyDerivInd")?.text.parse()?,
            issuer_lei: elem.get_child("Issr")?.text.to_owned(),
            fisn: gen_attrs.get_child("ShrtNm")?.text.to_owned(),
            trading_venue_attrs: TradingVenueAttributes::from_xml(
                elem.get_child("TradgVnRltdAttrbts")?
            )?,
            notional_currency: gen_attrs.get_child("NtnlCcy")?.text.to_owned(),
            technical_attributes: TechnicalAttributes::from_xml_option(
                elem.find_child("TechAttrbts")
            )?,
            debt_attributes: DebtAttributes::from_xml_option(
                elem.find_child("DebtInstrmAttrbts")
            )?,
            derivative_attributes: DerivativeAttributes::from_xml_option(
                elem.find_child("DerivInstrmAttrbts")
            )?
        })
    }
}

impl FromXml for NewRecord {
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self(ReferenceData::from_xml(elem)?))
    }
}

impl FromXml for ModifiedRecord {
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self(ReferenceData::from_xml(elem)?))
    }
}

impl FromXml for TerminatedRecord {
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self(ReferenceData::from_xml(elem)?))
    }
}

impl FromXml for DebtSeniority {
    /// Parse a `DebtSnrty` XML element from FIRDS data into a [`DebtSeniority`] enum.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self::from_str(&elem.text)?)
    }
}

impl FromXml for TransactionType {
    /// Parse a `TxTp` XML element from FIRDS data into a [`TransactionType`] enum.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self::from_str(&elem.text)?)
    }
}

impl FromXml for FinalPriceType {
    /// Parse a `FnlPricTp` XML element from FIRDS data into a [`FinalPriceType`] enum.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self::from_str(&elem.text)?)
    }
}

impl FromXml for FxType {
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self::from_str(&elem.text)?)
    }
}

impl FromXml for BaseProduct {
    /// Parse an appropriate XML element into a [`BaseProduct`] enum. The XML element can be of any
    /// kind that contains at least a `BasePdct` element and optionally `SubPdct` and `AddtlSubPdct`
    /// elements.
    fn from_xml(elem: &Element) -> Result<Self, XmlError> {
        Ok(Self::try_from_codes(
            &elem.get_child("BasePdct")?.text,
            text_or_none(elem.find_child("SubPdct")),
            text_or_none(elem.find_child("AddtlSubPdct")),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use crate::xml::iter_xml::XmlIterator;
    use crate::{
        CommodityDerivativeAttributes,
        DebtAttributes,
        DerivativeAttributes,
        FloatingRate,
        FxDerivativeAttributes,
        InterestRate,
        InterestRateDerivativeAttributes,
        ModifiedRecord,
        NewRecord,
        PublicationPeriod,
        ReferenceData,
        StrikePrice,
        TechnicalAttributes,
        TerminatedRecord,
        TradingVenueAttributes
    };
    use std::env::current_dir;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use crate::xml::from_xml::FromXml;

    fn get_firds_data_dir() -> PathBuf {
        current_dir().unwrap().join("test_data").join("firds_data")
    }

    fn test_parsing_xml<T: FromXml>(tag: &str, files: Vec<(&str, i32)>) {
        for (fname, count) in files {
            let path = get_firds_data_dir().join("esma").join(fname);
            println!("{path:?}");
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            let xml_iter = XmlIterator::new(tag, reader);
            let mut parsed = 0;
            for elem in xml_iter {
                assert!(elem.is_ok());
                let from_xml_res = T::from_xml(&elem.unwrap());
                assert!(from_xml_res.is_ok());
                parsed += 1;
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

    #[test]
    fn test_parse_ref_data() {
        test_parsing_xml::<ReferenceData>("RefData", vec![
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
    fn test_parse_modified() {
        test_parsing_xml::<ModifiedRecord>("ModfdRcrd", vec![
            ("DLTINS_20250205_02of02.xml", 183577),
            ("DLTINS_20250206_01of02.xml", 427787),
            ("DLTINS_20250204_01of02.xml", 396575),
            ("DLTINS_20250204_02of02.xml", 98287),
            ("DLTINS_20250203_01of01.xml", 16),
            ("DLTINS_20250202_01of01.xml", 7614),
            ("DLTINS_20250207_02of02.xml", 81638),
            ("DLTINS_20250205_01of02.xml", 369139),
            ("DLTINS_20250201_01of02.xml", 364297),
            ("DLTINS_20250207_01of02.xml", 370483),
            ("DLTINS_20250201_02of02.xml", 91768),
            ("DLTINS_20250206_02of02.xml", 13989),
        ])
    }

    #[test]
    fn test_parse_new() {
        test_parsing_xml::<NewRecord>("NewRcrd", vec![
            ("DLTINS_20250205_02of02.xml", 28341),
            ("DLTINS_20250206_01of02.xml", 38170),
            ("DLTINS_20250204_01of02.xml", 61742),
            ("DLTINS_20250204_02of02.xml", 19721),
            ("DLTINS_20250203_01of01.xml", 0),
            ("DLTINS_20250202_01of01.xml", 1924),
            ("DLTINS_20250207_02of02.xml", 34953),
            ("DLTINS_20250205_01of02.xml", 47349),
            ("DLTINS_20250201_01of02.xml", 83679),
            ("DLTINS_20250207_01of02.xml", 75119),
            ("DLTINS_20250201_02of02.xml", 27141),
            ("DLTINS_20250206_02of02.xml", 2763),
        ])
    }

    #[test]
    fn test_parse_terminated() {
        test_parsing_xml::<TerminatedRecord>("TermntdRcrd", vec![
            ("DLTINS_20250205_02of02.xml", 25490),
            ("DLTINS_20250206_01of02.xml", 34043),
            ("DLTINS_20250204_01of02.xml", 41681),
            ("DLTINS_20250204_02of02.xml", 10196),
            ("DLTINS_20250203_01of01.xml", 616),
            ("DLTINS_20250202_01of01.xml", 38436),
            ("DLTINS_20250207_02of02.xml", 19702),
            ("DLTINS_20250205_01of02.xml", 83512),
            ("DLTINS_20250201_01of02.xml", 52024),
            ("DLTINS_20250207_01of02.xml", 54398),
            ("DLTINS_20250201_02of02.xml", 24105),
            ("DLTINS_20250206_02of02.xml", 4368),
        ])
    }
}