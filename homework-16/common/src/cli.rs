//! Common command-line parsing utilities for any and all used parameters across client and server.
//!
//! This module provides functionality to parse command-line arguments
//! and extract values for different parameters such as host and port.

use clap::{Arg, ArgMatches, Command};
use std::error::Error;

pub(crate) const HOST_DEFAULT: &'static str = "localhost";
pub(crate) const PORT_DEFAULT: &'static str = "11111";
const FILE_DIRECTORY_DEFAULT: &'static str = "files";
const IMAGE_DIRECTORY_DEFAULT: &'static str = "images";

/// The enum defines all the possible parameters that can be used on the command line.
/// Client and Server share the parameter list, but choose which parameters they accept on CLI.
pub enum CliArg {
    Host,
    Port,
    FileDir,
    ImageDir,
    Debug,
}

/// The implementation of the [CliArg] enum provides methods to convert the enum variants
/// to [clap::Arg] instances and to retrieve their values from the command line arguments.
///
/// Any defaults of the arguments are defined here as well, thus effectively shared across
/// client and server.
impl CliArg {
    fn as_arg(&self) -> Arg {
        match self {
            CliArg::Host => Arg::new("host")
                .short('H')
                .long("host")
                .default_value(HOST_DEFAULT)
                .help("Sets the server host"),
            CliArg::Port => Arg::new("port")
                .short('p')
                .long("port")
                .default_value(PORT_DEFAULT)
                .help("Sets the server port"),
            CliArg::FileDir => Arg::new("file-dir")
                .short('f')
                .long("file-dir")
                .default_value(FILE_DIRECTORY_DEFAULT)
                .help("Sets the file directory"),
            CliArg::ImageDir => Arg::new("image-dir")
                .short('i')
                .long("image-dir")
                .default_value(IMAGE_DIRECTORY_DEFAULT)
                .help("Sets the image directory"),
            CliArg::Debug => Arg::new("debug")
                .short('d')
                .long("debug")
                .default_value("false")
                .help("Enables debug mode")
                .action(clap::ArgAction::Set)
                .default_missing_value("true")
                .num_args(0),
        }
    }

    /// Retrieves the value of the command line argument.
    /// The result is generalized into a `String` type to allow uniform handling of all parameters.
    fn get_value<'a>(&self, matches: &'a ArgMatches) -> Result<String, Box<dyn Error>> {
        let result: Option<&String> = match self {
            CliArg::Host => matches.get_one::<String>("host"),
            CliArg::Port => matches.get_one::<String>("port"),
            CliArg::FileDir => matches.get_one::<String>("file-dir"),
            CliArg::ImageDir => matches.get_one::<String>("image-dir"),
            CliArg::Debug => matches.get_one::<String>("debug"),
        };
        result
            .map(|s| Ok(s.clone()))
            .unwrap_or_else(|| Err("Parameter not found".into()))
    }
}

/// Parses command line arguments for the given application name and argument list.
///
/// This allows for a very concise code interaction from the caller (client, server) perspective,
/// which just receive back the vector of corresponding values (be it defaults or user-specified).
pub fn parse_args(app: &'static str, args: &[CliArg]) -> Result<Vec<String>, Box<dyn Error>> {
    let mut command = Command::new(app);
    for arg in args {
        command = command.arg(arg.as_arg());
    }
    let matches = command.get_matches();

    let mut values = Vec::new();
    for arg in args {
        values.push(arg.get_value(&matches)?);
    }
    Ok(values)
}
