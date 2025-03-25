use std::io::stdout;
use std::io::Write;

pub(crate) fn flush() {
    if let Err(e) = stdout().flush() {
        eprintln!("Error flushing stdout: {}", e);
        return;
    }
}
