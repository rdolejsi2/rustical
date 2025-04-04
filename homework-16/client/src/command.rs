//! Command handling for the server.
//!
//! The module handles all commands (including a declarative help for all of them).

use crate::client_error::ClientError;
use common::message::{ClientServerMessage, Payload};
use common::util::{base64_decode, base64_encode, flush};
use common::{log, util};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub struct Command {
    pub func: Option<fn(&str) -> Result<ClientServerMessage, ClientError>>,
    pub description: String,
}

lazy_static! {
    static ref CLIENT_COMMANDS: HashMap<&'static str, Command> = {
        #[rustfmt::skip]
        let functions = [
            (".info", Command { func: Some(info), description: "Sends an info text about the client to the server".to_string() }),
            (".msg", Command { func: Some(msg), description: "Sends a message".to_string() }),
            (".file", Command { func: Some(file), description: "Sends a file for storing into files/".to_string() }),
            (".image", Command { func: Some(image), description: "Sends an image for storing into images/".to_string() }),
            (".help", Command { func: Some(help), description: "Prints help from the server (supported commands)".to_string() }),
            (".quit", Command { func: Some(quit), description: "Terminates the client".to_string() }),
        ];
        functions.into_iter().collect()
    };
}

fn info(input: &str) -> Result<ClientServerMessage, ClientError> {
    let hostname = util::get_hostname().unwrap_or("unknown".to_string());
    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        payload: Some(Payload::Info {
            info: format!("{input}", input = input),
            hostname,
        }),
    })
}

fn msg(input: &str) -> Result<ClientServerMessage, ClientError> {
    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        payload: Some(Payload::Msg {
            text: input.to_string(),
        }),
    })
}

fn help(input: &str) -> Result<ClientServerMessage, ClientError> {
    if !input.is_empty() {
        return Err(ClientError::InvalidParameter(
            "Command '.help' has no arguments".into(),
        ));
    }
    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        payload: Some(Payload::Help {}),
    })
}

fn file(input: &str) -> Result<ClientServerMessage, ClientError> {
    let mut file = File::open(input).map_err(|e| ClientError::GeneralIssue(e.to_string()))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| ClientError::GeneralIssue(e.to_string()))?;
    let content_base64 = base64_encode(&buffer);

    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        payload: Some(Payload::File {
            filename: input.to_string(),
            content: content_base64,
        }),
    })
}

fn image(input: &str) -> Result<ClientServerMessage, ClientError> {
    let mut file = File::open(input).map_err(|e| ClientError::GeneralIssue(e.to_string()))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| ClientError::GeneralIssue(e.to_string()))?;
    let content_base64 = base64_encode(&buffer);

    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        payload: Some(Payload::Image {
            filename: input.to_string(),
            content: content_base64,
        }),
    })
}

fn quit(_input: &str) -> Result<ClientServerMessage, ClientError> {
    Ok(ClientServerMessage {
        msg_id: uuid::Uuid::new_v4().to_string(),
        payload: Some(Payload::Quit {}),
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

pub(crate) fn handle_command(input: &str) -> Result<ClientServerMessage, ClientError> {
    let (command, input) = if !input.starts_with('.') {
        (".msg", input)
    } else {
        let mut parts = input.splitn(2, ' ');
        match parts.next() {
            Some(cmd) => (cmd, parts.next().unwrap_or("")),
            None => return Err(ClientError::GeneralIssue("Failed to parse command".into())),
        }
    };

    if let Some(command_spec) = CLIENT_COMMANDS.get(command) {
        if let Some(func) = command_spec.func {
            func(input)
        } else {
            Err(ClientError::GeneralIssue(
                format!("Command {} is not registered for handling", command).into(),
            ))
        }
    } else {
        let commands = CLIENT_COMMANDS.keys().collect::<Vec<_>>();
        Err(ClientError::InvalidParameter(format!(
            "Invalid command '{}', valid are: {:?}",
            command, commands
        )))
    }
}
