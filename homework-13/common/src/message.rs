use serde::{Deserialize, Serialize};
use serde_json::json;
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
    pub command: String,
    pub payload: Option<Payload>,
}

impl fmt::Debug for ClientServerMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut value = json!(self);
        if let Some(payload) = value.get_mut("payload") {
            match payload {
                Value::Object(map) => {
                    map.remove("text");
                    map.remove("content");
                }
                _ => {}
            }
        }
        write!(f, "{}", value)
    }
}

impl JsonParser for ClientServerMessage {
    fn from_json(input: &str) -> Result<Self, Box<dyn Error>> {
        serde_json::from_str(input).map_err(|e| e.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Payload {
    File {
        filename: String,
        content: Vec<u8>,
    },
    Image {
        filename: String,
        content: Vec<u8>,
    },
    // Add other variants as needed
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

#[derive(Serialize, Deserialize)]
pub struct ServerClientMessage {
    pub msg_id_ref: String,
    pub code: String,
    pub text: Option<String>,
}

impl JsonParser for ServerClientMessage {
    fn from_json(input: &str) -> Result<Self, Box<dyn Error>> {
        serde_json::from_str(input).map_err(|e| e.into())
    }
}
