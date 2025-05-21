use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum DownloadError {
    /// Error parsing JSON.
    JsonParseError(serde_json::Error),
    /// JSON object does not contain the given member.
    JsonMapKeyNotFound(String),
    /// JSON not of expected shape.
    BadJson,
    /// Error parsing an enum value from text.
    EnumParseError(String),
    /// Error parsing a [`DateTime`] from a string.
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