//! File handling functions.
//!
//! This module contains functions for handling the file storage on the server
//! (file name deduction, receiving files, post-processing images).

use crate::server_error::ServerError;
use chrono::{SecondsFormat, Utc};
use common::log;
use common::util::{base64_decode, flush};
use image::{ImageFormat, ImageReader};
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Write};
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
    filename: &str,
    directory: &str,
    content: &str,
    post_processor: Option<fn(&[u8], &str) -> Result<(Vec<u8>, String, bool), Box<dyn Error>>>,
) -> Result<String, Box<dyn Error>> {
    let buffer = base64_decode(content)
        .map_err(|e| ServerError::InvalidEncoding("Decoding failed".into(), Some(Box::new(e))))?;
    let target_file = get_target_file(filename, directory)?;
    if let Some(processor) = post_processor {
        let (target_buffer, new_target_file, converted) = processor(&buffer, &target_file)?;
        let mut target_file = File::create(new_target_file.clone())?;
        target_file.write_all(&target_buffer)?;
        let msg = if converted {
            format!(
                "Received {} bytes and converted to {} bytes in {}",
                buffer.len(),
                target_buffer.len(),
                new_target_file
            )
        } else {
            format!(
                "Stored {} bytes in {}",
                buffer.len(),
                new_target_file
            )
        };
        log!("{}", msg);
        Ok(msg)
    } else {
        let mut file = File::create(target_file.clone())?;
        file.write_all(&buffer)?;
        let msg = format!("Stored {} bytes in {}", buffer.len(), target_file);
        log!("{}", msg);
        Ok(msg)
    }
}

pub(crate) fn post_process_image(
    buffer: &[u8],
    target_file: &str,
) -> Result<(Vec<u8>, String, bool), Box<dyn Error>> {
    let img = ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()?
        .decode()
        .or_else(|e| {
            Err(ServerError::ImageProcessingFailed(
                "Failed to decode image".into(),
                Some(Box::new(e)),
            ))
        })?;

    let format = ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()?
        .format();

    match format {
        None => {
            Err(Box::new(ServerError::ImageProcessingFailed(
                "Unknown image format".into(),
                None,
            )))
        }
        Some(ImageFormat::Png) => {
            Ok((buffer.to_vec(), target_file.to_string(), false))
        }
        Some(format) => {
            log!("Converting from {:?}", format);

            let png_target_file = target_file.rsplit_once('.').map_or_else(
                || format!("{}.png", target_file),
                |(base, _)| format!("{}.png", base),
            );

            let mut png_buffer = Vec::new();
            img.write_to(&mut Cursor::new(&mut png_buffer), ImageFormat::Png)?;

            Ok((png_buffer, png_target_file, true))
        }
    }
}
