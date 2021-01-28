#[derive(Debug)]
pub enum Error {
    ParseError(String),
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
