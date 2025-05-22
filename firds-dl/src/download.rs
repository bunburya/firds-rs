use crate::error::DownloadError;
use crate::error::DownloadError::{BadJson, JsonMapKeyNotFound};
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use clap::ValueEnum;
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

const ESMA_BASE_URL: &str = "https://registers.esma.europa.eu/solr/esma_registers_firds_files/select";
const FCA_BASE_URL: &str = "https://api.data.fca.org.uk/fca_data_firds_files";


/// Get a string from a JSON object, or return an error.
fn str_from_map<'a>(map: &'a Map<String, Value>, k: &str) -> Result<&'a str, DownloadError> {
    map.get(k)
        .ok_or(JsonMapKeyNotFound(k.to_owned()))?
        .as_str()
        .ok_or(BadJson)
}

/// Get a map from a JSON object, or return an error.
fn map_from_map<'a>(map: &'a Map<String, Value>, k: &str) -> Result<&'a Map<String, Value>, DownloadError> {
    map.get(k)
        .ok_or(JsonMapKeyNotFound(k.to_owned()))?
        .as_object()
        .ok_or(BadJson)
}

/// Get a map from a JSON value, which is assumed to represent a JSON object. Return an error if the
/// value is not an object.
fn str_from_value<'a>(value: &'a Value, k: &str) -> Result<&'a str, DownloadError> {
    if let Value::Object(m) = value {
        str_from_map(m, k)
    } else {
        Err(BadJson)
    }
}

