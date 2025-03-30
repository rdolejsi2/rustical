//! Client connection-handling module.
//!
//! The module handles a single client connection and its stream processing.

use std::error::Error;
use crate::command::handle_command;
use crate::config::Config;
use crate::server_error::ServerError;
use anyhow::Result;
use common::log;
use common::message::{ClientServerMessage, ServerClientMessage};
use common::util::flush;
use serde_json::from_slice;
use std::io::{Read, Write};
use std::net::TcpStream;

fn receive_message(
    stream: &mut TcpStream,
    debug: bool,
) -> Result<ClientServerMessage, ServerError> {
    let mut length_buffer = [0; 4];
    stream.read_exact(&mut length_buffer).map_err(|e| {
        ServerError::MessageReceiveFailed("Failed to read message length".into(), Some(Box::new(e)))
    })?;
    let length = u32::from_be_bytes(length_buffer) as usize;

    let mut buffer = vec![0; length];
    stream.read_exact(&mut buffer).map_err(|e| {
        ServerError::MessageReceiveFailed("Failed to read message".into(), Some(Box::new(e)))
    })?;

    if debug {
        log!("Received stream: {:?}", String::from_utf8_lossy(&buffer));
    }

    let message: ClientServerMessage = from_slice(&buffer).map_err(|e| {
        ServerError::MessageProcessingFailed("Failed to parse message".into(), Some(Box::new(e)))
    })?;
    if debug {
        log!("{}", format!("{message}", message = message));
    }
    Ok(message)
}

fn send_response(
    stream: &mut TcpStream,
    response: &ServerClientMessage,
    debug: bool,
) -> Result<(), ServerError> {
    let response_json = serde_json::to_string(response).map_err(|e| {
        ServerError::ResponseSendingFailed("Failed to serialize response".into(), Some(Box::new(e)))
    })?;
    if debug {
        log!("Sending JSON: {}", response_json);
    }
    let length = response_json.len() as u32;
    stream.write_all(&length.to_be_bytes()).map_err(|e| {
        ServerError::ResponseSendingFailed(
            "Failed to send response length".into(),
            Some(Box::new(e)),
        )
    })?;
    stream.write_all(response_json.as_bytes()).map_err(|e| {
        ServerError::ResponseSendingFailed("Failed to send response".into(), Some(Box::new(e)))
    })?;
    stream.flush().map_err(|e| {
        ServerError::ResponseSendingFailed("Failed to flush stream".into(), Some(Box::new(e)))
    })?;
    Ok(())
}

pub fn handle_stream(stream: &mut TcpStream, config: &Config) {
    log!("Connected client from {}", stream.peer_addr().unwrap());
    let debug: bool = config.debug.parse().unwrap_or(false);
    'stream: {
        loop {
            let message_result = receive_message(stream, debug);
            let response = match message_result {
                Ok(msg) => {
                    if debug {
                        log!("Client message: {:?}", msg);
                    }
                    let result = handle_command(&msg, config);
                    result.or_else(|e| {
                        Err(ServerError::MessageProcessingFailed(
                            "Failed to process message".into(),
                            Some(e),
                        )
                        .to_client_message(Some(msg.msg_id)))
                    })
                }
                Err(e) => Err(e.to_client_message(None)),
            };

            let response_message = response.unwrap_or_else(|error_message| error_message);
            if let Err(e) = send_response(stream, &response_message, debug) {
                if let Some(io_err) = &e
                    .source()
                    .and_then(|source| source.downcast_ref::<std::io::Error>())
                {
                    if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                        log!("Client disconnected due to broken pipe");
                        break 'stream;
                    }
                }
                log!(
                    "{:?}",
                    ServerError::GeneralIssue("Failed to send response".into(), Some(Box::new(e)))
                );
            }
            if let ServerClientMessage::Quit { .. } = response_message {
                break;
            }
            if debug {
                log!("Response sent: {:?}", response_message);
            }
        }
    }
}
