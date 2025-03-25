mod supplier;
mod transformer;
mod util;

use std::sync::mpsc;
use std::thread;
use transformer::Mode;
use util::flush;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode: Mode = if args.len() > 1 {
        Mode::CLI
    } else {
        println!("Welcome to streaming transformer!");
        flush();
        Mode::Streaming
    };

    let (tx_transformer, rx_transformer) = mpsc::channel();
    // additional channel to main to indicate transformer is ready (i.e., its welcome message has been printed)
    let (tx_main, rx_main) = mpsc::channel();

    let transformer_mode = mode.clone();
    let transformer_thread = thread::Builder::new()
        .name("transformer".to_string())
        .spawn(move || {
            transformer::run(transformer_mode, rx_transformer, tx_main);
        });

    // wait for transformer thread to be ready (i.e., it finished printing its welcome message)
    rx_main.recv().unwrap();

    // interactive streaming mode when no parameters are provided
    match mode {
        Mode::CLI => {
            let input = args[1..].join(" ");
            tx_transformer.send(input).unwrap();
            tx_transformer.send("exit".to_string()).unwrap();
        }
        Mode::Streaming => {
            let supplier_thread =
                thread::Builder::new()
                    .name("supplier".to_string())
                    .spawn(move || {
                        supplier::run(tx_transformer);
                    });

            supplier_thread.unwrap().join().unwrap();
        }
    }

    transformer_thread.unwrap().join().unwrap();
}
