//! File handling functions.
//!
//! This module contains functions for handling the file storage on the server
//! (file name deduction, receiving files, post-processing images).

use chrono::{SecondsFormat, Utc};
use common::log;
use common::util::flush;
use image::{ImageFormat, ImageReader};
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};
use std::net::TcpStream;
use std::path::Path;

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

pub(crate) fn store_file(
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
            Ok(format!(
                "Received {} bytes and converted to {} bytes in {}",
                size, new_size, new_target_file
            ))
        } else {
            Ok(format!("Stored {} bytes in {}", size, new_target_file))
        }
    } else {
        let mut file = File::create(target_file.clone())?;
        file.write_all(&buffer)?;
        Ok(format!("Stored {} bytes in {}", size, target_file))
    }
}

pub(crate) fn post_process_image(
    buffer: &[u8],
    target_file: &str,
) -> Result<(Vec<u8>, String, bool), Box<dyn Error>> {
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
