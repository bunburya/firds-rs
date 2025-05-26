mod error;

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use futures_util::StreamExt;
use md5::{Digest, Md5};
use reqwest::Client;
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::fs::{create_dir_all, remove_file, rename, File};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::download::error::DownloadError;

#[cfg(feature = "download-cli")]
use clap::ValueEnum;


const ESMA_BASE_URL: &str = "https://registers.esma.europa.eu/solr/esma_registers_firds_files/select";
const FCA_BASE_URL: &str = "https://api.data.fca.org.uk/fca_data_firds_files";

/// Get a string from a JSON object, or return an error.
fn str_from_map<'a>(map: &'a Map<String, Value>, k: &str) -> Result<&'a str, DownloadError> {
    map.get(k)
        .ok_or(DownloadError::JsonMapKeyNotFound(k.to_owned()))?
        .as_str()
        .ok_or(DownloadError::BadJson)
}

/// Get a map from a JSON object, or return an error.
fn map_from_map<'a>(map: &'a Map<String, Value>, k: &str) -> Result<&'a Map<String, Value>, DownloadError> {
    map.get(k)
        .ok_or(DownloadError::JsonMapKeyNotFound(k.to_owned()))?
        .as_object()
        .ok_or(DownloadError::BadJson)
}

/// Structs implementing this trait are used to display the progress of a streaming download, such
/// as display a progress bar to the user.
pub trait StreamProgress {

    /// This method is called when the stream is initialised. It is passed the content length (a
    /// value of 0 may indicate the content length is not known).
    fn on_init(&self, content_len: u64);

    /// This method is called each time a chunk of data is streamed. It is passed the size of the
    /// streamed data.
    fn on_progress(&self, chunk_len: u64);

    /// This method is called to pass a new message to the user relating to the progress.
    fn on_msg(&self, msg: &str);
}

/// An implementation of [`StreamProgress`] that does nothing. Slightly hacky way to allow ergonomic
/// download functions which do not need to take a concrete implementation of `StreamProgress` where
/// the caller does not want progress tracking.
struct _NoopProgress;

impl StreamProgress for _NoopProgress {
    fn on_init(&self, _: u64) {}
    fn on_progress(&self, _: u64) {}
    fn on_msg(&self, _: &str) {}
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "download-cli", derive(ValueEnum))]
pub enum FirdsDocType {
    Fulins,
    Dltins,
    Fulcan
}

impl FromStr for FirdsDocType {
    type Err = DownloadError;

    /// Parse an *upper-case* string (one of "FULINS", "DLTINS" or "FULCAN") into a variant of
    /// [`FirdsDocType`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FULINS" => Ok(Self::Fulins),
            "DLTINS" => Ok(Self::Dltins),
            "FULCAN" => Ok(Self::Fulcan),
            _ => Err(DownloadError::EnumParseError(s.to_owned()))
        }
    }
}

impl Display for FirdsDocType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fulins => write!(f, "FULINS"),
            Self::Dltins => write!(f, "DLTINS"),
            Self::Fulcan => write!(f, "FULCAN"),
        }
    }
}

/// Where we are downloading the data from (ESMA or FCA).
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "download-cli", derive(ValueEnum))]
pub enum FirdsSource {
    /// European Securities and Markets Authority (EU).
    Esma,
    /// Financial Conduct Authority (UK).
    Fca
}

impl Display for FirdsSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Esma => write!(f, "ESMA"),
            Self::Fca => write!(f, "FCA")
        }
    }
}

/// A single document reference, returned by searching a FIRDS database (ESMA or FCA).
pub struct FirdsDoc {
    /// The source of the document.
    pub source: FirdsSource,
    /// A URL to download the file from.
    pub download_link: String,
    /// An ID for the file.
    pub file_id: String,
    /// The name of the (zip) file.
    pub file_name: String,
    /// The type of the file.
    pub file_type: FirdsDocType,
    /// The timestamp of the document.
    pub timestamp: DateTime<FixedOffset>,
    /// MD5 checksum for the file, if present (should be present in ESMA data but not FCA data).
    pub checksum: Option<String>
}

