use crate::sql::error::SqlError;
use crate::*;
use chrono::{Duration, NaiveDate};
use sqlx::{Executor, SqliteTransaction};

/// A wrapper around a [`ReferenceData`] object which contains some additional data necessary for
/// storing modifications to the data.
pub struct RefDataDbEntry {
    /// The reference data itself.
    pub ref_data: ReferenceData,
    /// Whether this entry represents the latest record of the relevant instrument.
    pub latest_record: bool,
    /// The date from which this entry is valid.
    pub valid_from: NaiveDate,
    /// The date, if any, to which this entry was valid.
    pub valid_to: Option<NaiveDate>,
}

impl RefDataDbEntry {
    pub fn new(
        ref_data: ReferenceData,
        latest_record: bool,
        valid_from: NaiveDate,
        valid_to: Option<NaiveDate>
    ) -> Self {
        Self {
            ref_data,
            latest_record,
            valid_from,
            valid_to
        }
    }

    // Create a new [`RefDataDbEntry`] which represents the latest record of the given data.
    pub fn new_latest(ref_data: ReferenceData, valid_from: NaiveDate) -> Self {
        Self {
            ref_data,
            latest_record: true,
            valid_from,
            valid_to: None
        }
    }

    pub async fn mark_prev_record(&self, tx: &mut SqliteTransaction<'_>) -> Result<u64, SqlError> {
        let prev_valid_to = self.valid_from - Duration::days(1);
        let prev_valid_to_str = prev_valid_to.to_string();
        let query = sqlx::query!(
            r#"
                UPDATE ReferenceData
                SET valid_to = ?,
                    latest_record = false
                FROM TradingVenueAttributes
                WHERE ReferenceData.trading_venue_attrs_id = TradingVenueAttributes.id
                AND ReferenceData.isin = ? AND TradingVenueAttributes.trading_venue = ?
                AND ReferenceData.valid_to IS NULL
            "#,
            prev_valid_to_str,
            self.ref_data.isin,
            self.ref_data.trading_venue_attrs.trading_venue
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .rows_affected())
    }
}

/// Structs implementing this trait can be serialised to a database.
pub trait ToDb where Self: Sized {

    /// Serialise to a database.
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError>;
}

