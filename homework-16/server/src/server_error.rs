//! All possible errors that can occur on the server side.
use common::elog;
use common::message::ServerClientMessage;
use common::util::collect_error_messages;
use common::util::flush;
use common_proc_macro::EnumVariantName;
use std::error::Error;

/// Represents all possible errors that can occur on the server side.
///
/// The errors support retrieving enum variant names and error messages and error causes
/// for at least a partial traceability. The backtrace and other features typical from
/// other languages are not supported due to Rust limitations.
#[derive(thiserror::Error, Debug, EnumVariantName)]
pub enum ServerError {
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String, #[source] Option<Box<dyn Error>>),
    #[error("Image processing failed: {0}")]
    ImageProcessingFailed(String, #[source] Option<Box<dyn Error>>),
    #[error("General issue: {0}")]
    GeneralIssue(String, #[source] Option<Box<dyn Error>>),
    #[error("Message processing failed: {0}")]
    MessageProcessingFailed(String, #[source] Option<Box<dyn Error>>),
    #[error("Message receive failed: {0}")]
    MessageReceiveFailed(String, #[source] Option<Box<dyn Error>>),
    #[error("Response sending failed: {0}")]
    ResponseSendingFailed(String, #[source] Option<Box<dyn Error>>),
}

impl ServerError {
    /// Serializes the error into a client message.
    ///
    /// Please note: Rust seems to generalize error name when boxing into dyn Error,
    /// losing the real error name in the process (unless someone knows to which concrete
    /// error to downcast, which is of course unattainable in general frameworks handling
    /// the error on the top level only).
    ///
    /// While we make effort to truly print the actual error name, we don't expect
    /// real error names to be present in the server to client message due to the reasons
    /// described above. The clients will see just the generic `Error:`-prefixed texts, unfortunately.
    pub fn to_client_message(&self, msg_id_ref: Option<String>) -> ServerClientMessage {
        let msg_id = msg_id_ref.unwrap_or("".to_string()).clone();
        if let Some(source) = &self.source() {
            let source_message = collect_error_messages(source);
            elog!(
                "{}: {}: {}",
                self.variant_name(),
                self.to_string(),
                source_message
            );
            ServerClientMessage::Error {
                msg_id_ref: msg_id.into(),
                code: self.variant_name().to_string(),
                text: Some(format!("{}: {}", self.to_string(), source_message)),
            }
        } else {
            elog!("{}: {}", self.variant_name(), self);
            ServerClientMessage::Error {
                msg_id_ref: msg_id.into(),
                code: self.variant_name().to_string(),
                text: Some(self.to_string()),
            }
        }
    }
}
