-- PRODUCT ENUMS

CREATE TABLE IF NOT EXISTS BaseProduct (
    code CHAR(4) PRIMARY KEY,
    description TEXT
);
INSERT OR IGNORE INTO BaseProduct (code, description) VALUES
    ('AGRI', 'Agricultural'),
    ('NRGY', 'Energy'),
    ('ENVR', 'Environmental'),
    ('FRGT', 'Freight'),
    ('FRTL', 'Fertilizer'),
    ('INDP', 'IndustrialProducts'),
    ('METL', 'Metals'),
    ('MCEX', 'MultiCommodityExotic'),
    ('PAPR', 'Paper'),
    ('POLY', 'Polypropylene'),
    ('INFL', 'Inflation'),
    ('OEST', 'OfficialEconomicStatistics'),
    ('OTHC', 'OtherC10'),
    ('OTHR', 'Other');

CREATE TABLE IF NOT EXISTS SubProduct (
    code CHAR(4) PRIMARY KEY,
    description TEXT
);
INSERT OR IGNORE INTO SubProduct (code, description) VALUES
    -- Energy
    ('ELEC', 'Electricity'),
    ('NGAS', 'NaturalGas'),
    ('OILP', 'Oil'),
    ('COAL', 'Coal'),
    ('INRG', 'InterEnergy'),
    ('RNNG', 'RenewableEnergy'),
    ('LGHT', 'LightEnds'),
    ('DIST', 'Distillates'),
    -- Fertilizer
    ('AMMO', 'Ammonia'),
    ('DAPH', 'Dap'),
    ('PTSH', 'Potash'),
    ('SLPH', 'Sulphur'),
    ('UREA', 'Urea'),
    ('UAAN', 'Uan'),
    -- Industrial
    ('CSTR', 'Construction'),
    ('MFTG', 'Manufacturing'),
    -- Metal
    ('NPRM', 'NonPrecious'),
    ('PRME', 'Precious'),
    -- Paper
    ('CBRD', 'Containerboard'),
    ('NSPT', 'Newsprint'),
    ('PULP', 'Pulp'),
    ('RCVP', 'RecoveredPaper'),
    -- Polypropylene
    ('PLST', 'Plastic'),
    -- Other C10
    ('DLVR', 'Deliverable'),
    ('NDLV', 'NonDeliverable');

CREATE TABLE IF NOT EXISTS FurtherSubProduct (
    code CHAR(4) PRIMARY KEY,
    description TEXT
);
INSERT OR IGNORE INTO FurtherSubProduct (code, description) VALUES
    -- Grain and oil seeds
    ('FWHT', 'FeedWheat'),
    ('SOYB', 'Soybeans'),
    ('CORN', 'Corn'),
    ('RPSD', 'Rapeseed'),
    ('RICE', 'Rice'),
    -- Softs
    ('CCOA', 'Cocoa'),
    ('ROBU', 'RobustaCoffee'),
    ('WHSG', 'WhiteSugar'),
    ('BRWN', 'RawSugar'),
    -- Olive oil
    ('LAMP', 'Lampante'),
    -- Grain
    ('MWHT', 'MillingWheat'),
    -- Electricity
    ('BSLD', 'BaseLoad'),
    ('FITR', 'FinancialTransmissionRights'),
    ('PKLD', 'PeakLoad'),
    ('OFFP', 'OffPeak'),
    -- Natural gas
    ('GASP', 'Gaspool'),
    ('LNGG', 'Lng'),
    ('NBPG', 'Nbp'),
    ('NCGG', 'Ncg'),
    ('TTFG', 'Ttf'),
    -- Oil
    ('BAKK', 'Bakken'),
    ('BDSL', 'Biodiesel'),
    ('BRNT', 'Brent'),
    ('BRNX', 'BrentNx'),
    ('CNDA', 'Canadian'),
    ('COND', 'Condensate'),
    ('DSEL', 'Diesel'),
    ('DUBA', 'Dubai'),
    ('ESPO', 'Espo'),
    ('ETHA', 'Ethanol'),
    ('FUEL', 'Fuel'),
    ('FOIL', 'FuelOil'),
    ('GOIL', 'Gasoil'),
    ('GSLN', 'Gasoline'),
    ('HEAT', 'HeatingOil'),
    ('JTFL', 'JetFuel'),
    ('KERO', 'Kerosene'),
    ('LLSO', 'LightLouisianaSweet'),
    ('MARS', 'Mars'),
    ('NAPH', 'Naphtha'),
    ('NGLO', 'Ngl'),
    ('TAPI', 'Tapis'),
    ('URAL', 'Urals'),
    ('WTIO', 'Wti'),
    -- Emissions
    ('CERE', 'Cer'),
    ('ERUE', 'Eru'),
    ('EUAE', 'Euae'),
    ('EUAA', 'Euaa'),
    -- Wet freight
    ('TNKR', 'Tankers'),
    -- Dry Freight
    ('DBCR', 'DryBulkCarriers'),
    -- Non-precious metals
    ('ALUM', 'Aluminium'),
    ('ALUA', 'AluminiumAlloy'),
    ('CBLT', 'Cobalt'),
    ('COPR', 'Copper'),
    ('IRON', 'IronOre'),
    ('LEAD', 'Lead'),
    ('MOLY', 'Molybdenum'),
    ('NASC', 'Nasaac'),
    ('NICK', 'Nickel'),
    ('STEL', 'Steel'),
    ('TINN', 'Tin'),
    ('ZINC', 'Zinc'),
    -- Precious metals
    ('GOLD', 'Gold'),
    ('SLVR', 'Silver'),
    ('PTNM', 'Platinum'),
    ('PLDM', 'Palladium'),

    -- Used in various sub-products
    ('OTHR', 'Other');

