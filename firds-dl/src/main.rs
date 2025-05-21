use crate::download::{search_esma, FileType, FirdsSource};
use chrono::NaiveDate;
use clap::Parser;
use std::path::PathBuf;

mod download;
mod error;
mod esma;

#[derive(Parser, Debug)]
struct Args {
    /// Start date of period to search.
    from_date: NaiveDate,
    /// End date of period to search.
    to_date: NaiveDate,
    /// Directory to download files to.
    to_dir: PathBuf,
    /// Type of FIRDS file to search for.
    file_type: Option<FileType>,
    /// Where to search for the files.
    #[arg(default_value = "esma")]
    source: FirdsSource
    
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{args:?}");
    let from_dt = args.from_date.and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let to_dt = args.to_date.and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();
    let client = reqwest::Client::new();
    search_esma(&client, from_dt, to_dt, args.file_type).await.unwrap();
}