use crate::sql::error::SqlError;
use sqlx::Connection;

/// Structs implementing this trait can be serialised to a database.
pub trait ToDb where Self: Sized {
    /// Serialise to a database.
    fn to_db(&self, conn: &impl Connection) -> Result<usize, SqlError>;
}