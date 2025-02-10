use slug::slugify;
use std::env;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::Write;
use csv::Reader;

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
    widths.iter().map(|&w| format!("+{}", "-".repeat(w + 2))).collect::<Vec<_>>().join("") + "+\n"
}

fn format_csv_line(fields: &[&str], widths: &[usize]) -> String {
    fields.iter().enumerate().map(|(i, field)| format!("| {:<width$} ", field, width = widths[i])).collect::<Vec<_>>().join("") + "|\n"
}

fn csv(input: &str) -> Result<String, Box<dyn Error>> {
    let mut reader = Reader::from_reader(input.as_bytes());
    let headers = reader.headers()?.clone();
    let records = reader.records().collect::<Result<Vec<_>, _>>()?;

    // Check if all records have the same number of columns as the headers
    for record in &records {
        if record.len() != headers.len() {
            return Err(format!("Data line does not conform to header line: expected {} columns, found {} columns", headers.len(), record.len()).into());
        }
    }

    // Get the width of each column
    let widths = get_column_widths(&headers, &records);

    // Create a separator line
    let separator = format_csv_separator_line(&widths);

    // Create the header line
    let header_line = format_csv_line(&headers.iter().collect::<Vec<_>>(), &widths);

    // Create the value lines
    let value_lines: String = records.iter().map(|record| {
        format_csv_line(&record.iter().collect::<Vec<_>>(), &widths)
    }).collect::<Vec<_>>().join("");

    // Combine everything into the final output
    let result = format!("{}{}{}{}", separator, header_line, separator, value_lines) + &separator;
    Ok(result)
}

struct Transformer {
    func: fn(&str) -> Result<String, Box<dyn Error>>,
    description: String,
}

fn get_transformers() -> HashMap<&'static str, Transformer> {
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
        ("csv", Transformer { func: csv, description: "Formats CSV data".to_string() }),
    ];
    transformers.into_iter().collect()
}

fn transform(mode: &str, input: &str) -> Result<String, Box<dyn Error>> {
    let transformers = get_transformers();
    if let Some(transformer) = transformers.get(mode) {
        (transformer.func)(input)
    } else {
        Err(format!("Invalid mode {}. Valid modes are: {:?}", mode, transformers.keys().collect::<Vec<_>>()).into())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <transformation-mode>", args[0]);
        println!("Available transformation modes:");
        // get the longest mode name for padding
        let max_mode_len = get_transformers().keys().map(|k| k.len()).max().unwrap();
        let mut transformers: Vec<_> = get_transformers().into_iter().collect();
        // let's sort the transformers by mode name
        transformers.sort_by_key(|&(mode, _)| mode);
        for (mode, transformer) in transformers {
            // print modes (aligned to left with length of longest mode name)
            println!("  {:<max_mode_len$} .. {}", mode, transformer.description, max_mode_len = max_mode_len);
        }
        return;
    }
    let mode: &str = &args[1];

    println!("Please enter input text for performing {} (Ctrl+D to finish):", mode);
    if let Err(e) = io::stdout().flush() {
        eprintln!("Error flushing stdout: {}", e);
        return;
    }
    let mut input = String::new();
    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(0) => break, // EOF reached
            Ok(_) => input.push_str(&buffer),
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                return;
            }
        }
    }

    match transform(mode, &input) {
        Ok(output) => println!("Result: {}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}
