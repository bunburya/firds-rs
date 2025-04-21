use strum_macros::{Display, EnumString};

/// Classification of commodity and emission allowances derivatives.
/// https://ec.europa.eu/finance/securities/docs/isd/mifid/rts/160714-rts-23-annex_en.pdf
#[derive(Debug, EnumString, Display)]
pub enum BaseProduct {
    #[strum(serialize = "AGRI")]
    Agricultural,
    #[strum(serialize = "NRGY")]
    Energy,
    #[strum(serialize = "ENVR")]
    Environmental,
    #[strum(serialize = "FRGT")]
    Freight,
    #[strum(serialize = "FRTL")]
    Fertilizer,
    #[strum(serialize = "INDP")]
    IndustrialProducts,
    #[strum(serialize = "METL")]
    Metals,
    #[strum(serialize = "MCEX")]
    MultiCommodityExotic,
    #[strum(serialize = "PAPR")]
    Paper,
    #[strum(serialize = "POLY")]
    Polypropylene,
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
    OtherC10,
    #[strum(serialize = "OTHR")]
    Other,
}

/// Sub-classification of products.
#[derive(Debug, EnumString, Display)]
pub enum SubProduct {
    // Agricultural (AGRI)
    #[strum(serialize = "GROS")]
    GrainsAndOilSeeds,
    #[strum(serialize = "SOFT")]
    Softs,
    #[strum(serialize = "POTA")]
    Potato,
    #[strum(serialize = "OOLI")]
    OliveOil,
    #[strum(serialize = "DIRY")]
    Dairy,
    #[strum(serialize = "FRST")]
    Forestry,
    #[strum(serialize = "SEAF")]
    Seafood,
    #[strum(serialize = "LSTK")]
    Livestock,
    #[strum(serialize = "GRIN")]
    Grain,

    // Energy (NRGY)
    #[strum(serialize = "ELEC")]
    Electricity,
    #[strum(serialize = "NGAS")]
    NaturalGas,
    #[strum(serialize = "OILP")]
    Oil,
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

    // Environmental (ENVR)
    #[strum(serialize = "EMIS")]
    Emissions,
    #[strum(serialize = "WTHR")]
    Weather,
    #[strum(serialize = "CRBR")]
    CarbonRelated,

    // Freight (FRGT)
    #[strum(serialize = "WETF")]
    Wet,
    #[strum(serialize = "DRYF")]
    Dry,
    #[strum(serialize = "CSHP")]
    ContainerShips,

    // Fertilizer (FRTL)
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

    // Industrial Products (INDP)
    #[strum(serialize = "CSTR")]
    Construction,
    #[strum(serialize = "MFTG")]
    Manufacturing,

    // Metals (METL)
    #[strum(serialize = "NPRM")]
    NonPrecious,
    #[strum(serialize = "PRME")]
    Precious,

    // Paper (PAPR)
    #[strum(serialize = "CBRD")]
    Containerboard,
    #[strum(serialize = "NSPT")]
    Newsprint,
    #[strum(serialize = "PULP")]
    Pulp,
    #[strum(serialize = "RCVP")]
    RecoveredPaper,

    // Polypropylene (POLY)
    #[strum(serialize = "PLST")]
    Plastic,

    // Other Commodity (OTHC)
    #[strum(serialize = "DLVR")]
    Deliverable,
    #[strum(serialize = "NDLV")]
    NonDeliverable,
}

/// Further sub-classifications of products.
#[derive(Debug, EnumString, Display)]
pub enum FurtherSubProduct {
    // Grains (GROS)
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

    // Softs (SOFT)
    #[strum(serialize = "CCOA")]
    Cocoa,
    #[strum(serialize = "ROBU")]
    RobustaCoffee,
    #[strum(serialize = "WHSG")]
    WhiteSugar,
    #[strum(serialize = "BRWN")]
    RawSugar,

    // Olive Oil (OOLI)
    #[strum(serialize = "LAMP")]
    Lampante,

    // Grain (GRIN)
    #[strum(serialize = "MWHT")]
    MillingWheat,

    // Electricity (ELEC)
    #[strum(serialize = "BSLD")]
    BaseLoad,
    #[strum(serialize = "FITR")]
    FinancialTransmissionRights,
    #[strum(serialize = "PKLD")]
    PeakLoad,
    #[strum(serialize = "OFFP")]
    OffPeak,

    // Natural Gas (NGAS)
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

    // Oil (OILP)
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

    // Emissions (EMIS)
    #[strum(serialize = "CERE")]
    Cer,
    #[strum(serialize = "ERUE")]
    Eru,
    #[strum(serialize = "EUAE")]
    Euae,
    #[strum(serialize = "EUAA")]
    Euaa,

    // Wet Freight (WETF)
    #[strum(serialize = "TNKR")]
    Tankers,

    // Dry Freight (DRYF)
    #[strum(serialize = "DBCR")]
    DryBulkCarriers,

    // Non-Precious Metals (NPRM)
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

    // Precious Metals (PRME)
    #[strum(serialize = "GOLD")]
    Gold,
    #[strum(serialize = "SLVR")]
    Silver,
    #[strum(serialize = "PTNM")]
    Platinum,
    #[strum(serialize = "PLDM")]
    Palladium,

    // Other
    #[strum(serialize = "OTHR")]
    Other,
}