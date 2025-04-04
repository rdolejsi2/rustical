//! Common library for the project
//!
//! This library contains common code that is used by all other libraries in the project.
//! Any pre-determined defaults shared between client and server are part of this library as well.
pub mod cli;
pub mod message;
pub mod util;
mod test;
// all macros (exported at library level, hence not in a specific module)

/// Prints a message to the standard output stream and flushes it.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        println!("[{}] {}", std::thread::current().name().unwrap(), message.trim());
        flush();
    };
}

/// Prints an error message to the standard error stream and flushes it.
#[macro_export]
macro_rules! elog {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        eprintln!("[{}] {}", std::thread::current().name().unwrap(), message.trim());
        flush();
    };
}

/// Prints a message to the standard output stream and writes it to the provided stream.
#[macro_export]
macro_rules! stream {
    ($stream:expr, $($arg:tt)*) => {
        let message = format!($($arg)*);
        if ! message.trim().is_empty() {
            println!("[{}] {}", std::thread::current().name().unwrap(), message.trim());
        }
        $stream.write_all(message.as_bytes()).unwrap();
        flush();
    };
}

/// Prints an error message to the standard error stream and writes it to the provided stream.
#[macro_export]
macro_rules! estream {
    ($stream:expr, $($arg:tt)*) => {
        let message = format!($($arg)*);
        let full_message = format!("ERROR: {}", message);
        eprintln!("[{}] {}", std::thread::current().name().unwrap(), &full_message.trim());
        $stream.write_all(full_message.as_bytes()).unwrap();
        flush();
    };
}