pub trait ToDbOption where Self: Sized {
    async fn to_db_option(&self, tx: &mut SqliteTransaction<'_>) -> Result<Option<i64>, SqlError>;
}

impl<T: ToDb> ToDbOption for Option<T> {
    async fn to_db_option(&self, tx: &mut SqliteTransaction<'_>) -> Result<Option<i64>, SqlError> {
        if let Some(t) = self {
            Ok(Some(t.to_db(tx).await?))
        } else {
            Ok(None)
        }
    }
}

impl ToDb for Term {

    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let unit_str = self.unit.to_string();
        let query = sqlx::query!(
            "INSERT INTO Term (number, unit) VALUES (?, ?)",
            self.number,
            unit_str
        );
        Ok(query
            // https://stackoverflow.com/questions/64654769/how-to-build-and-commit-multi-query-transaction-in-sqlx
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for StrikePrice {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let price_type_str = self.price_type.to_string();
        let query = sqlx::query!(
            "INSERT INTO StrikePrice (price_type, price, pending, currency) VALUES (?, ?, ?, ?)",
            price_type_str,
            self.price,
            self.pending,
            self.currency
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for FloatingRate {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let name = self.name.as_ref().map(|n| n.to_string());
        let term_id = if let Some(term) = &self.term {
            Some(term.to_db(tx).await?)
        } else {
            None
        };
        let query = sqlx::query!(
            "INSERT INTO FloatingRate (name, term_id) VALUES (?, ?)",
            name,
            term_id
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for Index {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let name = self.name.to_db(tx).await?;
        let query = sqlx::query!(
            "INSERT INTO FirdsIndex (isin, name_id) VALUES (?, ?)",
            self.isin,
            name
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for TradingVenueAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let appr_date_str = self.approval_date.map(|d| d.to_string());
        let req_date_str = self.request_date.map(|d| d.to_string());
        let aoft_date_str = self.admission_or_first_trade_date.map(|d| d.to_string());
        let term_date_str = self.termination_date.map(|d| d.to_string());
        let query = sqlx::query!(
            r#"
                INSERT INTO TradingVenueAttributes (
                    trading_venue, 
                    requested_admission, 
                    approval_date, 
                    request_date, 
                    admission_or_first_trade_date, 
                    termination_date
                ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
            self.trading_venue,
            self.requested_admission,
            appr_date_str,
            req_date_str,
            aoft_date_str,
            term_date_str
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for InterestRate {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let (fixed, floating, spread) = match self {
            InterestRate::Fixed(rate) => (Some(rate), None, None),
            InterestRate::Floating(rate, spread) =>
                (None, Some(rate.to_db(tx).await?), *spread),
        };
        let query = sqlx::query!(
            "INSERT INTO InterestRate (fixed, floating_rate_id, spread) VALUES (?, ?, ?)",
            fixed,
            floating,
            spread
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for PublicationPeriod {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let from_date_str = &self.from_date.to_string();
        let to_date_str = &self.to_date.map(|d| d.to_string());
        let query = sqlx::query!(
            "INSERT INTO PublicationPeriod (from_date, to_date) VALUES (?, ?)",
            from_date_str,
            to_date_str
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for TechnicalAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let publication_period_id = if let Some(period) = &self.publication_period {
            Some(period.to_db(tx).await?)
        } else {
            None
        };
        let query = sqlx::query!(
            r#"
                INSERT INTO TechnicalAttributes (
                    relevant_competent_authority, 
                    publication_period_id, 
                    relevant_trading_venue
                ) VALUES (?, ?, ?)
            "#,
            self.relevant_competent_authority,
            publication_period_id,
            self.relevant_trading_venue
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for DebtAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let mat_date_str = self.maturity_date.map(|d| d.to_string());
        let ir = self.interest_rate.to_db(tx).await?;
        let seniority_str = self.seniority.map(|d| d.to_string());
        let query = sqlx::query!(
            r#"
                INSERT INTO DebtAttributes (
                    total_issued_amount, 
                    maturity_date, 
                    nominal_currency, 
                    nominal_value_per_unit, 
                    interest_rate_id, 
                    seniority
                ) VALUES (?, ?, ?, ?, ?, ?)            
            "#,
            self.total_issued_amount,
            mat_date_str,
            self.nominal_currency,
            self.nominal_value_per_unit,
            ir,
            seniority_str
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for CommodityDerivativeAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let (product, subproduct, further_subproduct) = self.product.to_codes();
        let trans_type_str = self.transaction_type.map(|t| t.to_string());
        let final_price_type_str = self.final_price_type.map(|t| t.to_string());
        let query = sqlx::query!(
            r#"
                INSERT INTO CommodityDerivativeAttributes (
                   product, 
                   subproduct, 
                   further_subproduct, 
                   transaction_type, 
                   final_price_type
                ) VALUES (?, ?, ?, ?, ?)
            "#,
            product,
            subproduct,
            further_subproduct,
            trans_type_str,
            final_price_type_str
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for InterestRateDerivativeAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let ref_rate = self.reference_rate.to_db(tx).await?;
        let interest_rate_1_id = if let Some(ir) = &self.interest_rate_1 {
            Some(ir.to_db(tx).await?)
        } else {
            None
        };
        let interest_rate_2_id = if let Some(ir) = &self.interest_rate_2 {
            Some(ir.to_db(tx).await?)
        } else {
            None
        };
        let query = sqlx::query!(
            r#"
                INSERT INTO InterestRateDerivativeAttributes (
                  reference_rate_id, 
                  interest_rate_1_id, 
                  notional_currency_2, 
                  interest_rate_2_id
                ) VALUES (?, ?, ?, ?)
            "#,
            ref_rate,
            interest_rate_1_id,
            self.notional_currency_2,
            interest_rate_2_id
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for FxDerivativeAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let fx_type = self.fx_type.map(|t| t.to_string());
        let query = sqlx::query!(
            "INSERT INTO FxDerivativeAttributes (notional_currency_2, fx_type) VALUES (?, ?)",
            self.notional_currency_2,
            fx_type
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for UnderlyingSingle {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let (isin, index_id, lei) = match &self {
            UnderlyingSingle::Isin(isin) => (Some(isin), None, None),
            UnderlyingSingle::Index(index) => (None, Some(index.to_db(tx).await?), None),
            UnderlyingSingle::Lei(lei) => (None, None, Some(lei)),
        };
        let query = sqlx::query!(
            "INSERT INTO UnderlyingSingle (isin, index_id, lei) VALUES (?, ?, ?)",
            isin,
            index_id,
            lei
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for UnderlyingBasket {

    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let basket_id = sqlx::query!("INSERT INTO UnderlyingBasket DEFAULT VALUES")
            .execute(&mut **tx)
            .await?
            .last_insert_rowid();
        for isin in &self.isin {
            sqlx::query!(
                "INSERT INTO UnderlyingBasketIsin VALUES (?, ?)",
                basket_id,
                isin
            ).execute(&mut **tx).await?;
        }
        for lei in &self.issuer_lei {
            sqlx::query!(
                "INSERT INTO UnderlyingBasketIssuerLei VALUES (?, ?)",
                basket_id,
                lei
            ).execute(&mut **tx).await?;
        }
        Ok(basket_id)
    }
}

impl ToDb for DerivativeUnderlying {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let (single_id, basket_id) = match &self {
            DerivativeUnderlying::Single(s) => (Some(s.to_db(tx).await?), None),
            DerivativeUnderlying::Basket(b) => (None, Some(b.to_db(tx).await?)),
        };
        let query = sqlx::query!(
            "INSERT INTO DerivativeUnderlying (single_id, basket_id) VALUES (?, ?)",
            single_id,
            basket_id
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for AssetClassSpecificAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let comm_attrs = self.commodity_attributes.to_db_option(tx).await?;
        let ir_attrs = self.ir_attributes.to_db_option(tx).await?;
        let fx_attrs = self.fx_attributes.to_db_option(tx).await?;
        let query = sqlx::query!(
            r#"
                INSERT INTO AssetClassSpecificAttributes (
                  commodity_attributes_id,
                  ir_attributes_id,
                  fx_attributes_id
                ) VALUES (?, ?, ?)
            "#,
            comm_attrs,
            ir_attrs,
            fx_attrs
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for DerivativeAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let expiry_date_str = self.expiry_date.map(|d| d.to_string());
        let underlying = self.underlying.to_db_option(tx).await?;
        let option_type_str = self.option_type.map(|t| t.to_string());
        let strike_price = self.strike_price.to_db_option(tx).await?;
        let opt_ex_type_str = self.option_exercise_style.map(|s| s.to_string());
        let delivery_type_str = self.delivery_type.map(|t| t.to_string());
        let acsa = self.asset_class_specific_attributes.to_db_option(tx).await?;
        let query = sqlx::query!(
            r#"
                INSERT INTO DerivativeAttributes (
                  expiry_date,
                  price_multiplier,
                  underlying_id, 
                  option_type, 
                  strike_price_id, 
                  option_exercise_style, 
                  delivery_type, 
                  asset_class_specific_attributes_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            expiry_date_str,
            self.price_multiplier,
            underlying,
            option_type_str,
            strike_price,
            opt_ex_type_str,
            delivery_type_str,
            acsa
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for RefDataDbEntry {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let tv_attrs = self.ref_data.trading_venue_attrs.to_db(tx).await?;
        let tech_attrs = self.ref_data.technical_attributes.to_db_option(tx).await?;
        let debt_attrs = self.ref_data.debt_attributes.to_db_option(tx).await?;
        let deriv_attrs = self.ref_data.derivative_attributes.to_db_option(tx).await?;
        let valid_from_str = self.valid_from.to_string();
        let valid_to_str = self.valid_to.map(|d| d.to_string());
        let query = sqlx::query!(
            r#"
                INSERT INTO ReferenceData (
                   isin,
                   full_name,
                   cfi,
                   is_commodities_derivative,
                   issuer_lei,
                   fisn,
                   trading_venue_attrs_id,
                   notional_currency,
                   technical_attributes_id,
                   debt_attributes_id,
                   derivative_attributes_id,
                   latest_record,
                   valid_from,
                   valid_to
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.ref_data.isin,
            self.ref_data.full_name,
            self.ref_data.cfi,
            self.ref_data.is_commodities_derivative,
            self.ref_data.issuer_lei,
            self.ref_data.fisn,
            tv_attrs,
            self.ref_data.notional_currency,
            tech_attrs,
            debt_attrs,
            deriv_attrs,
            self.latest_record,
            valid_from_str,
            valid_to_str,
        );
        Ok(query
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

#[cfg(all(test, feature = "xml", feature = "sql"))]
mod tests {
    use crate::sql::to_db::{RefDataDbEntry, ToDb};
    use crate::xml::IterRefData;
    use crate::xml::{FromXml, XmlIterator};
    use crate::{CancelledRecord, ModifiedRecord};
    use chrono::NaiveDate;
    use sqlx::Connection;
    use std::env::current_dir;
    use std::fs::read_dir;
    use std::path::PathBuf;

    const FULINS_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 2, 1).expect("Bad test date");
    const DLTINS_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 2, 2).expect("Bad test date");

    fn test_data_dir() -> PathBuf {
        current_dir().unwrap().join("test_data")
    }

    fn firds_data_dir() -> PathBuf {
        test_data_dir().join("firds_data")
    }
    
    fn esma_data_dir() -> PathBuf {
       firds_data_dir().join("esma") 
    }
    
    fn test_output_dir() -> PathBuf {
        current_dir().unwrap().join("test_output")
    }
    
    fn get_file_paths(file_type: &str, date: NaiveDate) -> Vec<PathBuf> {
        assert!(file_type == "FULINS" || file_type == "DLTINS" || file_type == "FULCAN");
        let date_str = date.format("%Y%m%d").to_string();
        read_dir(esma_data_dir())
            .expect("Couldn't get list of FIRDS data files")
            .filter_map(|e| {
                let entry = e.as_ref().expect("Could not access file metadata");
                if entry.path().is_file() && entry.file_name().to_str().is_some_and(|s|
                    s.contains(file_type) && s.contains(&date_str)
                ) {
                    return Some(entry.path())
                }
                None 
            })
            .collect()
    }
    
    #[tokio::test]
    async fn test_fulins_to_db() {
        let fulins_files = get_file_paths("FULINS", FULINS_DATE);
        let db_fpath = test_output_dir().join("test_esma.db");
        assert!(db_fpath.is_file());
        let mut conn = sqlx::SqliteConnection::connect(db_fpath.to_str().unwrap())
            .await.expect("Could not connect to database");
        for f in fulins_files {
            let mut tx = conn.begin().await.expect("Could not begin transaction");
            for r in IterRefData::new(&f).expect("Failed to create iterator") {
                let ref_data = r.expect("Could not read reference data");
                let ref_data_entry = RefDataDbEntry::new_latest(ref_data, FULINS_DATE);
                ref_data_entry.to_db(&mut tx).await.expect("Could not serialise to DB");
            }
            tx.commit().await.expect("Could not commit transaction");
        }
    }

    #[tokio::test]
    async fn test_dltins_to_db() {
        let dltins_files = get_file_paths("DLTINS", DLTINS_DATE);
        assert!(!dltins_files.is_empty());
        let db_fpath = firds_data_dir().join("esma_firds_fulins_only.db");
        assert!(db_fpath.is_file());
        let mut conn = sqlx::SqliteConnection::connect(db_fpath.to_str().unwrap())
            .await.expect("Could not connect to database");
        let mut elem_count = 0;
        let mut modified_count = 0;
        for f in dltins_files {
            assert!(f.is_file());
            let mut tx = conn.begin().await.expect("Could not begin transaction");
            for e in XmlIterator::from_file(["ModfdRcrd", "CancRcrd"], &f).expect("Failed to create iterator") {
                let elem = e.expect("Could not read reference data");
                elem_count += 1;

                match elem.local_name.as_str() {
                    "ModfdRcrd" => {
                        modified_count += 1;
                        let m = ModifiedRecord::from_xml(&elem).expect("Could not parse modified record");
                        let r = RefDataDbEntry::new_latest(m.0, DLTINS_DATE);
                        let aff = r.mark_prev_record(&mut tx).await.expect("Could not update previous record");
                        if aff != 1 {
                            println!("{:?}", r.ref_data.isin)
                        }
                        assert_eq!(aff, 1);
                        r.to_db(&mut tx).await.expect("Could not serialise to DB");
                    },
                    "CancRcrd" => {
                        let c = CancelledRecord::from_xml(&elem).expect("Could not parse modified record");
                        let r = RefDataDbEntry::new_latest(c.0, DLTINS_DATE);
                        r.mark_prev_record(&mut tx).await.expect("Could not update previous record");
                        r.to_db(&mut tx).await.expect("Could not serialise to DB");
                    },
                    "TermntdRcrd" => (),
                    other => panic!("Unknown element {other:?}"),
                }
            }
            println!("{modified_count}");
            tx.commit().await.expect("Could not commit transaction");
        }
        assert_ne!(elem_count, 0);
    }
}