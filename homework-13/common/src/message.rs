use common_proc_macro::EnumVariantName;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt;

pub trait JsonParser {
    fn from_json(input: &str) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

#[derive(Serialize, Deserialize)]
pub struct ClientServerMessage {
    pub msg_id: String,
    pub payload: Option<Payload>,
}

impl fmt::Debug for ClientServerMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientServerMessage")
            .field("msg_id", &self.msg_id)
            .field("payload", &self.payload)
            .finish()
    }
}

impl fmt::Display for ClientServerMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl JsonParser for ClientServerMessage {
    fn from_json(input: &str) -> Result<Self, Box<dyn Error>> {
        serde_json::from_str(input).map_err(|e| e.into())
    }
}

#[derive(Serialize, Deserialize, Debug, EnumVariantName)]
#[serde(tag = "type", content = "data")]
pub enum Payload {
    Help {},
    Info { info: String, hostname: String },
    Msg { text: String },
    File { filename: String, content: String },
    Image { filename: String, content: String },
    Quit {},
}

impl Payload {
    pub fn get(&self, key: &str) -> Option<String> {
        let json_value = serde_json::to_value(self).ok()?;
        match json_value {
            Value::Object(map) => map.get(key).and_then(|v| v.as_str().map(|s| s.to_string())),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum ServerClientMessage {
    Ok {
        msg_id_ref: String,
        text: Option<String>,
    },
    Error {
        msg_id_ref: String,
        code: String,
        text: Option<String>,
    },
    Quit {
        msg_id_ref: String,
        text: Option<String>,
    },
}

impl JsonParser for ServerClientMessage {
    fn from_json(input: &str) -> Result<Self, Box<dyn Error>> {
        serde_json::from_str(input).map_err(|e| e.into())
    }
}

impl fmt::Display for ServerClientMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerClientMessage::Ok { text, .. } => {
                let text_str = text.as_deref().map_or(String::new(), |t| format!(": {}", t));
                write!(f, "Ok{}", text_str)
            }
            ServerClientMessage::Error { code, text, .. } => {
                let code_str = if code.is_empty() { String::new() } else { format!("[{}]", code) };
                let text_str = text.as_deref().map_or(String::new(), |t| format!(": {}", t));
                write!(f, "Error{}{}", code_str, text_str)
            }
            ServerClientMessage::Quit { text, .. } => {
                let text_str = text.as_deref().map_or(String::new(), |t| format!(": {}", t));
                write!(f, "Quit{}", text_str)
            }
        }
    }
}
