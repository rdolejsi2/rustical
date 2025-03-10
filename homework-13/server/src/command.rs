//! Command handling for the server.
//!
//! The module handles all commands (including a declarative help for all of them).

use crate::config::Config;
use crate::file::{post_process_image, store_file};
use crate::server_error::ServerError;
use common::message::{ClientServerMessage, ServerClientMessage};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;

struct Command {
    pub description: String,
    pub params: &'static [&'static str],
    pub func: fn(
        &ClientServerMessage,
        &[&str],
        &Config,
    ) -> Result<ServerClientMessage, Box<dyn Error>>,
}

lazy_static! {
    static ref COMMANDS: HashMap<&'static str, Command> = {
        let functions = [
            (
                "help",
                Command {
                    description: "Lists all commands".to_string(),
                    params: &[],
                    func: help,
                },
            ),
            (
                "file",
                Command {
                    description: "Stores a generic file".to_string(),
                    params: &["name", "checksum_type", "checksum", "content"],
                    func: file,
                },
            ),
            (
                "image",
                Command {
                    description: "Stores an image file".to_string(),
                    params: &["name", "checksum_type", "checksum", "content"],
                    func: image,
                },
            ),
            (
                "info",
                Command {
                    description: "Logs an info text on server side".to_string(),
                    params: &[],
                    func: info,
                },
            ),
        ];
        functions.into_iter().collect()
    };
}

fn help(
    message: &ClientServerMessage,
    params: &[&str],
    _config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    _ = get_message_params(message, params)?;
    let commands = COMMANDS
        .iter()
        .map(|(name, command)| format!("  {}: {}", name, command.description))
        .collect::<Vec<_>>()
        .join("\n");
    let text = format!("Available commands:\n{}", commands);
    Ok(ServerClientMessage {
        msg_id_ref: message.msg_id.to_string(),
        code: "Ok".into(),
        text: Some(text),
    })
}

fn info(
    message: &ClientServerMessage,
    params: &[&str],
    _config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    _ = get_message_params(message, params)?;
    Ok(ServerClientMessage {
        msg_id_ref: message.msg_id.to_string(),
        code: "Ok".into(),
        text: Some(format!("Info received: {:?}", message)),
    })
}

fn file(
    message: &ClientServerMessage,
    params: &[&str],
    config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    let [filename, content]: [&str; 2] = get_message_params(message, params)?;
    store_file(&filename, &content, &config.file_dir, None)?;
    Ok(ServerClientMessage {
        msg_id_ref: message.msg_id.to_string(),
        code: "Ok".into(),
        text: None,
    })
}

fn image(
    message: &ClientServerMessage,
    params: &[&str],
    config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    let [filename, content]: [&str; 2] = get_message_params(message, params)?;
    store_file(&filename, &content, &config.image_dir, Some(post_process_image))?;
    Ok(ServerClientMessage {
        msg_id_ref: message.msg_id.to_string(),
        code: "Ok".into(),
        text: None,
    })
}

fn get_message_params<const N: usize>(
    message: &ClientServerMessage,
    params: &[&str],
) -> Result<[&'static str; N], Box<dyn Error>> {
    params
        .iter()
        .map(|&param| {
            message
                .payload
                .as_ref()
                .and_then(|payload| payload.get(param))
                .ok_or_else(|| ServerError::InvalidParameter(param.to_string()).into())
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| ServerError::InvalidParameter("Parameter conversion error".into()).into())
}

pub(crate) fn handle_command(
    message: &ClientServerMessage,
    config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    if let Some(command) = COMMANDS.get(message.command.as_str()) {
        (command.func)(message, command.params, config)
    } else {
        let commands = COMMANDS.keys().collect::<Vec<_>>();
        Err(format!("Invalid command {}, valid are: {:?}", message.command, commands).into())
    }
}
