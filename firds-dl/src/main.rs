use std::cmp::PartialEq;
use crate::download::{search_esma, search_fca, FirdsDocType, FirdsSource};
use chrono::NaiveDate;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use log::warn;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar};

mod download;
mod error;

#[derive(Parser, Debug)]
struct Args {
    /// Start date of period to search.
    from_date: NaiveDate,
    /// End date of period to search.
    to_date: NaiveDate,
    /// Directory to download files to.
    #[clap(short, long)]
    to_dir: Option<PathBuf>,
    /// Type of FIRDS file to search for.
    #[clap(short, long)]
    file_type: Option<FirdsDocType>,
    /// Where to search for the files.
    #[clap(short, long, default_value = "esma")]
    source: FirdsSource,
    #[clap(short, long, action)]
    verify: bool,
    #[clap(short, long, action)]
    overwrite: bool,
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
    println!("Found {} files.", docs.len());
    if let Some(to_dir) = args.to_dir {
        if args.verify && (args.source == FirdsSource::Fca) {
            warn!("Verification only possible where FIRDS source is ESMA. Not verifying.")
        }
        let multi_prog = MultiProgress::new();
        let f = docs.iter().map(|doc| {
            doc.download_xml(&client, &to_dir, args.overwrite, args.verify, true, Some(&multi_prog))
        });
        join_all(f).await;
    }
    
}