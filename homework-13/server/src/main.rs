//! Server for the file sharing application.
//!
//! The server listens for incoming connections and processes them in separate threads.
//! The handling of each connection is delegated to the `client_handler` module.

mod stream_handler;
mod command;
mod config;
mod file;
mod server_error;

use anyhow::{Context, Result};
use common::cli::{parse_args, CliArg};
use common::util::{ensure_directory, flush};
use common::{elog, log};
use config::Config;
use std::net::TcpListener;
use std::thread;
use stream_handler::handle_stream;

fn main() -> Result<()> {
    #[rustfmt::skip]
    let args = [CliArg::Host, CliArg::Port, CliArg::FileDir, CliArg::ImageDir];
    let (host, port, file_dir, image_dir) = match parse_args("server", &args) {
        Ok(params) => {
            let [host, port, file_dir, image_dir]: [String; 4] =
                params.try_into().context("Incorrect param count")?;
            (host, port, file_dir, image_dir)
        }
        Err(e) => {
            elog!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
    };

    ensure_directory(&file_dir);
    ensure_directory(&image_dir);
    let config = Config {
        file_dir,
        image_dir,
    };

    let address = format!("{}:{}", host, port);
    log!("Starting server on {}", address);
    let listener = TcpListener::bind(&address).context("Failed to bind to address")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let addr = match stream.peer_addr() {
                    Ok(addr) => addr,
                    Err(e) => {
                        elog!("Failed to get peer address: {}", e);
                        continue;
                    }
                };
                let name = format!("client-{}", addr);
                let config = config.clone();
                if let Err(e) = thread::Builder::new()
                    .name(name)
                    .spawn(move || handle_stream(&mut stream, &config))
                {
                    elog!("Failed to spawn thread: {}", e);
                }
            }
            Err(e) => {
                elog!("Failed to accept connection: {}", e);
            }
        }
    }
    Ok(())
}
