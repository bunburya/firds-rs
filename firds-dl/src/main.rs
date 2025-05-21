use crate::download::search_esma;
use chrono::{TimeZone, Utc};
use reqwest::Client;

mod download;
mod error;
mod esma;

#[tokio::main]
async fn main() {
    let client = Client::new();
    search_esma(
        &client,
        Utc.with_ymd_and_hms(2024, 8, 25, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2024, 9, 25, 23, 59, 59).unwrap(),
        None
    ).await.expect("Error");
}