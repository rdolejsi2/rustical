use crate::util::flush;
use crate::{FILE_DIRECTORY, IMAGE_DIRECTORY};
use chrono::{SecondsFormat, Utc};
use image::{ImageFormat, ImageReader};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{fs, thread};

pub struct Command {
    pub func: fn(&mut TcpStream, &str) -> Result<String, Box<dyn Error>>,
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

macro_rules! log {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        println!("[{}] {}", std::thread::current().name().unwrap(), message);
        flush();
    };
}

macro_rules! elog {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        eprintln!("[{}] {}", std::thread::current().name().unwrap(), message);
        flush();
    };
}

macro_rules! stream {
    ($stream:expr, $($arg:tt)*) => {
        let message = format!($($arg)*);
        if ! message.trim().is_empty() {
            print!("[{}] {}", std::thread::current().name().unwrap(), message);
        }
        $stream.write_all(message.as_bytes()).unwrap();
        flush();
    };
}

macro_rules! estream {
    ($stream:expr, $($arg:tt)*) => {
        let message = format!($($arg)*);
        let full_message = format!("ERROR: {}", message);
        eprint!("[{}] {}", std::thread::current().name().unwrap(), &full_message);
        $stream.write_all(full_message.as_bytes()).unwrap();
        flush();
    };
}

fn info(_: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    log!("received info: {}", input);
    Ok(format!("Info received: {}", input))
}

fn file(stream: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    store_file(stream, input, FILE_DIRECTORY, None)
}

fn image(stream: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    store_file(stream, input, IMAGE_DIRECTORY, Some(post_process_image))
}

fn store_file(
    stream: &mut TcpStream,
    input: &str,
    directory: &str,
    post_processor: Option<fn(&[u8], &str) -> Result<(Vec<u8>, String, bool), Box<dyn Error>>>,
) -> Result<String, Box<dyn Error>> {
    let mut parts = input.splitn(2, ' ');
    let size = parts.next().unwrap().parse::<usize>()?;
    let filename = parts.next().unwrap_or("");
    log!("Receiving {} (size {})", filename, size);
    receive_file(stream, filename, size, directory, post_processor)
}

fn receive_file(
    stream: &mut TcpStream,
    filename: &str,
    size: usize,
    directory: &str,
    post_processor: Option<fn(&[u8], &str) -> Result<(Vec<u8>, String, bool), Box<dyn Error>>>,
) -> Result<String, Box<dyn Error>> {
    let mut reader = BufReader::new(stream);
    let mut buffer = vec![0; size];
    reader.read_exact(&mut buffer)?;

    let target_file = get_target_file(filename, directory)?;
    if let Some(processor) = post_processor {
        let (target_buffer, new_target_file, converted) = processor(&buffer, &target_file)?;
        let mut target_file = File::create(new_target_file.clone())?;
        target_file.write_all(&target_buffer)?;
        if converted {
            // let's get new file size
            let new_size = target_buffer.len();
            Ok(format!("Received {} bytes and converted to {} bytes in {}", size, new_size, new_target_file))
        } else {
            Ok(format!("Stored {} bytes in {}", size, new_target_file))
        }
    } else {
        let mut file = File::create(target_file.clone())?;
        file.write_all(&buffer)?;
        Ok(format!("Stored {} bytes in {}", size, target_file))
    }
}

fn post_process_image(buffer: &[u8], target_file: &str) -> Result<(Vec<u8>, String, bool), Box<dyn Error>> {
    let img = match ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()?
        .decode()
    {
        Ok(img) => img,
        Err(_) => return Err("Failed to decode image".into()),
    };

    let format = ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()?
        .format()
        .ok_or("Unknown image format")?;

    if format == ImageFormat::Png {
        return Ok((buffer.to_vec(), target_file.to_string(), false));
    }

    log!("Converting from {:?}", format);

    let png_target_file = target_file.rsplit_once('.').map_or_else(
        || format!("{}.png", target_file),
        |(base, _)| format!("{}.png", base),
    );

    let mut png_buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_buffer), ImageFormat::Png)?;

    Ok((png_buffer, png_target_file, true))
}

fn get_target_file(filename: &str, directory: &str) -> Result<String, Box<dyn Error>> {
    let path = Path::new(filename);
    let filename = path.file_name().unwrap().to_str().unwrap();
    // We are not using a plain timestamp here as it's fairly unusable for the naked eye
    // Instead, we use standard ISO 8601 format typically used throughout the industry
    // But: we replace colons with dashes
    // (to accommodate an "unnamed" OS's issues with storing files like this - hint: not Unix  ;-))
    let re = Regex::new(r"[:+]").unwrap();
    let timestamp = re
        .replace_all(&Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true), "-")
        .to_string();
    let new_filename = format!("{}_{}", timestamp, filename);
    let target_path = Path::new(directory).join(&new_filename);
    Ok(target_path.to_str().unwrap().to_string())
}

fn help(_: &mut TcpStream, _: &str) -> Result<String, Box<dyn Error>> {
    let commands = COMMANDS
        .iter()
        .map(|(name, command)| format!("  {}: {}", name, command.description))
        .collect::<Vec<_>>()
        .join("\n");
    let message = format!("Available commands:\n{}", commands);
    Ok(message)
}

fn handle_command(stream: &mut TcpStream, input: &str) -> Result<String, Box<dyn Error>> {
    if !input.starts_with('.') {
        log!("Message: {}", input.trim());
        return Ok("".to_string());
    }

    let mut parts = input.splitn(2, ' ');
    let command = parts.next().unwrap();
    let input = parts.next().unwrap_or("");
    if let Some(command) = COMMANDS.get(command) {
        (command.func)(stream, input)
    } else {
        let commands = COMMANDS.keys().collect::<Vec<_>>();
        Err(format!("Invalid command {}, valid are: {:?}", command, commands).into())
    }
}

fn handle_stream(stream: &mut TcpStream) {
    log!("Accepted connection");
    let mut buffer = String::new();
    loop {
        buffer.clear();
        let mut reader = BufReader::new(&mut *stream);
        match reader.read_line(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(_) => {
                let input = buffer.trim_end().to_string();
                match handle_command(stream, &input) {
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

pub(crate) fn run(address: String) {
    ensure_directory(FILE_DIRECTORY);
    ensure_directory(IMAGE_DIRECTORY);

    println!("Starting server on {}", address);
    let listener = match TcpListener::bind(&address) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", address, e);
            std::process::exit(1);
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let addr = match stream.peer_addr() {
                    Ok(addr) => addr,
                    Err(e) => {
                        eprintln!("Failed to get peer address: {}", e);
                        continue;
                    }
                };
                let name = format!("client-{}", addr);
                if let Err(e) = thread::Builder::new()
                    .name(name)
                    .spawn(move || handle_stream(&mut stream))
                {
                    eprintln!("Failed to spawn thread: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn ensure_directory(directory: &str) {
    let files_dir = Path::new(directory);
    if !files_dir.exists() {
        if let Err(e) = fs::create_dir(files_dir) {
            eprintln!("Failed to create storage directory: {}", e);
            std::process::exit(1);
        }
        println!("Created directory: files/");
    }
}
