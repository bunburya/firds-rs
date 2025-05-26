use crate::error::ProductError;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

/// Verifies that `fsp` is not `None`, and then calls the given function on the `&str` wrapped by
/// `fsp`. Used to conveniently construct subproduct enum members which require an associated
/// [`FurtherSubProduct`].
fn with_fsp_or_err<T>(
    fsp: Option<&str>,
    func: fn(&str) -> Result<T, ProductError>
) -> Result<T, ProductError> {
    if let Some(fsp) = fsp {
        func(fsp)
    } else {
        Err(ProductError::NoSubProduct)
    }
}

/// Verifies that `fsp` is `None` and, if so, returns `sp`. Used to check that we have not
/// encountered a further subproduct code when dealing with a subproduct that does not expect one.
fn without_fsp_or_err<T>(sp: T, fsp: Option<&str>) -> Result<T, ProductError> {
    if fsp.is_some() {
        Err(ProductError::BadSubProduct)
    } else {
        Ok(sp)
    }
}

fn optional_fsp<T: FromStr>(fsp: Option<&str>) -> Result<Option<T>, ProductError> 
where ProductError: From<<T as FromStr>::Err> {
    if let Some(s) = fsp {
        Ok(Some(T::from_str(&s)?))
    } else {
        Ok(None)
    }
}

trait SubProduct {
    fn try_from_codes(sub_prod: &str, further_sub_prod: Option<&str>) 
        -> Result<Self, ProductError> where Self: Sized;
}

/// Classification of commodity and emission allowances derivatives.
#[derive(Debug, Display)]
pub enum BaseProduct {
    #[strum(serialize = "AGRI")]
    Agricultural(AgriculturalSubProduct),
    #[strum(serialize = "NRGY")]
    Energy(EnergySubProduct),
    #[strum(serialize = "ENVR")]
    Environmental(EnvironmentalSubProduct),
    #[strum(serialize = "FRGT")]
    Freight(FreightSubProduct),
    #[strum(serialize = "FRTL")]
    Fertilizer(FertilizerSubProduct),
    #[strum(serialize = "INDP")]
    IndustrialProducts(IndustrialProductsSubProduct),
    #[strum(serialize = "METL")]
    Metals(MetalsSubProduct),
    #[strum(serialize = "MCEX")]
    MultiCommodityExotic,
    #[strum(serialize = "PAPR")]
    Paper(PaperSubProduct),
    #[strum(serialize = "POLY")]
    Polypropylene(PolypropyleneSubProduct),
    #[strum(serialize = "INFL")]
    Inflation,
    #[strum(serialize = "OEST")]
    OfficialEconomicStatistics,
    /// Other C10 (as defined in Table 10.1 of Section 10 of Annex III to Commission Delegated
    /// Regulation supplementing Regulation (EU) No 600/2014 of the European Parliament and of the
    /// Council with regard to regulatory technical standards on transparency requirements for
    /// trading venues and investment firms in respect of bonds, structured finance products,
    /// emission allowances and derivatives)
    #[strum(serialize = "OTHC")]
    OtherC10(OtherC10SubProduct),
    #[strum(serialize = "OTHR")]
    Other,
}

