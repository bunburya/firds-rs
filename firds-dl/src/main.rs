use crate::download::{search_esma, search_fca, FirdsDoc, FirdsDocType, FirdsSource, StreamProgress};
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use log::warn;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

mod download;
mod error;

impl StreamProgress for ProgressBar {
    fn on_init(&self, content_length: u64) {
        self.set_length(content_length);
    }

    fn on_progress(&self, progress: u64) {
        self.inc(progress)
    }

    fn on_msg(&self, msg: &str) {
        self.set_message(msg.to_owned())
    }
}

fn new_progress_bar(file_name: &str) -> ProgressBar {
    let template = format!(
        "{:<30} [{{elapsed_precise}}] {{bar:40.cyan/blue}} {{decimal_bytes:>7}}/{{decimal_total_bytes:7}} {{msg}}",
        file_name
    );
    ProgressBar::new(0).with_style(
        ProgressStyle::with_template(&template).unwrap_or(ProgressStyle::default_bar())
    )
}

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
    /// Verify the MD5 sum of a file downloaded from ESMA. Does nothing where files are downloaded
    /// from the FCA.
    #[clap(short, long, action)]
    verify: bool,
    /// Overwrite an existing file with the same name.
    #[clap(short, long, action)]
    overwrite: bool,
    /// Keep zip files after extracting to XML (by default, these are deleted).
    #[clap(short, long, action)]
    keep_zip: bool
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let ft_str = if let Some(ft) = args.file_type {
        ft.to_string()
    } else {
        "all".to_string()
    };
    eprintln!(
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
    eprintln!("Found {} files.", docs.len());
    if let Some(to_dir) = args.to_dir {
        if args.verify && (args.source == FirdsSource::Fca) {
            warn!("Verification only possible where FIRDS source is ESMA. Not verifying.")
        }
        let multi_prog = MultiProgress::new();
        let f = docs.iter()
            .map(|doc| {
                doc.download_xml_with_progress(
                    &client,
                    &to_dir,
                    args.overwrite,
                    args.verify,
                    !args.keep_zip,
                    multi_prog.add(new_progress_bar(&doc.file_name))
                )
            });
        join_all(f).await;
    }
    
}