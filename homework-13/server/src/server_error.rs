use common::elog;
use common::message::ServerClientMessage;
use common::util::collect_error_messages;
use common::util::flush;
use common_proc_macro::EnumVariantName;
use std::error::Error;

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
