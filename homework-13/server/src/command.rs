//! Command handling for the server.
//!
//! The module handles all commands (including a declarative help for all of them).

//! Command handling for the server.
//!
//! The module handles all commands (including a declarative help for all of them).

use crate::config::Config;
use crate::file::{post_process_image, store_file};
use common::message::{ClientServerMessage, Payload, ServerClientMessage};
use common::{log, util};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use util::flush;

struct Command {
    pub description: String,
    pub func: fn(&ClientServerMessage, &Config) -> Result<ServerClientMessage, Box<dyn Error>>,
}

lazy_static! {
    static ref COMMANDS: HashMap<&'static str, Command> = {
        let functions = [
            (
                "Help",
                Command {
                    description: "Lists all commands".to_string(),
                    func: help,
                },
            ),
            (
                "Info",
                Command {
                    description: "Logs an info text on server side".to_string(),
                    func: info,
                },
            ),
            (
                "Msg",
                Command {
                    description: "Sends a message".to_string(),
                    func: msg,
                },
            ),
            (
                "File",
                Command {
                    description: "Stores a generic file".to_string(),
                    func: file,
                },
            ),
            (
                "Image",
                Command {
                    description: "Stores an image file".to_string(),
                    func: image,
                },
            ),
            (
                "Quit",
                Command {
                    description: "Finalizes the communication, closes the stream".to_string(),
                    func: quit,
                },
            ),
        ];
        functions.into_iter().collect()
    };
}

fn help(
    message: &ClientServerMessage,
    _config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    let commands = COMMANDS
        .iter()
        .map(|(name, command)| format!("  {}: {}", name, command.description))
        .collect::<Vec<_>>()
        .join("\n");
    let text = format!("Available commands:\n{}", commands);
    Ok(ServerClientMessage::Ok {
        msg_id_ref: message.msg_id.to_string(),
        text: Some(text),
    })
}

fn info(
    message: &ClientServerMessage,
    _config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    match &message.payload {
        Some(Payload::Info { info, hostname }) => {
            log!(
                "Information received from client on hostname {}: {}",
                hostname,
                info
            );
            Ok(ServerClientMessage::Ok {
                msg_id_ref: message.msg_id.to_string(),
                text: Some(format!("Info received: {}", info)),
            })
        }
        _ => Err(format!("Invalid message type: {}", message).into()),
    }
}

fn msg(
    message: &ClientServerMessage,
    _config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    match &message.payload {
        Some(Payload::Msg { text }) => {
            log!("Message received from client: {}", text);
            Ok(ServerClientMessage::Ok {
                msg_id_ref: message.msg_id.to_string(),
                text: Some(format!("Message received: {}", text)),
            })
        }
        _ => Err(format!("Invalid message type: {}", message).into()),
    }
}

fn file(
    message: &ClientServerMessage,
    config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    match &message.payload {
        Some(Payload::File { filename, content }) => {
            let result = store_file(&filename, &config.file_dir, &content, None)?;
            Ok(ServerClientMessage::Ok {
                msg_id_ref: message.msg_id.to_string(),
                text: Some(result),
            })
        }
        _ => Err(format!("Invalid message type: {}", message).into()),
    }
}

fn image(
    message: &ClientServerMessage,
    config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    match &message.payload {
        Some(Payload::Image { filename, content }) => {
            let result = store_file(
                &filename,
                &config.image_dir,
                &content,
                Some(post_process_image),
            )?;
            Ok(ServerClientMessage::Ok {
                msg_id_ref: message.msg_id.to_string(),
                text: Some(result),
            })
        }
        _ => Err(format!("Invalid message type: {}", message).into()),
    }
}

fn quit(
    message: &ClientServerMessage,
    _config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    Ok(ServerClientMessage::Quit {
        msg_id_ref: message.msg_id.to_string(),
        text: Some("Hasta la vista".to_string()),
    })
}

pub(crate) fn handle_command(
    message: &ClientServerMessage,
    config: &Config,
) -> Result<ServerClientMessage, Box<dyn Error>> {
    if let Some(payload) = &message.payload {
        let command_name = payload.variant_name();
        if let Some(command_spec) = COMMANDS.get(command_name) {
            (command_spec.func)(message, config)
        } else {
            let commands = COMMANDS.keys().collect::<Vec<_>>();
            Err(format!(
                "Invalid command {}, valid are: {:?}",
                command_name, commands
            )
            .into())
        }
    } else {
        Err("No payload found in message".into())
    }
}