-- ENUMS

CREATE TABLE IF NOT EXISTS TermUnit (
    code CHAR(8) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO TermUnit (code, label) VALUES
    ('DAYS', 'Days'),
    ('WEEK', 'Week'),
    ('MNTH', 'Month'),
    ('YEAR', 'Year');

CREATE TABLE IF NOT EXISTS IndexCode (
    code CHAR(4) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO IndexCode (code, label) VALUES
    ('EONA', 'Eonia'),
    ('EONS', 'EoniaSwap'),
    ('EURO', 'Euribor'),
    ('EUCH', 'EuroSwiss'),
    ('GCFR', 'GcfRepo'),
    ('ISDA', 'Isdafix'),
    ('LIBI', 'Libid'),
    ('LIBO', 'Libor'),
    ('MAAA', 'MuniAaa'),
    ('PFAN', 'Pfandbriefe'),
    ('TIBO', 'Tibor'),
    ('STBO', 'Stibor'),
    ('BBSW', 'Bbsw'),
    ('JIBA', 'Jibar'),
    ('BUBO', 'Bubor'),
    ('CDOR', 'Cdor'),
    ('CIBO', 'Cibor'),
    ('MOSP', 'Mosprim'),
    ('NIBO', 'Nibor'),
    ('PRBO', 'Pribor'),
    ('TLBO', 'Telbor'),
    ('WIBO', 'Wibor'),
    ('TREA', 'Treasury'),
    ('SWAP', 'Swap'),
    ('FUSW', 'FutureSwap');

CREATE TABLE IF NOT EXISTS IndexName (
    id INTEGER PRIMARY KEY,
    index_code CHAR(4) REFERENCES IndexCode(code),
    text TEXT,
    -- Enforce that exactly one of (index_code, text) is non-null
    CHECK (
       (index_code IS NOT NULL AND text IS NULL)
           OR
       (index_code IS NULL AND text IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS DebtSeniority (
    code CHAR(4) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO DebtSeniority (code, label) VALUES
    ('SNDB', 'Senior'),
    ('MZZD', 'Mezzanine'),
    ('SBOD', 'Subordinated'),
    ('JUND', 'Junior');

CREATE TABLE IF NOT EXISTS OptionType (
    code CHAR(4) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO OptionType (code, label) VALUES
    ('PUTO', 'Put'),
    ('CALL', 'Call'),
    ('OTHR', 'Other');

CREATE TABLE IF NOT EXISTS OptionExerciseStyle (
    code CHAR(4) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO OptionExerciseStyle (code, label) VALUES
    ('EURO', 'European'),
    ('AMER', 'American'),
    ('ASIA', 'Asian'),
    ('BERM', 'Bermudan'),
    ('OTHR', 'Other');

CREATE TABLE IF NOT EXISTS DeliveryType (
    code CHAR(4) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO DeliveryType (code, label) VALUES
    ('PHYS', 'Physical'),
    ('CASH', 'Cash'),
    ('OPTL', 'Optional');

CREATE TABLE IF NOT EXISTS TransactionType (
    code CHAR(4) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO TransactionType (code, label) VALUES
    ('FUTR', 'Futures'),
    ('OPTN', 'Options'),
    ('TAPO', 'Tapos'),
    ('SWAP', 'Swaps'),
    ('MINI', 'Minis'),
    ('OTCT', 'OverTheCounter'),
    ('ORIT', 'Outright'),
    ('CRCK', 'Crack'),
    ('DIFF', 'Differential'),
    ('OTHR', 'Other');

CREATE TABLE IF NOT EXISTS FinalPriceType (
    code CHAR(4) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO FinalPriceType (code, label) VALUES
    ('ARGM', 'ArgusMcCloskey'),
    ('BLTC', 'Baltic'),
    ('EXOF', 'Exchange'),
    ('GBCL', 'GlobalCoal'),
    ('IHSM', 'IHSMarkit'),
    ('PLAT', 'Platts'),
    ('OTHR', 'Other');

CREATE TABLE IF NOT EXISTS FxType (
    code CHAR(4) PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO FxType (code, label) VALUES
    ('FXCR', 'CrossRates'),
    ('FXEM', 'EmergingMarkets'),
    ('FXMJ', 'Majors');

CREATE TABLE IF NOT EXISTS StrikePriceType (
    code TEXT PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT OR IGNORE INTO StrikePriceType (code, label) VALUES
    ('MONETARY_VALUE', 'MonetaryValue'),
    ('PERCENTAGE', 'Percentage'),
    ('YIELD', 'Yield'),
    ('BASIS_POINTS', 'BasisPoints'),
    ('NO_PRICE', 'NoPrice');

-- MODEL

CREATE TABLE IF NOT EXISTS Term (
    id INTEGER PRIMARY KEY,
    number INTEGER NOT NULL,
    unit CHAR(4) NOT NULL, -- maps to TermUnit enum
    FOREIGN KEY (unit) REFERENCES TermUnit(code)
);

CREATE TABLE IF NOT EXISTS StrikePrice (
    id INTEGER PRIMARY KEY,
    price_type CHAR(4) NOT NULL,
    price DOUBLE PRECISION,
    pending BOOLEAN NOT NULL,
    currency TEXT,
    FOREIGN KEY (price_type) REFERENCES StrikePriceType(code)
);

CREATE TABLE IF NOT EXISTS FloatingRate (
    id INTEGER PRIMARY KEY,
    name TEXT,
    term_id INTEGER,
    FOREIGN KEY (term_id) REFERENCES Term(id)
);

-- Table for Index struct. Called FirdsIndex to avoid conflict with INDEX keyword in SQL.
CREATE TABLE IF NOT EXISTS FirdsIndex (
    id INTEGER PRIMARY KEY,
    isin VARCHAR(12),
    name_id INTEGER NOT NULL,
    FOREIGN KEY (name_id) REFERENCES FloatingRate(id)
);

CREATE TABLE IF NOT EXISTS TradingVenueAttributes (
    id INTEGER PRIMARY KEY,
    trading_venue CHAR(4) NOT NULL,
    requested_admission BOOLEAN NOT NULL,
    approval_date TIMESTAMP,
    request_date TIMESTAMP,
    admission_or_first_trade_date TIMESTAMP,
    termination_date TIMESTAMP
);

CREATE TABLE IF NOT EXISTS InterestRate (
    id INTEGER PRIMARY KEY,
    fixed DOUBLE PRECISION,
    floating_rate_id INTEGER,
    spread INTEGER,
    FOREIGN KEY (floating_rate_id) REFERENCES FloatingRate(id),
    CHECK (
        (fixed IS NOT NULL AND floating_rate_id IS NULL AND spread IS NULL)
            OR
        (fixed IS NULL AND floating_rate_id IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS PublicationPeriod (
    id INTEGER PRIMARY KEY,
    from_date DATE NOT NULL,
    to_date DATE
);

CREATE TABLE IF NOT EXISTS TechnicalAttributes (
    id INTEGER PRIMARY KEY,
    relevant_competent_authority TEXT,
    publication_period_id INTEGER,
    relevant_trading_venue CHAR(4),
    FOREIGN KEY (publication_period_id) REFERENCES PublicationPeriod(id)
);

CREATE TABLE IF NOT EXISTS DebtAttributes (
    id INTEGER PRIMARY KEY,
    total_issued_amount DOUBLE PRECISION NOT NULL,
    maturity_date DATE,
    nominal_currency TEXT NOT NULL,
    nominal_value_per_unit DOUBLE PRECISION NOT NULL,
    interest_rate_id INTEGER NOT NULL,
    seniority CHAR(4),
    FOREIGN KEY (interest_rate_id) REFERENCES InterestRate(id),
    FOREIGN KEY (seniority) REFERENCES DebtSeniority(code)
);

CREATE TABLE IF NOT EXISTS CommodityDerivativeAttributes (
    id INTEGER PRIMARY KEY,
    product CHAR(4) NOT NULL,
    subproduct CHAR(4),
    -- Note: Products and subproducts are flattened as otherwise it would add a fair bit of complexity to the schema.
    further_subproduct CHAR(4),
    transaction_type CHAR(4),
    final_price_type CHAR(4),
    FOREIGN KEY (transaction_type) REFERENCES TransactionType(code),
    FOREIGN KEY (final_price_type) REFERENCES FinalPriceType(code)
);

CREATE TABLE IF NOT EXISTS InterestRateDerivativeAttributes (
    id INTEGER PRIMARY KEY,
    reference_rate_id INTEGER NOT NULL,
    interest_rate_1_id INTEGER,
    notional_currency_2 TEXT,
    interest_rate_2_id INTEGER,
    FOREIGN KEY (reference_rate_id) REFERENCES FloatingRate(id),
    FOREIGN KEY (interest_rate_1_id) REFERENCES InterestRate(id),
    FOREIGN KEY (interest_rate_2_id) REFERENCES InterestRate(id)
);

CREATE TABLE IF NOT EXISTS FxDerivativeAttributes (
    id INTEGER PRIMARY KEY,
    notional_currency_2 TEXT,
    fx_type CHAR(4), 
    FOREIGN KEY (fx_type) REFERENCES FxType(code)
);

CREATE TABLE IF NOT EXISTS UnderlyingBasket (
    id INTEGER PRIMARY KEY
    -- Underlying ISINs and issuer_lei stored in separate tables
);

CREATE TABLE IF NOT EXISTS UnderlyingBasketIsin (
    basket_id INTEGER NOT NULL,
    isin CHAR(12) NOT NULL,
    FOREIGN KEY (basket_id) REFERENCES UnderlyingBasket(id)
);

CREATE TABLE IF NOT EXISTS UnderlyingBasketIssuerLei (
    basket_id INTEGER NOT NULL,
    issuer_lei CHAR(20) NOT NULL,
    FOREIGN KEY (basket_id) REFERENCES UnderlyingBasket(id)
);

CREATE TABLE IF NOT EXISTS UnderlyingSingle (
    id INTEGER PRIMARY KEY,
    isin CHAR(12),
    index_id INTEGER,
    lei CHAR(20),
    FOREIGN KEY (index_id) REFERENCES FirdsIndex(id),
    CHECK (
        (isin IS NOT NULL AND index_id IS NULL AND lei IS NULL)
            OR
        (isin IS NULL AND index_id IS NOT NULL AND lei IS NULL)
            OR
        (isin IS NULL AND index_id IS NULL AND lei IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS DerivativeUnderlying (
    id INTEGER PRIMARY KEY,
    single_id INTEGER,
    basket_id INTEGER,
    FOREIGN KEY (single_id) REFERENCES UnderlyingSingle(id),
    FOREIGN KEY (basket_id) REFERENCES UnderlyingBasket(id),
    CHECK (
        (single_id IS NOT NULL AND basket_id IS NULL)
            OR
        (single_id IS NULL AND basket_id IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS AssetClassSpecificAttributes (
    id INTEGER PRIMARY KEY,
    commodity_attributes_id INTEGER,
    ir_attributes_id INTEGER,
    fx_attributes_id INTEGER,
    FOREIGN KEY (commodity_attributes_id) REFERENCES CommodityDerivativeAttributes(id),
    FOREIGN KEY (ir_attributes_id) REFERENCES InterestRateDerivativeAttributes(id),
    FOREIGN KEY (fx_attributes_id) REFERENCES FxDerivativeAttributes(id)
);

CREATE TABLE IF NOT EXISTS DerivativeAttributes (
    id INTEGER PRIMARY KEY,
    expiry_date DATE,
    price_multiplier DOUBLE PRECISION,
    underlying_id INTEGER,
    option_type CHAR(4),
    strike_price_id INTEGER,
    option_exercise_style CHAR(4),
    delivery_type CHAR(4),
    asset_class_specific_attributes_id INTEGER,
    FOREIGN KEY (underlying_id) REFERENCES DerivativeUnderlying(id),
    FOREIGN KEY (option_type) REFERENCES OptionType(code),
    FOREIGN KEY (strike_price_id) REFERENCES StrikePrice(id),
    FOREIGN KEY (option_exercise_style) REFERENCES OptionExerciseStyle(code),
    FOREIGN KEY (delivery_type) REFERENCES DeliveryType(code),
    FOREIGN KEY (asset_class_specific_attributes_id) REFERENCES AssetClassSpecificAttributes(id)
);

CREATE TABLE IF NOT EXISTS ReferenceData (
    id SERIAL PRIMARY KEY,
    isin CHAR(12) NOT NULL,
    full_name TEXT NOT NULL,
    cfi CHAR(6) NOT NULL,
    is_commodities_derivative BOOLEAN NOT NULL,
    issuer_lei CHAR(20) NOT NULL,
    fisn VARCHAR(35) NOT NULL,
    trading_venue_attrs_id INTEGER NOT NULL,
    notional_currency TEXT NOT NULL,
    technical_attributes_id INTEGER,
    debt_attributes_id INTEGER,
    derivative_attributes_id INTEGER,
    FOREIGN KEY (trading_venue_attrs_id) REFERENCES TradingVenueAttributes(id),
    FOREIGN KEY (technical_attributes_id) REFERENCES TechnicalAttributes(id),
    FOREIGN KEY (debt_attributes_id) REFERENCES DebtAttributes(id),
    FOREIGN KEY (derivative_attributes_id) REFERENCES DerivativeAttributes(id)
);

-- Tables for NewRecord, ModifiedRecord, TerminatedRecord (all just wrappers around ReferenceData)
CREATE TABLE IF NOT EXISTS NewRecord (
    id INTEGER PRIMARY KEY,
    reference_data_id INTEGER NOT NULL,
    FOREIGN KEY (reference_data_id) REFERENCES ReferenceData(id)
);

CREATE TABLE IF NOT EXISTS ModifiedRecord (
    id INTEGER PRIMARY KEY,
    reference_data_id INTEGER NOT NULL,
    FOREIGN KEY (reference_data_id) REFERENCES ReferenceData(id)
);

CREATE TABLE IF NOT EXISTS TerminatedRecord (
    id INTEGER PRIMARY KEY,
    reference_data_id INTEGER NOT NULL,
    FOREIGN KEY (reference_data_id) REFERENCES ReferenceData(id)
);
