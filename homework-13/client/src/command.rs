//! Command handling for the server.
//!
//! The module handles all commands (including a declarative help for all of them).

use common::{log, util};
use common::message::{ClientServerMessage, Payload};
use common::util::flush;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use base64::encode;
use std::fs::File;
use std::io::Read;

pub struct Command {
    pub func: Option<fn(&str) -> Result<ClientServerMessage, Box<dyn Error>>>,
    pub description: String,
}

lazy_static! {
    static ref CLIENT_COMMANDS: HashMap<&'static str, Command> = {
        #[rustfmt::skip]
        let functions = [
            (".msg", Command { func: Some(msg), description: "Sends a message".to_string() }),
            (".file", Command { func: Some(file), description: "Sends a file for storing into files/".to_string() }),
            (".image", Command { func: Some(image), description: "Sends an image for storing into images/".to_string() }),
            (".info", Command { func: Some(info), description: "Sends an info text about the client to the server".to_string() }),
            (".help", Command { func: Some(help), description: "Prints help from the server (supported commands)".to_string() }),
            (".quit", Command { func: None, description: "Terminates the client".to_string() }),
        ];
        functions.into_iter().collect()
    };
}

fn msg(input: &str) -> Result<ClientServerMessage, Box<dyn Error>> {
    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        command: "msg".into(),
        payload: Some(Payload::Message {
            text: input.to_string(),
        }),
    })
}

fn info(input: &str) -> Result<ClientServerMessage, Box<dyn Error>> {
    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        command: "info".into(),
        payload: Some(Payload::Message {
            text: format!("Client is hailing from {hostname}: {input}", hostname = util::get_hostname().unwrap_or("unknown".to_string()), input = input),
        }),
    })
}

fn help(input: &str) -> Result<ClientServerMessage, Box<dyn Error>> {
    if !input.is_empty() {
        return Err("Command '.help' has no arguments".into());
    }
    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        command: "help".into(),
        payload: None,
    })
}

fn file(input: &str) -> Result<ClientServerMessage, Box<dyn Error>> {
    let mut file = File::open(input)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let content_base64 = encode(&buffer);

    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        command: "file".into(),
        payload: Some(Payload::File {
            filename: input.to_string(),
            content: content_base64.into_bytes(),
        }),
    })
}

fn image(input: &str) -> Result<ClientServerMessage, Box<dyn Error>> {
    let mut file = File::open(input)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let content_base64 = encode(&buffer);

    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        command: "image".into(),
        payload: Some(Payload::Image {
            filename: input.to_string(),
            content: content_base64.into_bytes(),
        }),
    })
}

pub(crate) fn print_commands() {
    log!("Available commands:");
    let max_command_len = CLIENT_COMMANDS.keys().map(|k| k.len()).max().unwrap();
    let mut commands: Vec<_> = CLIENT_COMMANDS.iter().collect();
    commands.sort_by_key(|&(command, _)| command);
    for (command, spec) in commands {
        log!(
            "  {:<max_command_len$} .. {}",
            command,
            spec.description,
            max_command_len = max_command_len
        );
    }
}

pub(crate) fn handle_command(input: &str) -> Result<ClientServerMessage, Box<dyn Error>> {
    let (command, input) = if !input.starts_with('.') {
        (".msg", input)
    } else {
        let mut parts = input.splitn(2, ' ');
        match parts.next() {
            Some(cmd) => (cmd, parts.next().unwrap_or("")),
            None => return Err("Failed to parse command".into()),
        }
    };

    if let Some(command_spec) = CLIENT_COMMANDS.get(command) {
        if let Some(func) = command_spec.func {
            func(input)
        } else {
            Err("Command '.quit' is not handled".into())
        }
    } else {
        let commands = CLIENT_COMMANDS.keys().collect::<Vec<_>>();
        Err(format!("Invalid command '{}', valid are: {:?}", command, commands).into())
    }
}
