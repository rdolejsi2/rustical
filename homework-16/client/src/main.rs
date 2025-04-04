///! Client application connecting to a simple TCP server.
///! This client connects to a server, sends commands, and handles responses.
mod command;
mod stream_handler;
mod client_error;

use common::cli::{parse_args, CliArg};
use common::util::flush;
use common::{elog, log};
use std::net::TcpStream;
use stream_handler::handle_stream;
use crate::client_error::ClientError;

/// Main entry point to the client application.
/// It parses command line arguments, connects to a server.
/// Stream handling is delegated to the (stream_handler)[stream_handler] module.
fn main() {
    let args = [CliArg::Host, CliArg::Port, CliArg::Debug];
    let (host, port, debug) = match parse_args("client", &args) {
        Ok(params) => {
            let [host, port, debug]: [String; 3] =
                params.try_into().expect("Exactly 2 parameters expected");
            (host, port, debug)
        }
        Err(e) => {
            elog!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
    };

    let debug: bool = debug.parse().unwrap_or(false);
    let address = format!("{}:{}", host, port);
    log!("Connecting to {}", address);
    match TcpStream::connect(&address) {
        Ok(mut stream) => {
            loop {
                // Check if the stream is shut down
                if stream.peer_addr().is_err() {
                    elog!("Stream has been shut down.");
                    break;
                }
                if let Err(e) = handle_stream(&mut stream, debug) {
                    elog!("Error: {:?}", e);
                    if let ClientError::StreamShutdownError(_) = e {
                        break;
                    }
                }
            }
        }
        Err(e) => {
            elog!("Failed to connect to server: {}", e);
            std::process::exit(1);
        }
    }
}
