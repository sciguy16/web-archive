//! Module for the error parsing functionality

use std::string::FromUtf8Error;

/// Error type used by `web_archive` to wrap the errors returned by
/// operations in this crate or errors from other sources (e.g. URL
/// parsing or network errors).
#[derive(Debug)]
pub enum Error {
	/// Some kind of parsing error
    ParseError(String),
    /// Error fetching a resource
    ReqwestError(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::ParseError(e.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Self::ParseError(e.to_string())
    }
}
