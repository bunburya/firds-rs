#[derive(Debug)]
pub enum SqlError {
    /// Received an error from the [`sqlx`] crate.
    Sqlx(sqlx::Error),
    /// [`ReferenceData`] struct is missing `technical_attributes.publication_period` field.
    MissingPublicationPeriod,
}

impl From<sqlx::Error> for SqlError {
    fn from(e: sqlx::Error) -> Self {
        SqlError::Sqlx(e)
    }
}