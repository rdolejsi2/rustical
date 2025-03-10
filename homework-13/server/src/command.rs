//! Command handling for the server.
//!
//! The module handles all commands (including a declarative help for all of them).

use crate::config::Config;
use crate::file::{post_process_image, store_file};
use common::log;
use common::util::flush;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::net::TcpStream;

struct Command {
    pub func: fn(&mut TcpStream, &str, &Config) -> Result<String, Box<dyn Error>>,
    pub description: String,
}

lazy_static! {
    static ref COMMANDS: HashMap<&'static str, Command> = {
        #[rustfmt::skip]
        let functions = [
            (".help", Command { func: help, description: "Lists all commands".to_string() }),
            (".file", Command { func: file, description: "Stores a generic file".to_string() }),
            (".image", Command { func: image, description: "Stores an image file".to_string() }),
            (".info", Command { func: info, description: "Logs an info text on server side".to_string() }),
        ];
        functions.into_iter().collect()
    };
}

fn help(_: &mut TcpStream, _: &str, _config: &Config) -> Result<String, Box<dyn Error>> {
    let commands = COMMANDS
        .iter()
        .map(|(name, command)| format!("  {}: {}", name, command.description))
        .collect::<Vec<_>>()
        .join("\n");
    let message = format!("Available commands:\n{}", commands);
    Ok(message)
}

fn info(_: &mut TcpStream, input: &str, _config: &Config) -> Result<String, Box<dyn Error>> {
    Ok(format!("Info received: {}", input))
}

fn file(stream: &mut TcpStream, input: &str, config: &Config) -> Result<String, Box<dyn Error>> {
    store_file(stream, input, &config.file_dir, None)
}

fn image(stream: &mut TcpStream, input: &str, config: &Config) -> Result<String, Box<dyn Error>> {
    store_file(stream, input, &config.image_dir, Some(post_process_image))
}

pub(crate) fn handle_command(
    stream: &mut TcpStream,
    input: &str,
    config: &Config,
) -> Result<String, Box<dyn Error>> {
    if !input.starts_with('.') {
        log!("Message: {}", input.trim());
        return Ok("".to_string());
    }

    let mut parts = input.splitn(2, ' ');
    let command = parts.next().unwrap();
    let input = parts.next().unwrap_or("");
    if let Some(command) = COMMANDS.get(command) {
        (command.func)(stream, input, config)
    } else {
        let commands = COMMANDS.keys().collect::<Vec<_>>();
        Err(format!("Invalid command {}, valid are: {:?}", command, commands).into())
    }
}
