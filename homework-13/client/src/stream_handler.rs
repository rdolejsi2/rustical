//! Stream handler module for client.
//!
//! The module handles communication stream with the server for the client.

use crate::command::{handle_command, print_commands};
use common::util::flush;
use common::{elog, log};
use std::io::{stdin, Write};
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
                    "" => continue,
                    ".quit" => break,
                    _ => {
                        match handle_command(&input) {
                            Ok(response) => {
                                if let Err(e) = writeln!(stream, "{:?}", response) {
                                    elog!("Error writing to stream: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                elog!("Error: {}", e);
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