impl BaseProduct {
    pub fn try_from_codes(prod: &str, sub_prod: Option<&str>, further_sub_prod: Option<&str>) -> Result<Self, ProductError> {
        if let Some(sp) = sub_prod {
            match prod {
                "AGRI" => Ok(Self::Agricultural(
                    AgriculturalSubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "NRGY" => Ok(Self::Energy(
                    EnergySubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "ENVR" => Ok(Self::Environmental(
                    EnvironmentalSubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "FRGT" => Ok(Self::Freight(
                    FreightSubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "FRTL" => Ok(Self::Fertilizer(
                    FertilizerSubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "INDP" => Ok(Self::IndustrialProducts(
                    IndustrialProductsSubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "METL" => Ok(Self::Metals(
                    MetalsSubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "PAPR" => Ok(Self::Paper(
                    PaperSubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "POLY" => Ok(Self::Polypropylene(
                    PolypropyleneSubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                "OTHC" => Ok(Self::OtherC10(
                    OtherC10SubProduct::try_from_codes(sp, further_sub_prod)?
                )),
                _ => Err(ProductError::BadSubProduct)
            }
        } else {
            match prod {
                "MCEX" => Ok(Self::MultiCommodityExotic),
                "INFL" => Ok(Self::Inflation),
                "OEST" => Ok(Self::OfficialEconomicStatistics),
                "OTHR" => Ok(Self::Other),
                _ => Err(ProductError::BadProduct)
            }
        }
    }
}

/// Sub-classification of products.
#[derive(Debug, Display)]
pub enum AgriculturalSubProduct {
    GrainsAndOilSeeds(GrainsAndOilSeedsFurtherSubProduct),
    Softs(SoftsFurtherSubProduct),
    Potato,
    OliveOil(Option<OliveOilFurtherSubProduct>),
    Dairy,
    Forestry,
    Seafood,
    Livestock,
    Grain(Option<GrainFurtherSubProduct>),
}

impl SubProduct for AgriculturalSubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product {
            "GROS" => with_fsp_or_err(further_sub_product, |fsp| {
                Ok(Self::GrainsAndOilSeeds(GrainsAndOilSeedsFurtherSubProduct::try_from(fsp)?))
            }),
            "SOFT" => with_fsp_or_err(further_sub_product, |fsp| {
                Ok(Self::Softs(SoftsFurtherSubProduct::try_from(fsp)?))
            }),
            "POTA" => without_fsp_or_err(Self::Potato, further_sub_product),
            "OOLI" => Ok(Self::OliveOil(optional_fsp(further_sub_product)?)),
            "DIRY" => without_fsp_or_err(Self::Dairy, further_sub_product),
            "FRST" => without_fsp_or_err(Self::Forestry, further_sub_product),
            "SEAF" => without_fsp_or_err(Self::Seafood, further_sub_product),
            "LSTK" => without_fsp_or_err(Self::Livestock, further_sub_product),
            "GRIN" => Ok(Self::Grain(optional_fsp(further_sub_product)?)),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}


#[derive(Debug, Display)]
pub enum EnergySubProduct {
    #[strum(serialize = "ELEC")]
    Electricity(ElectricityFurtherSubProduct),
    #[strum(serialize = "NGAS")]
    NaturalGas(Option<NaturalGasFurtherSubProduct>),
    #[strum(serialize = "OILP")]
    Oil(Option<OilFurtherSubProduct>),
    #[strum(serialize = "COAL")]
    Coal,
    #[strum(serialize = "INRG")]
    InterEnergy,
    #[strum(serialize = "RNNG")]
    RenewableEnergy,
    #[strum(serialize = "LGHT")]
    LightEnds,
    #[strum(serialize = "DIST")]
    Distillates,
}

impl SubProduct for EnergySubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product {
            "ELEC" => with_fsp_or_err(further_sub_product, |fsp| {
                Ok(Self::Electricity(ElectricityFurtherSubProduct::try_from(fsp)?))
            }),
            "NGAS" => Ok(Self::NaturalGas(optional_fsp(further_sub_product)?)),
            "OILP" => Ok(Self::Oil(optional_fsp(further_sub_product)?)),
            "COAL" => without_fsp_or_err(Self::Coal, further_sub_product),
            "INRG" => without_fsp_or_err(Self::InterEnergy, further_sub_product),
            "RNNG" => without_fsp_or_err(Self::RenewableEnergy, further_sub_product),
            "LGHT" => without_fsp_or_err(Self::LightEnds, further_sub_product),
            "DIST" => without_fsp_or_err(Self::Distillates, further_sub_product),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

#[derive(Debug, Display)]
pub enum EnvironmentalSubProduct {
    #[strum(serialize = "EMIS")]
    Emissions(Option<EmissionsFurtherSubProduct>),
    #[strum(serialize = "WTHR")]
    Weather,
    #[strum(serialize = "CRBR")]
    CarbonRelated,
}

impl SubProduct for EnvironmentalSubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product {
            "EMIS" => Ok(Self::Emissions(optional_fsp(further_sub_product)?)),
            "WTHR" => without_fsp_or_err(Self::Weather, further_sub_product),
            "CRBR" => without_fsp_or_err(Self::CarbonRelated, further_sub_product),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

#[derive(Debug, Display)]
pub enum FreightSubProduct {
    Wet(Option<WetFreightFurtherSubProduct>),
    Dry(Option<DryFreightFurtherSubProduct>),
    ContainerShips,
}

impl SubProduct for FreightSubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product {
            "WETF" => Ok(Self::Wet(optional_fsp(further_sub_product)?)),
            "DRYF" => Ok(Self::Dry(optional_fsp(further_sub_product)?)),
            "CSHP" => without_fsp_or_err(Self::ContainerShips, further_sub_product),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

#[derive(Debug, Display)]
pub enum FertilizerSubProduct {
    #[strum(serialize = "AMMO")]
    Ammonia,
    #[strum(serialize = "DAPH")]
    Dap,
    #[strum(serialize = "PTSH")]
    Potash,
    #[strum(serialize = "SLPH")]
    Sulphur,
    #[strum(serialize = "UREA")]
    Urea,
    #[strum(serialize = "UAAN")]
    Uan,
}

impl SubProduct for FertilizerSubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product {
            "AMMO" => without_fsp_or_err(Self::Ammonia, further_sub_product),
            "DAPH" => without_fsp_or_err(Self::Dap, further_sub_product),
            "PTSH" => without_fsp_or_err(Self::Potash, further_sub_product),
            "SLPH" => without_fsp_or_err(Self::Sulphur, further_sub_product),
            "UREA" => without_fsp_or_err(Self::Urea, further_sub_product),
            "UAAN" => without_fsp_or_err(Self::Uan, further_sub_product),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

#[derive(Debug, EnumString, Display)]
pub enum IndustrialProductsSubProduct {
    #[strum(serialize = "CSTR")]
    Construction,
    #[strum(serialize = "MFTG")]
    Manufacturing,
}

impl SubProduct for IndustrialProductsSubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product { 
            "CSTR" => without_fsp_or_err(Self::Construction, further_sub_product),
            "MFTG" => without_fsp_or_err(Self::Manufacturing, further_sub_product),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

#[derive(Debug, Display)]
pub enum MetalsSubProduct {
    #[strum(serialize = "NPRM")]
    NonPrecious(NonPreciousMetalsFurtherSubProduct),
    #[strum(serialize = "PRME")]
    Precious(PreciousMetalsFurtherSubProduct),
}

impl SubProduct for MetalsSubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product { 
            "NPRM" => with_fsp_or_err(further_sub_product, |fsp| {
                Ok(Self::NonPrecious(NonPreciousMetalsFurtherSubProduct::try_from(fsp)?))
            }),
            "PRME" => with_fsp_or_err(further_sub_product, |fsp| {
                Ok(Self::Precious(PreciousMetalsFurtherSubProduct::try_from(fsp)?))
            }),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

#[derive(Debug, Display)]
pub enum PaperSubProduct {
    Containerboard,
    Newsprint,
    Pulp,
    RecoveredPaper,
}

impl SubProduct for PaperSubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product { 
            "CBRD" => without_fsp_or_err(Self::Containerboard, further_sub_product),
            "NSPT" => without_fsp_or_err(Self::Newsprint, further_sub_product),
            "PULP" => without_fsp_or_err(Self::Pulp, further_sub_product),
            "RCVP" => without_fsp_or_err(Self::RecoveredPaper, further_sub_product),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

#[derive(Debug, Display)]
pub enum PolypropyleneSubProduct {
    Plastic,
}

impl SubProduct for PolypropyleneSubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product { 
            "PLST" => without_fsp_or_err(Self::Plastic, further_sub_product),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

#[derive(Debug, EnumString, Display)]
pub enum OtherC10SubProduct {
    #[strum(serialize = "DLVR")]
    Deliverable,
    #[strum(serialize = "NDLV")]
    NonDeliverable,
}

impl SubProduct for OtherC10SubProduct {
    fn try_from_codes(
        sub_product: &str,
        further_sub_product: Option<&str>
    ) -> Result<Self, ProductError> {
        match sub_product {
            "DLVR" => without_fsp_or_err(Self::Deliverable, further_sub_product),
            "NDLV" => without_fsp_or_err(Self::NonDeliverable, further_sub_product),
            _ => Err(ProductError::BadSubProduct)
        }
    }
}

/// Further sub-classifications of products.
#[derive(Debug, EnumString, Display)]
pub enum GrainsAndOilSeedsFurtherSubProduct {
    #[strum(serialize = "FWHT")]
    FeedWheat,
    #[strum(serialize = "SOYB")]
    Soybeans,
    #[strum(serialize = "CORN")]
    Corn,
    #[strum(serialize = "RPSD")]
    Rapeseed,
    #[strum(serialize = "RICE")]
    Rice,
    #[strum(serialize = "OTHR")]
    Other,
}

#[derive(Debug, EnumString, Display)]
pub enum SoftsFurtherSubProduct {
    #[strum(serialize = "CCOA")]
    Cocoa,
    #[strum(serialize = "ROBU")]
    RobustaCoffee,
    #[strum(serialize = "WHSG")]
    WhiteSugar,
    #[strum(serialize = "BRWN")]
    RawSugar,
    #[strum(serialize = "OTHR")]
    Other,
}

#[derive(Debug, EnumString, Display)]
pub enum OliveOilFurtherSubProduct {
    #[strum(serialize = "LAMP")]
    Lampante,
}

#[derive(Debug, EnumString, Display)]
pub enum GrainFurtherSubProduct {
    #[strum(serialize = "MWHT")]
    MillingWheat,
}

#[derive(Debug, EnumString, Display)]
pub enum ElectricityFurtherSubProduct {
    #[strum(serialize = "BSLD")]
    BaseLoad,
    #[strum(serialize = "FITR")]
    FinancialTransmissionRights,
    #[strum(serialize = "PKLD")]
    PeakLoad,
    #[strum(serialize = "OFFP")]
    OffPeak,
    #[strum(serialize = "OTHR")]
    Other,
}

#[derive(Debug, EnumString, Display)]
pub enum NaturalGasFurtherSubProduct {
    #[strum(serialize = "GASP")]
    Gaspool,
    #[strum(serialize = "LNGG")]
    Lng,
    #[strum(serialize = "NBPG")]
    Nbp,
    #[strum(serialize = "NCGG")]
    Ncg,
    #[strum(serialize = "TTFG")]
    Ttf,
}

#[derive(Debug, EnumString, Display)]
pub enum OilFurtherSubProduct {
    #[strum(serialize = "BAKK")]
    Bakken,
    #[strum(serialize = "BDSL")]
    Biodiesel,
    #[strum(serialize = "BRNT")]
    Brent,
    #[strum(serialize = "BRNX")]
    BrentNx,
    #[strum(serialize = "CNDA")]
    Canadian,
    #[strum(serialize = "COND")]
    Condensate,
    #[strum(serialize = "DSEL")]
    Diesel,
    #[strum(serialize = "DUBA")]
    Dubai,
    #[strum(serialize = "ESPO")]
    Espo,
    #[strum(serialize = "ETHA")]
    Ethanol,
    #[strum(serialize = "FUEL")]
    Fuel,
    #[strum(serialize = "FOIL")]
    FuelOil,
    #[strum(serialize = "GOIL")]
    Gasoil,
    #[strum(serialize = "GSLN")]
    Gasoline,
    #[strum(serialize = "HEAT")]
    HeatingOil,
    #[strum(serialize = "JTFL")]
    JetFuel,
    #[strum(serialize = "KERO")]
    Kerosene,
    #[strum(serialize = "LLSO")]
    LightLouisianaSweet,
    #[strum(serialize = "MARS")]
    Mars,
    #[strum(serialize = "NAPH")]
    Naphtha,
    #[strum(serialize = "NGLO")]
    Ngl,
    #[strum(serialize = "TAPI")]
    Tapis,
    #[strum(serialize = "URAL")]
    Urals,
    #[strum(serialize = "WTIO")]
    Wti,
}

#[derive(Debug, EnumString, Display)]
pub enum EmissionsFurtherSubProduct {
    #[strum(serialize = "CERE")]
    Cer,
    #[strum(serialize = "ERUE")]
    Eru,
    #[strum(serialize = "EUAE")]
    Euae,
    #[strum(serialize = "EUAA")]
    Euaa,
    #[strum(serialize = "OTHR")]
    Other,
}

#[derive(Debug, EnumString, Display)]
pub enum WetFreightFurtherSubProduct {
    #[strum(serialize = "TNKR")]
    Tankers,
}

#[derive(Debug, EnumString, Display)]
pub enum DryFreightFurtherSubProduct {
    #[strum(serialize = "DBCR")]
    DryBulkCarriers,
}

#[derive(Debug, EnumString, Display)]
pub enum NonPreciousMetalsFurtherSubProduct {
    #[strum(serialize = "ALUM")]
    Aluminium,
    #[strum(serialize = "ALUA")]
    AluminiumAlloy,
    #[strum(serialize = "CBLT")]
    Cobalt,
    #[strum(serialize = "COPR")]
    Copper,
    #[strum(serialize = "IRON")]
    IronOre,
    #[strum(serialize = "LEAD")]
    Lead,
    #[strum(serialize = "MOLY")]
    Molybdenum,
    #[strum(serialize = "NASC")]
    Nasaac,
    #[strum(serialize = "NICK")]
    Nickel,
    #[strum(serialize = "STEL")]
    Steel,
    #[strum(serialize = "TINN")]
    Tin,
    #[strum(serialize = "ZINC")]
    Zinc,
    #[strum(serialize = "OTHR")]
    Other,
}

#[derive(Debug, EnumString, Display)]
pub enum PreciousMetalsFurtherSubProduct {
    #[strum(serialize = "GOLD")]
    Gold,
    #[strum(serialize = "SLVR")]
    Silver,
    #[strum(serialize = "PTNM")]
    Platinum,
    #[strum(serialize = "PLDM")]
    Palladium,
    #[strum(serialize = "OTHR")]
    Other,
}