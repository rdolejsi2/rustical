mod client;
mod server;
mod util;

use clap::{Arg, Command};

const HOST_DEFAULT: &'static str = "localhost";
const PORT_DEFAULT: &'static str = "11111";

const FILE_DIRECTORY: &'static str = "files";
const IMAGE_DIRECTORY: &'static str = "images";

fn main() {
    #[rustfmt::skip]
    let matches = Command::new("client-server-file-sender")
        .arg(Arg::new("mode").required(true).index(1).value_parser(["server", "client"]))
        .arg(Arg::new("host").short('H').long("host").default_value(HOST_DEFAULT))
        .arg(Arg::new("port").short('p').long("port").default_value(PORT_DEFAULT))
        .get_matches();

    let mode = matches.get_one::<String>("mode").unwrap();
    let host = matches.get_one::<String>("host").unwrap();
    let port = matches.get_one::<String>("port").unwrap();
    let address = format!("{}:{}", host, port);

    match mode.as_str() {
        "server" => {
            server::run(address);
        }
        "client" => {
            client::run(address);
        }
        _ => {
            eprintln!("Invalid mode: {}. Use 'server' or 'client'.", mode);
            std::process::exit(1);
        }
    }
}