impl FirdsDoc {
    pub fn from_esma_json(json: &Value) -> Result<Self, DownloadError> {
        if let Value::Object(map) = json {
            Ok(Self {
                source: FirdsSource::Esma,
                download_link: str_from_map(map, "download_link")?.to_owned(),
                file_id: str_from_map(map, "id")?.to_owned(),
                file_name: str_from_map(map, "file_name")?.to_owned(),
                file_type: <FirdsDocType as FromStr>::from_str(str_from_map(map, "file_type")?)?,
                timestamp: DateTime::parse_from_rfc3339(str_from_map(map, "timestamp")?)?,
                checksum: Some(str_from_map(map, "checksum")?.to_owned())
            })
        } else {
            Err(DownloadError::BadJson)
        }
    }

    pub fn from_fca_json(json: &Value) -> Result<Self, DownloadError> {
        if let Value::Object(map) = json {
            let source_json = map_from_map(map, "_source")?;
            Ok(Self {
                source: FirdsSource::Fca,
                download_link: str_from_map(source_json, "download_link")?.to_owned(),
                file_id: str_from_map(map, "_id")?.to_owned(),
                file_name: str_from_map(source_json, "file_name")?.to_owned(),
                file_type: <FirdsDocType as FromStr>::from_str(str_from_map(source_json, "file_type")?)?,
                timestamp: DateTime::parse_from_rfc3339(str_from_map(source_json, "last_refreshed")?)?,
                checksum: None
            })
        } else {
            Err(DownloadError::BadJson)
        }
    }

    /// Verify the md5 checksum of the file at the given path. Assumes that a checksum is present in
    /// the struct, returning an error if not.
    pub fn verify_file(&self, fpath: &Path) -> Result<(), DownloadError> {
        if let Some(cs) = &self.checksum {
            let mut file = File::open(fpath)?;
            let mut hasher = Md5::new();
            io::copy(&mut file, &mut hasher)?;
            let hash = hasher.finalize();
            let hex = base16ct::lower::encode_string(&hash);
            if &hex != cs {
                return Err(DownloadError::Md5CheckFailed(hex))
            }
        } else {
            return Err(DownloadError::NoMd5Sum)
        }
        Ok(())
    }

    /// Download the zip file from the source, tracking progress of the download.
    ///
    /// This method takes the same arguments as [`FirdsDoc::download_zip`] as well as an additional
    /// `progress` argument, a struct that implements the [`StreamProgress`] trait.
    pub async fn download_zip_with_progress(
        &self,
        client: &Client,
        to_dir: &Path,
        overwrite: bool,
        verify: bool,
        progress: &impl StreamProgress,
    ) -> Result<PathBuf, DownloadError> {
        progress.on_msg("Downloading...");
        if !to_dir.exists() {
            create_dir_all(to_dir)?
        }
        let fpath = to_dir.join(&self.file_name);
        let mut fpath_part = OsString::from(&fpath);
        fpath_part.push(".part");
        let fpath_part: PathBuf = fpath_part.into();
        if !overwrite && fpath.exists() {
            return Err(DownloadError::FileExists(fpath))
        }
        if fpath_part.exists() {
            // Remove .part file if present because it probably represents a failed previous attempt
            // at downloading
            remove_file(&fpath_part)?
        }
        let resp = client.get(&self.download_link).send().await?;
        let mut file = File::create(&fpath_part)?;
        progress.on_init(resp.content_length().unwrap_or(0));

        let mut stream = resp.bytes_stream();
        while let Some(res) = stream.next().await {
            let bytes = res?;
            file.write_all(&bytes)?;
            progress.on_progress(bytes.len() as u64)

        }
        if verify {
            self.verify_file(&fpath_part)?;
        }
        rename(&fpath_part, &fpath)?;
        Ok(fpath)
    }