/// Get a map from a JSON value, which is assumed to represent a JSON object. Return an error if the
/// value is not an object.
fn map_from_value<'a>(value: &'a Value, k: &str) -> Result<&'a Map<String, Value>, DownloadError> {
    if let Value::Object(m) = value {
        map_from_map(m, k)
    } else {
        Err(BadJson)
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub(crate) enum FirdsDocType {
    Fulins,
    Dltins,
    Fulcan
}

impl FromStr for FirdsDocType {
    type Err = DownloadError;

    /// Parse an *upper-case* string (one of "FULINS", "DLTINS" or "FULCAN") into a variant of
    /// [`FirdsDocType`].
    ///
    /// **NOTE**: [`FirdsDocType`] also has an implementation of `from_str` which is provided as a
    /// result of implementing [`ValueEnum`]. That function is used to parse command line arguments
    /// and, unlike this one, operates on *lower-case* strings.
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
#[derive(Debug, Copy, Clone, ValueEnum)]
pub(crate) enum FirdsSource {
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
    /// The source of the file.
    pub source: FirdsSource,
    /// A URL to download the file from.
    pub download_link: String,
    /// An ID for the file.
    pub file_id: String,
    /// The name of the file.
    pub file_name: String,
    /// The type of the file.
    pub file_type: FirdsDocType,
    /// The timestamp of the document.
    pub timestamp: DateTime<FixedOffset>,
    /// MD5 checksum for the file, if present (should be present in ESMA data but not FCA data).
    pub checksum: Option<String>
}

impl FirdsDoc {
    fn from_esma_json(json: &Value) -> Result<Self, DownloadError> {
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
    
    fn from_fca_json(json: &Value) -> Result<Self, DownloadError> {
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
            Err(BadJson)
        }
    }

    /// Verify the md5 checksum of the file at the given path. Assumes that a checksum is present in
    /// the struct, returning an error if not.
    fn verify_file(&self, fpath: &Path) -> Result<(), DownloadError> {
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

    /// Download the zip file from the source.
    ///
    /// The file is downloaded in chunks and saved to a `.part` file, which is then renamed to the
    /// expected file name once download is complete.
    ///
    /// This method is wrapped by source-specific methods which should be called by the user (see
    /// [`FirdsDoc::download_esma_zip`] and [`FirdsDoc::download_fca_zip`]).
    async fn download_zip(
        &self,
        client: &Client,
        to_dir: &Path,
        overwrite: bool,
        verify: bool
    ) -> Result<PathBuf, DownloadError> {
        if !to_dir.exists() {
            create_dir_all(to_dir)?
        }
        let fpath = to_dir.join(&self.file_name);
        let mut fpath_part = OsString::from(&fpath);
        fpath_part.push(".part");
        let fpath_part: PathBuf = fpath_part.into();
        if !overwrite {
            if fpath.exists() {
                return Err(DownloadError::FileExists(fpath))
            } else if fpath_part.exists() {
                return Err(DownloadError::FileExists(fpath_part))
            }
        } else if fpath_part.exists() {
            remove_file(&fpath_part)?
        }
        let resp = client.get(&self.download_link).send().await?;
        let mut file = File::create(&fpath_part)?;
        let mut stream = resp.bytes_stream();
        while let Some(item) = stream.next().await {
            file.write_all(&item?)?;
        }
        if verify {
            self.verify_file(&fpath_part)?;
        }
        rename(&fpath_part, &fpath)?;
        Ok(fpath)
    }


    /// Download the zip file from the ESMA FIRDS database.
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
    async fn download_esma_zip(
        &self,
        client: &Client,
        to_dir: &Path,
        overwrite: bool,
        verify: bool
    ) -> Result<PathBuf, DownloadError> {
        self.download_zip(client, to_dir, overwrite, verify).await
    }

    /// Download the zip file from the FCA FIRDS database.
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
    async fn download_fca_zip(
        &self,
        client: &Client,
        to_dir: &Path,
        overwrite: bool
    ) -> Result<PathBuf, DownloadError> {
        self.download_zip(client, to_dir, overwrite, false).await
    }

    async fn download_xml(
        &self,
        client: &Client,
        to_dir: &Path,
        overwrite: bool,
        verify: bool,
        delete_xml: bool
    ) -> Result<PathBuf, DownloadError> {
        let zip_fpath = self.download_zip(client, to_dir, overwrite, verify).await?;
        let zipped_file = File::open(&zip_fpath)?;
        let mut archive = zip::ZipArchive::new(zipped_file)?;
        let mut zip_file = archive.by_name(&self.file_name)?;
        let unzipped_fpath = to_dir.join(&self.file_name);
        if !overwrite && unzipped_fpath.exists() {
            return Err(DownloadError::FileExists(unzipped_fpath))
        }
        let mut unzipped_file = File::create(&unzipped_fpath)?;
        io::copy(&mut zip_file, &mut unzipped_file)?;
        if delete_xml {
            remove_file(&zip_fpath)?;
        }
        Ok(unzipped_fpath)
    }
}

/// A single page of document results obtained by searching the ESMA or FCA firds database.
struct ResultsPage {
    /// Where in the results we are starting from.
    start: u32,
    /// How many rows were searched for (the number of actual results may be less).
    rows: u32,
    /// The documents returned by the search.
    docs: Vec<FirdsDoc>
}

/// Search the ESMA FIRDS database for files from the given time period and, if applicable, of the
/// given type.
pub(crate) async fn search_esma(
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
        let resp_body = json.get("response").and_then(Value::as_object).ok_or(BadJson)?;
        if num_found < 0 {
            num_found= resp_body.get("numFound").and_then(Value::as_i64).ok_or(BadJson)?;
        }
        let resp = json.get("response").and_then(Value::as_object).ok_or(BadJson)?;
        docs.extend(
            resp.get("docs")
                .and_then(Value::as_array)
                .ok_or(BadJson)?
                .iter()
                .map(FirdsDoc::from_esma_json)
                .collect::<Result<Vec<FirdsDoc>, DownloadError>>()?
        );
        start += rows;
    }
    Ok(docs)
}

pub(crate) async fn search_fca(
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
        let resp_body = json.get("hits").and_then(Value::as_object).ok_or(BadJson)?;
        if num_found < 0 {
            num_found= resp_body.get("total").and_then(Value::as_i64).ok_or(BadJson)?;
        }
        docs.extend(
            resp_body.get("hits")
                .and_then(Value::as_array)
                .ok_or(BadJson)?
                .iter()
                .map(FirdsDoc::from_fca_json)
                .collect::<Result<Vec<FirdsDoc>, DownloadError>>()?
        );
        start += rows;
    }
    Ok(docs)
}