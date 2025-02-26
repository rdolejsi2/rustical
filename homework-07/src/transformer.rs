use crate::util::flush;
use csv::Reader;
use lazy_static::lazy_static;
use slug::slugify;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::sync::mpsc::{Receiver, Sender};

#[derive(PartialEq, Clone)]
pub enum Mode {
    CLI,
    Streaming,
}

pub struct Transformer {
    pub func: fn(&str) -> Result<String, Box<dyn Error>>,
    pub description: String,
}

lazy_static! {
    static ref TRANSFORMERS: HashMap<&'static str, Transformer> = {
        let transformers = [
            ("lower", Transformer { func: lowercase, description: "Converts to lowercase".to_string() }),
            ("alpha_lower", Transformer { func: alphabetic_lowercase, description: "Converts to lowercase (non-alphabetic chars not allowed)".to_string() }),
            ("upper", Transformer { func: uppercase, description: "Converts to uppercase".to_string() }),
            ("alpha_upper", Transformer { func: alphabetic_uppercase, description: "Converts to uppercase (non-alphabetic chars not allowed)".to_string() }),
            ("no_spaces", Transformer { func: no_spaces, description: "Removes all spaces".to_string() }),
            ("slugify", Transformer { func: slugify_text, description: "Converts to a URL-friendly slug".to_string() }),
            ("kebab", Transformer { func: kebab, description: "Converts to kebab-case".to_string() }),
            ("snake", Transformer { func: snake, description: "Converts to snake_case".to_string() }),
            ("hex", Transformer { func: hex, description: "Converts to hexadecimal representation".to_string() }),
            ("csv", Transformer { func: csv, description: "Formats CSV data from a specified file".to_string() }),
        ];
        transformers.into_iter().collect()
    };
}

fn lowercase(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.to_lowercase())
}

fn alphabetic_lowercase(input: &str) -> Result<String, Box<dyn Error>> {
    if input.chars().any(|c| !c.is_alphabetic()) {
        return Err("Input contains non-alphabetic characters".into());
    }
    Ok(input.to_lowercase())
}

fn uppercase(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.to_uppercase())
}

fn alphabetic_uppercase(input: &str) -> Result<String, Box<dyn Error>> {
    if input.chars().any(|c| !c.is_alphabetic()) {
        return Err("Input contains non-alphabetic characters".into());
    }
    Ok(input.to_lowercase())
}

fn no_spaces(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.replace(" ", ""))
}

fn slugify_text(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(slugify(input))
}

fn kebab(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.replace(" ", "-").to_lowercase())
}

fn snake(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.replace(" ", "_").to_lowercase())
}

fn hex(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.chars().map(|c| format!("{:02x}", c as u8)).collect())
}

fn get_column_widths(headers: &csv::StringRecord, records: &[csv::StringRecord]) -> Vec<usize> {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for record in records {
        for (i, field) in record.iter().enumerate() {
            if field.len() > widths[i] {
                widths[i] = field.len();
            }
        }
    }
    widths
}

fn format_csv_separator_line(widths: &[usize]) -> String {
    widths.iter()
        .map(|&w| format!("+{}", "-".repeat(w + 2)))
        .collect::<Vec<_>>()
        .join("") + "+\n"
}

fn format_csv_line(fields: &[&str], widths: &[usize]) -> String {
    fields.iter()
        .enumerate()
        .map(|(i, field)| format!("| {:<width$} ", field, width = widths[i]))
        .collect::<Vec<_>>()
        .join("") + "|\n"
}

fn csv(input: &str) -> Result<String, Box<dyn Error>> {
    // Open the file specified in the input
    let file = File::open(input)?;
    let mut reader = Reader::from_reader(file);
    let headers = reader.headers()?.clone();
    let records = reader.records().collect::<Result<Vec<_>, _>>()?;
    for record in &records {
        if record.len() != headers.len() {
            return Err(format!("Data vs. header mismatch: found {} columns, expected {}", record.len(), headers.len()).into());
        }
    }

    let widths = get_column_widths(&headers, &records);
    let separator = format_csv_separator_line(&widths);
    let header_line = format_csv_line(&headers.iter().collect::<Vec<_>>(), &widths);
    let value_lines: String = records.iter()
        .map(|record| format_csv_line(&record.iter().collect::<Vec<_>>(), &widths))
        .collect::<Vec<_>>()
        .join("");

    let result = format!("{}{}{}{}", separator, header_line, separator, value_lines) + &separator;
    Ok(result)
}

fn transform(mode: &str, input: &str) -> Result<String, Box<dyn Error>> {
    if let Some(transformer) = TRANSFORMERS.get(mode) {
        (transformer.func)(input)
    } else {
        let modes = TRANSFORMERS.keys().collect::<Vec<_>>();
        Err(format!("Invalid mode {}. Valid modes are: {:?}", mode, modes).into())
    }
}

pub(crate) fn run(mode: Mode, rx: Receiver<String>, tx_main: Sender<String>) {
    if mode == Mode::Streaming {
        println!("Transformer thread is running, available transformers:");
        let max_mode_len = TRANSFORMERS.keys().map(|k| k.len()).max().unwrap();
        let mut transformers: Vec<_> = TRANSFORMERS.iter().collect();
        transformers.sort_by_key(|&(mode, _)| mode);
        for (mode, transformer) in transformers {
            println!(
                "  {:<max_mode_len$} .. {}",
                mode,
                transformer.description,
                max_mode_len = max_mode_len
            );
        }
        flush();
    }
    tx_main.send("ready".to_string()).unwrap();

    while let Ok(input) = rx.recv() {
        // Process the input received from the supplier
        // split it to mode and parameter
        let mut parts = input.splitn(2, ' ');
        let transformer = parts.next().unwrap();
        let parameter = parts.next().unwrap_or("");
        match transformer {
            "exit" => {
                if mode == Mode::Streaming {
                    println!("Exiting transformer thread");
                }
                break;
            }
            _ => match transform(transformer, parameter) {
                Ok(output) => {
                    println!("{}", output);
                }
                Err(e) => {
                    eprintln!("Error transforming input: {}", e);
                }
            },
        }
    }
}
