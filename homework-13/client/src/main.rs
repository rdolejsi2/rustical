mod command;
mod stream_handler;

use common::cli::{parse_args, CliArg};
use common::util::flush;
use common::{elog, log};
use std::net::TcpStream;
use stream_handler::handle_stream;

fn main() {
    let args = [CliArg::Host, CliArg::Port];
    let (host, port) = match parse_args("client", &args) {
        Ok(params) => {
            let [host, port]: [String; 2] =
                params.try_into().expect("Exactly 2 parameters expected");
            (host, port)
        }
        Err(e) => {
            elog!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
    };

    let address = format!("{}:{}", host, port);
    log!("Connecting to {}", address);
    match TcpStream::connect(&address) {
        Ok(mut stream) => {
            handle_stream(&mut stream);
        }
        Err(e) => {
            elog!("Failed to connect to server: {}", e);
            std::process::exit(1);
        }
    }
}