    /// Download the zip file from the source.
    ///
    /// The file is downloaded in chunks and saved to a `.part` file, which is then renamed to the
    /// expected file name once download is complete.
    ///
    /// # Arguments:
    ///
    /// * `client`: A [`Client`] that will be used to make the request.
    /// * `to_dir`: The directory in which to save the downloaded file. It will be created if it
    ///   does not exist.
    /// * `overwrite`: Whether to overwrite an existing file if it exists at the destination. If
    ///   `false`, an error will be returned if a file already exists.
    /// * `verify`: Whether to verify the file after it is downloaded by comparing its md5 sum
    ///   against the checksum stored in the struct. If `true` and no checksum is present in the
    ///   struct, an error will be returned.
    ///
    /// To track the progress of the download (eg, using a progress bar), see
    /// [`FirdsDoc::download_zip_with_progress`].
    pub async fn download_zip(
        &self,
        client: &Client,
        to_dir: &Path,
        overwrite: bool,
        verify: bool,
    ) -> Result<PathBuf, DownloadError> {
        self.download_zip_with_progress(client, to_dir, overwrite, verify, &_NoopProgress).await
    }

    /// Download the XML file from the source, tracking progress of the download.
    ///
    /// This method takes the same arguments as [`FirdsDoc::download_xml`] as well as an additional
    /// `progress` argument, a struct that implements the [`StreamProgress`] trait.
    pub async fn download_xml_with_progress(
        &self,
        client: &Client,
        to_dir: &Path,
        overwrite: bool,
        verify: bool,
        delete_zip: bool,
        progress: impl StreamProgress
    ) -> Result<PathBuf, DownloadError> {
        let zip_fpath = self.download_zip_with_progress(
            client,
            to_dir,
            overwrite,
            verify,
            &progress
        ).await?;
        let zipped_file = File::open(&zip_fpath)?;
        let mut archive = zip::ZipArchive::new(zipped_file)?;
        let unzipped_fname = self.file_name.replace(".zip", ".xml");
        let mut zip_file = archive.by_name(&unzipped_fname)?;
        let unzipped_fpath = to_dir.join(&unzipped_fname);
        if !overwrite && unzipped_fpath.exists() {
            return Err(DownloadError::FileExists(unzipped_fpath))
        }
        let mut unzipped_file = File::create(&unzipped_fpath)?;
        progress.on_msg("Extracting...");

        io::copy(&mut zip_file, &mut unzipped_file)?;
        if delete_zip {
            remove_file(&zip_fpath)?;
        }
        Ok(unzipped_fpath)
    }

    /// Download the XML file from the source, by first downloading the zip file and then
    /// extracting. the XML file.
    ///
    /// # Arguments:
    ///
    /// * `client`: A [`Client`] that will be used to make the request.
    /// * `to_dir`: The directory in which to save the downloaded file. It will be created if it
    ///   does not exist.
    /// * `overwrite`: Whether to overwrite an existing file if it exists at the destination. If
    ///   `false`, an error will be returned if a file already exists.
    /// * `verify`: Whether to verify the file after it is downloaded by comparing its md5 sum
    ///   against the checksum stored in the struct. If `true` and no checksum is present in the
    ///   struct, an error will be returned.
    /// * `delete_zip`: Whether to delete the downloaded zip file once the XML file has been
    ///   extracted.
    ///
    /// To track the progress of the download (eg, using a progress bar), see
    /// [`FirdsDoc::download_xml_with_progress`].
    pub async fn download_xml(
        &self,
        client: &Client,
        to_dir: &Path,
        overwrite: bool,
        verify: bool,
        delete_zip: bool,
    ) -> Result<PathBuf, DownloadError> {
        self.download_xml_with_progress(
            client,
            to_dir,
            overwrite,
            verify,
            delete_zip,
            _NoopProgress
        ).await
    }
}

