use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Command error: {0}")]
    CommandError(String),
    #[error("Input error: {0}")]
    InputError(String),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("General issue: {0}")]
    GeneralIssue(String),
    #[error("Request serialization failed: {0}")]
    RequestSerializationFailed(String),
    #[error("Response error: {0}")]
    ResponseError(String),
    #[error("Stream shutdown error: {0}")]
    StreamShutdownError(String),
    #[error("Stream write error: {0}")]
    StreamWriteError(String),
}
