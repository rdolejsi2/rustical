pub mod cli;
pub mod util;
pub mod message;
// all macros (exported at library level, hence not in a specific module)

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        println!("[{}] {}", std::thread::current().name().unwrap(), message.trim());
        flush();
    };
}

#[macro_export]
macro_rules! elog {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        eprintln!("[{}] {}", std::thread::current().name().unwrap(), message.trim());
        flush();
    };
}

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
