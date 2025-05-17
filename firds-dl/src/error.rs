use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum DownloadError {
    /// Error parsing JSON.
    BadJson,
    /// Error parsing an enum value from text.
    BadEnum(strum::ParseError),
    /// Error parsing a [`DateTime`] from a string.
    BadDateTime(chrono::ParseError),
    /// Input/output error.
    IoError(io::Error),
    /// A file already exists.
    FileExists(PathBuf),
    /// An error was encountered when parsing a request.
    Request(reqwest::Error),
    /// File validation failed.
    Md5CheckFailed(String),
    /// Could not parse an md5 checksum from a string.
    BadMd5Sum(String)
}

impl From<chrono::ParseError> for DownloadError {
    fn from(err: chrono::ParseError) -> DownloadError {
        Self::BadDateTime(err)
    }
}

impl From<serde_json::Error> for DownloadError {
    fn from(_err: serde_json::Error) -> Self {
        Self::BadJson
    }
}

impl From<strum::ParseError> for DownloadError {
    fn from(err: strum::ParseError) -> Self {
        Self::BadEnum(err)
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