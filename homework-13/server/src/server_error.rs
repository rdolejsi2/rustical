use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),
    #[error("General issue: {0}")]
    GeneralIssue(String),
    #[error("Unknown error")]
    Unknown,
}
