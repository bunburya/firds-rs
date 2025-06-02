use crate::sql::error::SqlError;
use crate::*;
use sqlx::SqliteConnection;

/// Structs implementing this trait can be serialised to a database.
pub trait ToDb where Self: Sized {

    /// Serialise to a database.
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError>;
}

impl ToDb for Term {

    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO Term VALUES ($1, $2)")
            .bind(&self.number)
            .bind(&self.unit.to_string())
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for StrikePrice {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO StrikePrice VALUES ($1, $2, $3, $4)")
            .bind(&self.price_type.to_string())
            .bind(&self.price)
            .bind(&self.pending)
            .bind(&self.currency)
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for FloatingRate {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        let name = if let Some(name) = &self.name {
            Some(name.to_string())
        } else {
            None
        };
        let term_id = if let Some(term) = &self.term {
            Some(term.to_db(conn).await?)
        } else {
            None
        };
        Ok(sqlx::query("INSERT INTO FloatingRate VALUES ($1, $2)")
            .bind(name)
            .bind(term_id)
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for Index {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO FirdsIndex VALUES ($1, $2)")
            .bind(&self.isin)
            .bind(self.name.to_db(conn).await?)
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for TradingVenueAttributes {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO TradingVenueAttributes VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(&self.trading_venue)
            .bind(&self.requested_admission)
            .bind(&self.approval_date.map(|d| d.to_string()))
            .bind(&self.request_date.map(|d| d.to_string()))
            .bind(&self.admission_or_first_trade_date.map(|d| d.to_string()))
            .bind(self.termination_date.map(|d| d.to_string()))
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for InterestRate {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        let (fixed, floating, spread) = match self {
            InterestRate::Fixed(rate) => (Some(rate), None, None),
            InterestRate::Floating(rate, spread) =>
                (None, Some(rate.to_db(conn).await?), *spread),
        };
        Ok(sqlx::query("INSERT INTO InterestRate VALUES ($1, $2, $3)")
            .bind(fixed)
            .bind(floating)
            .bind(spread)
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for PublicationPeriod {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO PublicationPeriod VALUES ($1, $2)")
            .bind(&self.from_date.to_string())
            .bind(&self.to_date.map(|d| d.to_string()))
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for TechnicalAttributes {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        let publication_period_id = if let Some(period) = &self.publication_period {
            Some(period.to_db(conn).await?)
        } else {
            None
        };
        Ok(sqlx::query("INSERT INTO TechnicalAttributes VALUES ($1, $2, $3)")
            .bind(&self.relevant_competent_authority)
            .bind(&publication_period_id)
            .bind(&self.relevant_trading_venue)
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for DebtAttributes {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO DebtAttributes VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(&self.total_issued_amount)
            .bind(&self.maturity_date.map(|d| d.to_string()))
            .bind(&self.nominal_currency)
            .bind(&self.nominal_value_per_unit)
            .bind(&self.interest_rate.to_db(conn).await?)
            .bind(&self.seniority.map(|d| d.to_string()))
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for CommodityDerivativeAttributes {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        let (product, subproduct, further_subproduct) = self.product.to_codes();
        Ok(sqlx::query("INSERT INTO CommodityDerivativeAttributes VALUES ($1, $2, $3, $4, $5)")
            .bind(&product)
            .bind(&subproduct)
            .bind(&further_subproduct)
            .bind(&self.transaction_type.map(|t| t.to_string()))
            .bind(&self.final_price_type.map(|t| t.to_string()))
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for InterestRateDerivativeAttributes {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        let interest_rate_1_id = if let Some(ir) = &self.interest_rate_1 {
            Some(ir.to_db(conn).await?)
        } else {
            None
        };
        let interest_rate_2_id = if let Some(ir) = &self.interest_rate_2 {
            Some(ir.to_db(conn).await?)
        } else {
            None
        };
        Ok(sqlx::query("INSERT INTO InterestRateDerivativeAttributes VALUES ($1, $2, $3, $4)")
            .bind(&self.reference_rate.to_db(conn).await?)
            .bind(&interest_rate_1_id)
            .bind(&self.notional_currency_2)
            .bind(&interest_rate_2_id)
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for FxDerivativeAttributes {
    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        Ok(sqlx::query("INSERT INTO FxDerivativeAttributes VALUES ($1, $2)")
            .bind(&self.notional_currency_2)
            .bind(&self.fx_type.map(|t| t.to_string()))
            .execute(conn)
            .await?
            .last_insert_rowid())
    }
}

impl ToDb for UnderlyingBasket {

    async fn to_db(&self, conn: &mut SqliteConnection) -> Result<i64, SqlError> {
        let basket_id = sqlx::query("INSERT INTO UnderlyingBasket DEFAULT VALUES")
            .execute(&mut *conn)
            .await?
            .last_insert_rowid();
        for isin in &self.isin {
            sqlx::query("INSERT INTO UnderlyingBasketIsin VALUES ($1, $2)")
                .bind(basket_id)
                .bind(isin)
                .execute(&mut *conn)
                .await?;
        }
        for lei in &self.issuer_lei {
            sqlx::query("INSERT INTO UnderlyingBasketIssuerLei VALUES ($1, $2)")
                .bind(basket_id)
                .bind(lei)
                .execute(&mut *conn)
                .await?;
        }
        Ok(basket_id)
    }
}
