//! A simple CLI-based UI for testing the core functionality.
//!
//! Usage: cargo run -p slsk-mock

use slsk_core::{config::Config, start, Command};

use std::io::{self, Write};

fn main() {
    println!("=== Menthol CLI Mock UI ===");
    println!("Type 'connect <username> <password>' to connect");
    println!("Type 'search <query>' to search");
    println!("Type 'quit' to exit\n");

    // Default config
    let config = Config {
        username: String::new(),
        password: String::new(),
        port: 2234,
        host: "server.slsknet.org".to_string(),
    };

    // Start the core in a background thread
    let mut core = start(config);

    // Main input loop
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        match parts[0] {
            "quit" | "exit" => {
                core.send(Command::Disconnect);
                break;
            }
            "connect" => {
                if parts.len() < 2 {
                    println!("Usage: connect <username> <password>");
                    continue;
                }
                let creds: Vec<&str> = parts[1].splitn(2, ' ').collect();
                if creds.len() < 2 {
                    println!("Usage: connect <username> <password>");
                    continue;
                }
                core.send(Command::Connect {
                    username: creds[0].to_string(),
                    password: creds[1].to_string(),
                });
            }
            "search" => {
                if parts.len() < 2 {
                    println!("Usage: search <query>");
                    continue;
                }
                let token = rand_u32();
                core.send(Command::Search {
                    query: parts[1].to_string(),
                    token,
                });
            }
            "disconnect" => {
                core.send(Command::Disconnect);
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
            }
        }
    }

    println!("Goodbye!");
}

fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    nanos
}
