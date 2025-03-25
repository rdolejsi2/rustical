use crate::util::flush;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

pub struct Command {
    pub func: Option<fn(&mut TcpStream, &str) -> Result<String, Box<dyn Error>>>,
    pub description: String,
}

lazy_static! {
    static ref CLIENT_COMMANDS: HashMap<&'static str, Command> = {
        #[rustfmt::skip]
        let functions = [
            (".file", Command { func: Some(file), description: "Sends a file to the server for storing into files/".to_string() }),
            (".image", Command { func: Some(image), description: "Sends an image to the server for storing into images/".to_string() }),
            (".info", Command { func: Some(info), description: "Sends an info text to the server (to be logged there)".to_string() }),
            (".help", Command { func: Some(help), description: "Requests help from server".to_string() }),
            (".quit", Command { func: None, description: "Terminates the client".to_string() }),
        ];
        functions.into_iter().collect()
    };
}

fn file(stream: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    println!("Starting to send file {}", input);
    send_command_with_content(stream, input, ".file", true)
}

fn image(stream: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    println!("Starting to send image {}", input);
    send_command_with_content(stream, input, ".image", true)
}

fn info(stream: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    send_command_with_content(stream, input, ".info", false)
}

fn send_command_with_content(stream: &mut TcpStream, input: &str, command: &str, is_file: bool) -> Result<String, Box<dyn Error>> {
    if input.is_empty() {
        return Err(format!("Command '{}' requires a {}parameter", command, if is_file { "<filename> " } else { "" }).into());
    }

    if is_file {
        send_file(stream, command, input)?;
    } else {
        stream.write_all(format!("{} {}\n", command, input).as_bytes())?;
    }

    receive_server_response(stream)
}

fn send_file(stream: &mut TcpStream, command: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let file_size = std::fs::metadata(file_name)?.len();
    stream.write_all(format!("{} {} {}\n", command, file_size, file_name).as_bytes())?;
    let mut file = File::open(file_name)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    stream.write_all(&content)?;
    println!("File {} sent", file_name);
    Ok(())
}

fn help(stream: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    if !input.is_empty() {
        return Err("Command '.help' has no arguments".into());
    }
    // send '.file input' to the server, wait for response
    if let Err(e) = stream.write_all(".help\n".as_bytes()) {
        return Err(e.into());
    }
    receive_server_response(stream)
}

fn receive_server_response(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();
    let mut response = String::new();

    while reader.read_line(&mut buffer)? > 0 {
        if buffer.trim().is_empty() {
            break;
        }
        response.push_str(&buffer);
        buffer.clear();
    }

    let trimmed_response = response.trim();
    if trimmed_response.starts_with("ERROR:") {
        Err(trimmed_response.into())
    } else {
        Ok(trimmed_response.into())
    }
}

fn handle_command(stream: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    if !input.starts_with('.') {
        stream.write_all(format!("{}\n", input).as_bytes())?;
        return receive_server_response(stream);
    }

    let mut parts = input.splitn(2, ' ');
    let command = parts.next().unwrap();
    let input = parts.next().unwrap_or("");
    if let Some(command_spec) = CLIENT_COMMANDS.get(command) {
        match command_spec.func {
            Some(func) => func(stream, input),
            None => Err("Command '.quit' is not handled".into()),
        }
    } else {
        let commands = CLIENT_COMMANDS.keys().collect::<Vec<_>>();
        Err(format!("Invalid command {}, valid are: {:?}", command, commands).into())
    }
}

fn print_commands() {
    println!("Available commands:");
    let max_command_len = CLIENT_COMMANDS.keys().map(|k| k.len()).max().unwrap();
    let mut commands: Vec<_> = CLIENT_COMMANDS.iter().collect();
    commands.sort_by_key(|&(command, _)| command);
    for (command, spec) in commands {
        println!(
            "  {:<max_command_len$} .. {}",
            command,
            spec.description,
            max_command_len = max_command_len
        );
    }
}

pub(crate) fn handle_stream(stream: &mut TcpStream) {
    println!("Connected to server, please input '.<cmd> <param>' (Ctrl+D or 'exit' to finish):");
    print_commands();
    flush();

    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let input = buffer.trim_end().to_string();
                match input.as_str().trim() {
                    "" => {
                        continue;
                    }
                    ".quit" => {
                        break;
                    }
                    _ => {
                        let result = handle_command(stream, &input);
                        match result {
                            Ok(response) => {
                                println!("Server: {}", response);
                            }
                            Err(e) => {
                                eprintln!("Error handling command: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

pub(crate) fn run(address: String) {
    println!("Connecting to {}", address);
    match TcpStream::connect(&address) {
        Ok(mut stream) => {
            handle_stream(&mut stream);
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            std::process::exit(1);
        }
    }
}
