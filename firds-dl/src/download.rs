use chrono::{DateTime, FixedOffset};
use serde_json::{Map, Value};
use std::str::FromStr;
use strum_macros::EnumString;

const ESMA_BASE_URL: &str = "https://registers.esma.europa.eu/solr/esma_registers_firds_files/";
const FCA_BASE_URL: &str = "https://api.data.fca.org.uk/fca_data_firds_files";

enum DownloadError {
    BadJson,
    BadEnum(strum::ParseError),
    /// Error parsing a [`chrono::DateTime`] from a string.
    DateTime(chrono::ParseError),
}

impl From<chrono::ParseError> for DownloadError {
    fn from(err: chrono::ParseError) -> DownloadError {
        Self::DateTime(err)
    }
}

impl From<serde_json::Error> for DownloadError {
    fn from(err: serde_json::Error) -> Self {
        Self::BadJson
    }
}

impl From<strum::ParseError> for DownloadError {
    fn from(err: strum::ParseError) -> Self {
        Self::BadEnum(err)
    }
}

/// Get a string from a JSON object, or return an error.
fn str_from_map<'a>(map: &'a Map<String, Value>, k: &str) -> Result<&'a str, DownloadError> {
    if let Some(Value::String(s)) = map.get(k) {
        Ok(s)
    } else {
        Err(DownloadError::BadJson)
    }
}

/// Get a map from a JSON object, or return an error.
fn map_from_map<'a>(map: &'a Map<String, Value>, k: &str) -> Result<&'a Map<String, Value>, DownloadError> {
    if let Some(Value::Object(m)) = map.get(k) {
        Ok(m)
    } else {
        Err(DownloadError::BadJson)
    }
}

#[derive(Debug, EnumString)]
pub(crate) enum FileType {
    #[strum(serialize = "FULINS")]
    Fulins,
    #[strum(serialize = "DLTINS")]
    Dltins,
    #[strum(serialize = "FULCAN")]
    Fulcan
}

/// A single document reference, returned by searching a FIRDS database (ESMA or FCA).
struct FirdsDoc {
    /// A URL to download the file from.
    download_link: String,
    /// An ID for the file.
    file_id: String,
    /// The name of the file.
    file_name: String,
    /// The type of the file.
    file_type: FileType,
    /// The timestamp of the document.
    timestamp: DateTime<FixedOffset>,
    /// MD5 checksum for the file, if present (should be present in ESMA data but not FCA data).
    checksum: Option<String>
}

impl FirdsDoc {
    fn from_esma_json(json: &Value) -> Result<Self, DownloadError> {
        if let Value::Object(map) = json {
            Ok(Self {
                download_link: str_from_map(map, "download_link")?.to_owned(),
                file_id: str_from_map(map, "id")?.to_owned(),
                file_name: str_from_map(map, "file_name")?.to_owned(),
                file_type: FileType::from_str(str_from_map(map, "file_type")?)?,
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
                download_link: str_from_map(source_json, "download_link")?.to_owned(),
                file_id: str_from_map(map, "_id")?.to_owned(),
                file_name: str_from_map(source_json, "file_name")?.to_owned(),
                file_type: FileType::from_str(str_from_map(source_json, "file_type")?)?,
                timestamp: DateTime::parse_from_rfc3339(str_from_map(source_json, "last_refreshed")?)?,
                checksum: None
            })
        } else {
            Err(DownloadError::BadJson)
        }
    }
}