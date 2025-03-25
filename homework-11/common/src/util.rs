use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::fs;

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
