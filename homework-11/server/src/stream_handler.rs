//! Client connection-handling module.
//!
//! The module handles a single client connection and its stream processing.

use crate::command::handle_command;
use common::util::flush;
use common::{elog, estream, log, stream};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub(crate) fn handle_stream(stream: &mut TcpStream, config: &crate::Config) {
    log!("Accepted connection");
    let mut buffer = String::new();
    loop {
        buffer.clear();
        let mut reader = BufReader::new(&mut *stream);
        match reader.read_line(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(_) => {
                let input = buffer.trim_end().to_string();
                match handle_command(stream, &input, config) {
                    Ok(response) => {
                        stream!(stream, "{}\n\n", response);
                    }
                    Err(e) => {
                        estream!(stream, "{}\n\n", e);
                    }
                }
            }
            Err(e) => {
                elog!("Error reading from stream: {}", e);
                break;
            }
        }
    }
}