/// Search the ESMA FIRDS database for files from the given time period and, if applicable, of the
/// given type.
pub async fn search_esma(
    client: &Client,
    from_datetime: DateTime<Utc>,
    to_datetime: DateTime<Utc>,
    file_type: Option<FirdsDocType>
) -> Result<Vec<FirdsDoc>, DownloadError> {
    let from_dt_str = from_datetime.format("%Y-%m-%dT%H:%M:%SZ");
    let to_dt_str = to_datetime.format("%Y-%m-%dT%H:%M:%SZ");
    let pub_date_fq = format!("publication_date:[{from_dt_str} TO {to_dt_str}]");
    let q = if let Some(ft) = file_type {
        ft.to_string()
    } else {
        "*".to_owned()
    };
    let mut start = 0;
    let rows = 100;
    let rows_str = rows.to_string();
    let mut num_found= -1;
    let mut docs: Vec<FirdsDoc> = vec![];
    while (num_found < 0) || (num_found > start) {
        let url = reqwest::Url::parse_with_params(
            ESMA_BASE_URL,
            &[
                ("q", q.as_str()),
                ("fq", pub_date_fq.as_str()),
                ("wt", "json"),
                ("start", &start.to_string()),
                ("rows", &rows_str),
            ]
        )?;
        let text = client.get(url).send().await?.text().await?;
        let json: Value = serde_json::from_str(&text)?;
        let resp_body = json.get("response")
            .and_then(Value::as_object)
            .ok_or(DownloadError::BadJson)?;
        if num_found < 0 {
            num_found= resp_body.get("numFound")
                .and_then(Value::as_i64)
                .ok_or(DownloadError::BadJson)?;
        }
        let resp = json.get("response")
            .and_then(Value::as_object)
            .ok_or(DownloadError::BadJson)?;
        docs.extend(
            resp.get("docs")
                .and_then(Value::as_array)
                .ok_or(DownloadError::BadJson)?
                .iter()
                .map(FirdsDoc::from_esma_json)
                .collect::<Result<Vec<FirdsDoc>, DownloadError>>()?
        );
        start += rows;
    }
    Ok(docs)
}

pub async fn search_fca(
    client: &Client,
    from_date: NaiveDate,
    to_date: NaiveDate,
    file_type: Option<FirdsDocType>
) -> Result<Vec<FirdsDoc>, DownloadError> {
    let from_date_str = from_date.format("%Y-%m-%d");
    let to_date_str = to_date.format("%Y-%m-%d");
    let pub_date_q = format!("publication_date:[{from_date_str} TO {to_date_str}]");
    let q = if let Some(ft) = file_type {
        format!("((file_type:{}) AND ({pub_date_q}))", ft.to_string())
    } else {
        format!("({pub_date_q})")
    };
    let mut start = 0;
    let rows = 100;
    let rows_str = rows.to_string();
    let mut num_found= -1;
    let mut docs = vec![];
    while (num_found < 0) || (num_found > start) {
        let url = reqwest::Url::parse_with_params(
            FCA_BASE_URL,
            &[
                ("q", q.as_str()),
                ("from", &start.to_string()),
                ("size", &rows_str),
            ]
        )?;
        let text = client.get(url).send().await?.text().await?;
        let json: Value = serde_json::from_str(&text)?;
        let resp_body = json.get("hits")
            .and_then(Value::as_object)
            .ok_or(DownloadError::BadJson)?;
        if num_found < 0 {
            num_found= resp_body.get("total")
                .and_then(Value::as_i64)
                .ok_or(DownloadError::BadJson)?;
        }
        docs.extend(
            resp_body.get("hits")
                .and_then(Value::as_array)
                .ok_or(DownloadError::BadJson)?
                .iter()
                .map(FirdsDoc::from_fca_json)
                .collect::<Result<Vec<FirdsDoc>, DownloadError>>()?
        );
        start += rows;
    }
    Ok(docs)
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};
    use reqwest::Client;
    use crate::download::{search_esma, search_fca};

    #[tokio::test]
    async fn test_search_fca() {
        let client = Client::new();
        let all_docs = search_fca(
            &client,
            NaiveDate::from_ymd_opt(2024, 10, 15).expect("Bad date"),
            NaiveDate::from_ymd_opt(2024, 12, 31).expect("Bad date"),
            None
        ).await;
        assert!(all_docs.is_ok());
        assert_eq!(all_docs.unwrap().len(), 476);
    }

    #[tokio::test]
    async fn test_search_esma() {
        let client = Client::new();
        let all_docs = search_esma(
            &client,
            Utc.with_ymd_and_hms(2024, 10, 15, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
            None
        ).await;
        //assert!(all_docs.is_ok());
        assert_eq!(all_docs.unwrap().len(), 449);

    }
}