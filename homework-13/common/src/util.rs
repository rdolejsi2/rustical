use base64::{alphabet, engine::{self, general_purpose}, DecodeError, Engine as _};
use hostname;
use std::any::type_name;
use std::error::Error;
use std::fs;
use std::io::stdout;
use std::io::Write;
use std::path::Path;

const BASE64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

pub fn base64_encode(data: &[u8]) -> String {
    BASE64_ENGINE.encode(data)
}

pub fn base64_decode(data: &str) -> Result<Vec<u8>, DecodeError> {
    BASE64_ENGINE.decode(data)
}

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

/// Prints the full error message including its sources.
///
/// Rust seems to not really have a single, easy way to print the error message including
/// all its clause/source messages, so the user actually hardly understands what happened.
///
/// This function is a workaround to print the error message in a human-readable way, with
/// full info about the error and its sources. This way, we don't get just a generic
/// 'DecodeError: Decoding failed' error, for example, but the actual originating source error
/// message from the bottom of the chain.
///
/// But please note: any usage of dyn Error seems to basically lose the actual information
/// about the error type and replaces it with a generic core::error::Error, which throws
/// the whole traceability out of the window. Added to the fact that there seems to be
/// no stacktrace, this is a bit of a mess and far cry from transparency we are used
/// from languages which honor stacktrace with all error / exception handling. I wonder
/// if this will be improved in the future or I'm missing some substantial piece of information -
/// I would assume that Rust, with its compilation always from sources would put traceability
/// to the source code line and the error type directly in each error object as a set of
/// first-class citizen fields.
///
/// (Please indicate if this all is not true and there is a better way to do this, an easy
/// way to print a stack trace indicating where the error happened, how it propagated through
/// the code, and what the actual error type is. I'm sure this must work somehow, I cannot
/// imagine that Rust is so limited in this regard. It would otherwise mean that panicking
/// is actually much better way how to achieve traceability and error handling in Rust,
/// which would be a surprise). Perhaps something like a backtrace crate can be enabled
/// to automatically decorate all errors with a backtrace? Not sure.
pub fn collect_error_messages(err: &(dyn Error)) -> String {
    let mut messages = vec![format!("{}: {}", simple_name_of(type_name_of_val(err)), err)];
    let mut source = err.source();
    while let Some(src) = source {
        messages.push(format!("{}: {}", simple_name_of(type_name_of_val(src)), src));
        source = src.source();
    }
    messages.join(": ")
}

pub fn print_error_stack_trace(err: &(dyn Error)) {
    let mut current_error: Option<&(dyn Error)> = Some(err);
    while let Some(e) = current_error {
        println!("{}: {}", type_name_of_val(e), e);
        current_error = e.source();
    }
}

pub fn type_name_of_val<T: ?Sized>(_: &T) -> &str {
    type_name::<T>()
}

pub fn simple_name_of(full_name: &str) -> &str {
    full_name.rsplit("::").next().unwrap_or(full_name)
}
