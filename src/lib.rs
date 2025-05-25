//! Types for representing data in the [Financial Instrument Reference Data System](https://data.europa.eu/data/datasets/financial-instruments-reference-data-system)
//! (FIRDS).
//! 
//! This crate just contains the types themselves. Other crates provide additional functionality for
//! accessing the data and constructing the crates.

mod model;
mod categories;
mod product;
mod error;

#[cfg(feature = "download")]
pub mod download;

#[cfg(feature = "xml")]
pub mod xml;

pub use model::*;
pub use categories::*;
pub use product::*;
pub use error::*;