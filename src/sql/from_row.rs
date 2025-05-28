use sqlx::{Connection, Row};
use crate::*;

/// Structs implementing this trait can be constructed from an appropriately structured
/// [`sqlx::Row`].
pub trait FromRow where Self: Sized {
    /// Create an instance of this struct from the given row (if possible). `conn` is passed to
    /// allow querying other tables in the database in order to construct other structs implementing
    /// `FromRow` which are fields of this struct. 
    fn from_row(row: &impl Row, conn: &impl Connection) -> Result<Self, ParseError>;
}

impl FromRow for ReferenceData {
    fn from_row(row: &impl Row, conn: &impl Connection) -> Result<Self, ParseError> {
        todo!()
    }
}