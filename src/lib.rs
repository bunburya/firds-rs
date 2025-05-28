//! `firds-rs` is a Rust crate for interacting with the
//! [Financial Instrument Reference Data System](https://data.europa.eu/data/datasets/financial-instruments-reference-data-system)
//! (FIRDS).
//! 
//! The "core" of the crate, without any features enabled, just contains structs and enums for
//! representing FIRDS data.

mod model;
mod enums;
mod product_enums;
mod error;

#[cfg(feature = "download")]
pub mod download;

#[cfg(feature = "xml")]
pub mod xml;

#[cfg(feature = "sql")]
mod sql;

pub use model::*;
pub use enums::*;
pub use product_enums::*;
pub use error::*;