use std::error::Error;
use std::fs;
use std::io::stdout;
use std::io::Write;
use std::path::Path;
use hostname;

pub fn flush() {
    if let Err(e) = stdout().flush() {
        eprintln!("Error flushing stdout: {}", e);
        return;
    }
}

pub fn ensure_directory(directory: &String) {
    let files_dir = Path::new(directory);
    if !files_dir.exists() {
        if let Err(e) = fs::create_dir(files_dir) {
            eprintln!("Failed to create storage directory: {}", e);
            std::process::exit(1);
        }
        println!("Created directory: {}/", directory);
    }
}

pub fn get_enum_variant_name<T>(_: &T) -> String {
    let type_name = std::any::type_name::<T>();
    type_name
        .split("::")
        .last()
        .unwrap_or("Unknown")
        .to_string()
}

pub fn get_hostname() -> Result<String, Box<dyn Error>> {
    let hostname = hostname::get()?;
    Ok(hostname.to_string_lossy().into_owned())
}
