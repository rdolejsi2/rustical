//! Shared configuration for the server.
//!
//! Please note: this is not a complete configuration of all the server settings,
//! but only the settings that are passed into the processing threads and their functions.

/// Configuration structure for the server passed along the threads.
#[derive(Clone)]
pub struct Config {
    pub(crate) file_dir: String,
    pub(crate) image_dir: String,
    pub(crate) debug: String,
}
