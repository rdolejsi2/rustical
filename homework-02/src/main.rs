use slug::slugify;
use std::env;
use std::io;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a single argument for the transformation mode");
        return;
    }
    let mode: &str = &args[1];

    print!("Please enter input text for performing {}: ", mode);
    io::stdout().flush().unwrap(); // flush before read (to ensure the message is displayed)

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let result = match mode {
        "lowercase" => input.to_lowercase(),
        "uppercase" => input.to_uppercase(),
        "no_spaces" => input.replace(" ", ""),
        "slugify" => slugify(input),
        "kebab" => input.replace(" ", "-").to_lowercase(), // seems same as slugify
        "snake" => input.replace(" ", "_").to_lowercase(),
        "hex" =>
            input.chars()
                .map(|c| format!("{:02x}", c as u8))
                .collect::<String>(),
        _ => {
            println!("Invalid mode: {}", mode);
            std::process::exit(1);
        }
    };

    println!("Result: {}", result);
}
