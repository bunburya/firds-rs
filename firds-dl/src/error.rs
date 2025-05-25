use std::fmt::Display;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DownloadError {
    /// Error parsing JSON.
    JsonParseError(serde_json::Error),
    /// JSON object does not contain the given member.
    JsonMapKeyNotFound(String),
    /// JSON not of expected shape.
    BadJson,
    /// Error parsing an enum value from text.
    EnumParseError(String),
    /// Error parsing a [`chrono::DateTime`] from a string.
    BadDateTime(chrono::ParseError),
    /// Input/output error.
    IoError(io::Error),
    /// A file already exists.
    FileExists(PathBuf),
    /// An error was encountered when parsing a request.
    Request(reqwest::Error),
    /// File validation failed. The contained string is the actual md5 sum of the file.
    Md5CheckFailed(String),
    /// Cannot verify file because no md5 checksum present.
    NoMd5Sum,
    /// Error extracting a file from a zip archive.
    ZipError(zip::result::ZipError),
    /// Error parsing or creating a URL.
    UrlError(url::ParseError),
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonParseError(e) => write!(f, "Error parsing JSON: {e}"),
            Self::JsonMapKeyNotFound(k) => write!(f, "Key not found in JSON object: {k}"),
            Self::BadJson => write!(f, "JSON was not in expected form"),
            Self::EnumParseError(e) => write!(f, "Could not parse enum value from string: {e}"),
            Self::BadDateTime(e) => write!(f, "Error parsing DateTime from string: {e}"),
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::FileExists(p) => write!(f, "File exists: {p:?}"),
            Self::Request(e) => write!(f, "Error with HTTP request: {e}"),
            Self::Md5CheckFailed(e) => write!(f, "MD5 checksum does not match: {e}"),
            Self::NoMd5Sum => write!(f, "No MD5 sum was provided"),
            Self::ZipError(e) => write!(f, "Error extracting file from zip archive: {e}"),
            Self::UrlError(e) => write!(f, "Error constructing URL: {e}"),
        }
    }
}

impl std::error::Error for DownloadError {}

impl From<chrono::ParseError> for DownloadError {
    fn from(err: chrono::ParseError) -> DownloadError {
        Self::BadDateTime(err)
    }
}

impl From<serde_json::Error> for DownloadError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParseError(err)
    }
}

impl From<io::Error> for DownloadError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}

impl From<zip::result::ZipError> for DownloadError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::ZipError(err)
    }
}

impl From<url::ParseError> for DownloadError {
    fn from(err: url::ParseError) -> Self {
        Self::UrlError(err)
    }
}