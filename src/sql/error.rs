#[derive(Debug)]
pub enum SqlError {
    /// Received an error from the [`sqlx`] crate.
    Sqlx(sqlx::Error),
}

impl From<sqlx::Error> for SqlError {
    fn from(e: sqlx::Error) -> Self {
        SqlError::Sqlx(e)
    }
}