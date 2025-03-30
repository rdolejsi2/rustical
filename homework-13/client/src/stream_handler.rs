//! Stream handler module for client.
//!
//! The module handles communication stream with the server for the client.

use crate::client_error::ClientError;
use crate::command::{handle_command, print_commands};
use common::message::ServerClientMessage;
use common::util::flush;
use common::log;
use serde_json::from_slice;
use std::io;
use std::io::{BufRead, Read, Write};
use std::net::TcpStream;

fn receive_response(stream: &mut TcpStream) -> Result<ServerClientMessage, ClientError> {
    let mut length_buffer = [0; 4];
    stream.read_exact(&mut length_buffer).map_err(|e| {
        ClientError::GeneralIssue(format!("Failed to read response length: {:?}", e))
    })?;
    let length = u32::from_be_bytes(length_buffer) as usize;

    let mut buffer = vec![0; length];
    stream
        .read_exact(&mut buffer)
        .map_err(|e| ClientError::GeneralIssue(format!("Failed to read response: {:?}", e)))?;

    let response: ServerClientMessage = from_slice(&buffer)
        .map_err(|_| ClientError::GeneralIssue("Failed to parse response".into()))?;
    Ok(response)
}

pub(crate) fn handle_stream(stream: &mut TcpStream, debug: bool) -> Result<(), ClientError> {
    log!("Connected to server, please input '.<cmd> <param>' (Ctrl+D or 'exit' to finish):");
    print_commands();
    flush();

    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut buffer = String::new();
        if handle.read_line(&mut buffer).is_err() {
            return Err(ClientError::InputError("Error reading input".into()));
        }

        let input = buffer.trim_end().to_string();
        if input.is_empty() {
            continue;
        }

        match handle_command(&input) {
            Ok(request) => {
                match serde_json::to_string(&request) {
                    Ok(serialized) => {
                        if debug {
                            log!("Sending JSON: {}", serialized);
                        }
                        let length = serialized.len() as u32;
                        if stream.write_all(&length.to_be_bytes()).is_err()
                            || stream.write_all(serialized.as_bytes()).is_err()
                            || stream.flush().is_err()
                        {
                            return Err(ClientError::StreamWriteError("Error writing to stream".into()));
                        }

                        match receive_response(stream) {
                            Ok(response) => {
                                if debug {
                                    log!("Received response: {:?}", response);
                                }
                                log!("{}", format!("{response}", response = response));
                                if let ServerClientMessage::Quit { .. } = response {
                                    break;
                                }
                            }
                            Err(e) => return Err(ClientError::ResponseError(format!("Error receiving response: {:?}", e))),
                        }
                    }
                    Err(e) => return Err(ClientError::RequestSerializationFailed(format!("Error serializing request: {:?}", e))),
                }
            }
            Err(e) => return Err(ClientError::CommandError(format!("Error handling command: {:?}", e))),
        }
    }

    // Close the connection when quitting
    if stream.shutdown(std::net::Shutdown::Both).is_err() {
        return Err(ClientError::StreamShutdownError("Error shutting down the connection".into()));
    }

    Ok(())
}
