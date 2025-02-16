use crate::util::flush;
use std::io;
use std::sync::mpsc::Sender;

pub(crate) fn run(tx: Sender<String>) {
    println!("Supplier thread is running, please input <transformer> <parameter> (Ctrl+D or 'exit' to finish):");
    flush();

    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let input = buffer.trim_end().to_string();
                match input.as_str().trim() {
                    "" => {
                        continue;
                    }
                    // handle the control command explicitly (we would kill the transformer thread prematurely otherwise)
                    "exit" => {
                        break;
                    }
                    _ => {
                        if let Err(e) = tx.send(input) {
                            eprintln!("Error sending input: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    // finish the transformer thread
    tx.send("exit".to_string()).unwrap();

    println!("Exiting supplier thread");
    flush();
}
