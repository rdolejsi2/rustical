//! Stream handler module for client.
//!
//! The module handles communication stream with the server for the client.

use crate::command::{handle_command, print_commands};
use common::util::flush;
use common::{elog, log};
use std::io::stdin;
use std::net::TcpStream;

pub(crate) fn handle_stream(stream: &mut TcpStream) {
    log!("Connected to server, please input '.<cmd> <param>' (Ctrl+D or 'exit' to finish):");
    print_commands();
    flush();

    loop {
        let mut buffer = String::new();
        match stdin().read_line(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let input = buffer.trim_end().to_string();
                match input.as_str().trim() {
                    "" => {
                        continue;
                    }
                    ".quit" => {
                        break;
                    }
                    _ => {
                        let result = handle_command(stream, &input);
                        match result {
                            Ok(response) => {
                                log!("Server: {}", response);
                            }
                            Err(e) => {
                                elog!("Error handling command: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                elog!("Error reading input: {}", e);
                break;
            }
        }
    }
}
