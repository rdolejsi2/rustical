//! Client connection-handling module.
//!
//! The module handles a single client connection and its stream processing.

use crate::command::handle_command;
use crate::config::Config;
use crate::server_error::ServerError;
use anyhow::{Context, Result};
use common::message::{ClientServerMessage, ServerClientMessage};
use common::{elog, log, util};
use common::util::flush;
use serde_json::from_slice;
use std::io::Read;
use std::net::TcpStream;
use uuid::uuid;

fn receive_message(stream: &mut TcpStream) -> Result<ClientServerMessage, ServerError> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).map_err(|e| ServerError::GeneralIssue(format!("Failed to read from stream: {:?}", e)))?;
    from_slice(&buffer).map_err(|_| ServerError::GeneralIssue("Failed to parse message".into()))
}

pub(crate) fn handle_stream(stream: &mut TcpStream, config: &Config) -> ServerClientMessage {
    let message_result = receive_message(stream);
    let response = match message_result {
        Ok(msg) => {
            let msg_id = msg.msg_id.clone(); // Clone the msg_id to avoid moving it
            log!("Client message: {:?}", msg);
            handle_command(&msg, config).map(|_| {
                log!("Ok: {:?}", msg);
                ServerClientMessage {
                    msg_id_ref: msg_id.clone(), // Clone msg_id again inside the closure
                    code: "Ok".into(),
                    text: None,
                }
            }).or_else(|e| {
                elog!("Server Error (app logic): {:?}", e);
                Err(ServerClientMessage {
                    msg_id_ref: msg_id.clone(), // Clone msg_id again inside the closure
                    code: util::get_enum_variant_name(&e),
                    text: Some(format!("{:?}", e)),
                })
            })
        },
        Err(e) => {
            elog!("Server Error (message receive): {:?}", e);
            Err(ServerClientMessage {
                msg_id_ref: uuid!("00000000-0000-0000-0000-000000000000").to_string(),
                code: util::get_enum_variant_name(&e),
                text: Some(format!("{:?}", e)),
            })
        },
    };

    response.unwrap_or_else(|error_message| error_message)
}

fn convert_params(params: Vec<String>) -> Result<[String; 4]> {
    params.try_into()
        .map_err(|e: Vec<String>| anyhow::Error::msg(format!("Incorrect param count: {:?}", e)))
        .context("Failed to convert params")
}
