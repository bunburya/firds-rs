use crate::sql::error::SqlError;
use crate::*;
use sqlx::{Executor, SqliteTransaction};

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
        Ok(sqlx::query("INSERT INTO Term (number, unit) VALUES ($1, $2)")
            .bind(&self.number)
            .bind(&self.unit.to_string())
            // https://stackoverflow.com/questions/64654769/how-to-build-and-commit-multi-query-transaction-in-sqlx
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for StrikePrice {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO StrikePrice (price_type, price, pending, currency) VALUES ($1, $2, $3, $4)")
            .bind(&self.price_type.to_string())
            .bind(&self.price)
            .bind(&self.pending)
            .bind(&self.currency)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for FloatingRate {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let name = if let Some(name) = &self.name {
            Some(name.to_string())
        } else {
            None
        };
        let term_id = if let Some(term) = &self.term {
            Some(term.to_db(tx).await?)
        } else {
            None
        };
        Ok(sqlx::query("INSERT INTO FloatingRate (name, term_id) VALUES ($1, $2)")
            .bind(name)
            .bind(term_id)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for Index {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO FirdsIndex (isin, name_id) VALUES ($1, $2)")
            .bind(&self.isin)
            .bind(&self.name.to_db(tx).await?)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for TradingVenueAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query(
            "INSERT INTO TradingVenueAttributes(trading_venue, requested_admission, approval_date, request_date, admission_or_first_trade_date, termination_date) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(&self.trading_venue)
            .bind(&self.requested_admission)
            .bind(&self.approval_date.map(|d| d.to_string()))
            .bind(&self.request_date.map(|d| d.to_string()))
            .bind(&self.admission_or_first_trade_date.map(|d| d.to_string()))
            .bind(self.termination_date.map(|d| d.to_string()))
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
        Ok(sqlx::query("INSERT INTO InterestRate (fixed, floating_rate_id, spread) VALUES ($1, $2, $3)")
            .bind(fixed)
            .bind(floating)
            .bind(spread)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for PublicationPeriod {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO PublicationPeriod (from_date, to_date) VALUES ($1, $2)")
            .bind(&self.from_date.to_string())
            .bind(&self.to_date.map(|d| d.to_string()))
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
        Ok(sqlx::query("INSERT INTO TechnicalAttributes (relevant_competent_authority, publication_period_id, relevant_trading_venue) VALUES ($1, $2, $3)")
            .bind(&self.relevant_competent_authority)
            .bind(&publication_period_id)
            .bind(&self.relevant_trading_venue)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for DebtAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO DebtAttributes (total_issued_amount, maturity_date, nominal_currency, nominal_value_per_unit, interest_rate_id, seniority) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(&self.total_issued_amount)
            .bind(&self.maturity_date.map(|d| d.to_string()))
            .bind(&self.nominal_currency)
            .bind(&self.nominal_value_per_unit)
            .bind(&self.interest_rate.to_db(tx).await?)
            .bind(&self.seniority.map(|d| d.to_string()))
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for CommodityDerivativeAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let (product, subproduct, further_subproduct) = self.product.to_codes();
        Ok(sqlx::query("INSERT INTO CommodityDerivativeAttributes (product, subproduct, further_subproduct, transaction_type, final_price_type) VALUES ($1, $2, $3, $4, $5)")
            .bind(&product)
            .bind(&subproduct)
            .bind(&further_subproduct)
            .bind(&self.transaction_type.map(|t| t.to_string()))
            .bind(&self.final_price_type.map(|t| t.to_string()))
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for InterestRateDerivativeAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
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
        Ok(sqlx::query("INSERT INTO InterestRateDerivativeAttributes (reference_rate_id, interest_rate_1_id, notional_currency_2, interest_rate_2_id) VALUES ($1, $2, $3, $4)")
            .bind(&self.reference_rate.to_db(tx).await?)
            .bind(&interest_rate_1_id)
            .bind(&self.notional_currency_2)
            .bind(&interest_rate_2_id)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for FxDerivativeAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO FxDerivativeAttributes (notional_currency_2, fx_type) VALUES ($1, $2)")
            .bind(&self.notional_currency_2)
            .bind(&self.fx_type.map(|t| t.to_string()))
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
        Ok(sqlx::query("INSERT INTO UnderlyingSingle (isin, index_id, lei) VALUES ($1, $2, $3)")
            .bind(isin)
            .bind(index_id)
            .bind(lei)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for UnderlyingBasket {

    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        let basket_id = sqlx::query("INSERT INTO UnderlyingBasket DEFAULT VALUES")
            .execute(&mut **tx)
            .await?
            .last_insert_rowid();
        for isin in &self.isin {
            sqlx::query("INSERT INTO UnderlyingBasketIsin VALUES ($1, $2)")
                .bind(basket_id)
                .bind(isin)
                .execute(&mut **tx)
                .await?;
        }
        for lei in &self.issuer_lei {
            sqlx::query("INSERT INTO UnderlyingBasketIssuerLei VALUES ($1, $2)")
                .bind(basket_id)
                .bind(lei)
                .execute(&mut **tx)
                .await?;
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
        Ok(sqlx::query("INSERT INTO DerivativeUnderlying (single_id, basket_id) VALUES ($1, $2)")
            .bind(single_id)
            .bind(basket_id)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for AssetClassSpecificAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO AssetClassSpecificAttributes (commodity_attributes_id, ir_attributes_id, fx_attributes_id) VALUES ($1, $2, $3)")
            .bind(&self.commodity_attributes.to_db_option(tx).await?)
            .bind(&self.ir_attributes.to_db_option(tx).await?)
            .bind(&self.fx_attributes.to_db_option(tx).await?)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for DerivativeAttributes {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query(
            "INSERT INTO DerivativeAttributes (expiry_date, price_multiplier, underlying_id, option_type, strike_price_id, option_exercise_style, delivery_type, asset_class_specific_attributes_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(self.expiry_date.map(|d| d.to_string()))
            .bind(self.price_multiplier)
            .bind(self.underlying.to_db_option(tx).await?)
            .bind(self.option_type.map(|t| t.to_string()))
            .bind(self.strike_price.to_db_option(tx).await?)
            .bind(self.option_exercise_style.map(|s| s.to_string()))
            .bind(self.delivery_type.map(|t| t.to_string()))
            .bind(self.asset_class_specific_attributes.to_db_option(tx).await?)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for ReferenceData {
    async fn to_db(&self, tx: &mut SqliteTransaction<'_>) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO ReferenceData (isin, full_name, cfi, is_commodities_derivative, issuer_lei, fisn, trading_venue_attrs_id, notional_currency, technical_attributes_id, debt_attributes_id, derivative_attributes_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)")
            .bind(&self.isin)
            .bind(&self.full_name)
            .bind(&self.cfi)
            .bind(&self.is_commodities_derivative)
            .bind(&self.issuer_lei)
            .bind(&self.fisn)
            .bind(&self.trading_venue_attrs.to_db(tx).await?)
            .bind(&self.notional_currency)
            .bind(&self.technical_attributes.to_db_option(tx).await?)
            .bind(&self.debt_attributes.to_db_option(tx).await?)
            .bind(&self.derivative_attributes.to_db_option(tx).await?)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid())
    }
}

#[cfg(all(test, feature = "xml", feature = "sql"))]
mod tests {
    use crate::sql::to_db::ToDb;
    use crate::xml::IterRefData;
    use chrono::NaiveDate;
    use sqlx::Connection;
    use std::env::current_dir;
    use std::fs::read_dir;
    use std::path::PathBuf;

    const FULINS_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 2, 1).expect("Bad test date");

    fn firds_data_dir() -> PathBuf {
        current_dir().unwrap().join("test_data").join("firds_data")
    }
    
    fn esma_data_dir() -> PathBuf {
       firds_data_dir().join("esma") 
    }
    
    fn get_test_output_dir() -> PathBuf {
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
                    s.contains(&file_type) && s.contains(&date_str)
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
        let db_fpath = get_test_output_dir().join("test.db");
        assert!(db_fpath.is_file());
        let mut conn = sqlx::SqliteConnection::connect(db_fpath.to_str().unwrap())
            .await.expect("Could not connect to database");
        for f in fulins_files {
            let mut tx = conn.begin().await.expect("Could not begin transaction");
            for r in IterRefData::new(&f).expect("Failed to create iterator") {
                let ref_data = r.expect("Could not read reference data");
                ref_data.to_db(&mut tx).await.expect("Could not serialise to DB.");
            }
            tx.commit().await.expect("Could not commit transaction");
        }
    }
}