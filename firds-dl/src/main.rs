use crate::download::{search_esma, search_fca, FirdsDocType, FirdsSource};
use chrono::NaiveDate;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

mod download;
mod error;

#[derive(Parser, Debug)]
struct Args {
    /// Start date of period to search.
    from_date: NaiveDate,
    /// End date of period to search.
    to_date: NaiveDate,
    /// Directory to download files to.
    to_dir: PathBuf,
    /// Type of FIRDS file to search for.
    file_type: Option<FirdsDocType>,
    /// Where to search for the files.
    #[arg(default_value = "esma")]
    source: FirdsSource
    
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let ft_str = if let Some(ft) = args.file_type {
        ft.to_string()
    } else {
        "all".to_string()
    };
    println!(
        "Searching {} FIRDS for {} files from {} to {}.",
        args.source,
        ft_str,
        args.from_date,
        args.to_date,
    );
    let client = reqwest::Client::new();
    let docs = match args.source {
        FirdsSource::Esma => {
            let from_dt = args.from_date.and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let to_dt = args.to_date.and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            search_esma(&client, from_dt, to_dt, args.file_type).await.unwrap()
        },
        FirdsSource::Fca => 
            search_fca(&client, args.from_date, args.to_date, args.file_type).await.unwrap()
    };
    println!("Found {} docs.", docs.len());
    
}