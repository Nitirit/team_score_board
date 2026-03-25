use futures_util::SinkExt;
use std::io::{self, Write};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const SERVER_URL: &str = "ws://127.0.0.1:3000/ws/publish";

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║        Scoreboard Publisher          ║");
    println!("╚══════════════════════════════════════╝");
    println!();
    // println!("Connecting to {} ...", SERVER_URL);

    let (ws_stream, _) = connect_async(SERVER_URL)
        .await
        .expect("Failed to connect to the broker. Is the server running?");

    let (mut write, _read) = futures_util::StreamExt::split(ws_stream);

    println!("Connected!\n");
    println!("┌─────────────────────────────────────┐");
    println!("│  Commands                           │");
    println!("│  a  →  +1 for Team A               │");
    println!("│  b  →  +1 for Team B               │");
    println!("│  q  →  quit                         │");
    println!("└─────────────────────────────────────┘");
    println!();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read from stdin");

        let cmd = input.trim().to_lowercase();

        let payload = match cmd.as_str() {
            "a" => {
                println!("  ➕ +1 for Team A");
                r#"{"team":"A"}"#.to_string()
            }
            "b" => {
                println!("  ➕ +1 for Team B");
                r#"{"team":"B"}"#.to_string()
            }
            "q" | "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            "" => continue,
            other => {
                println!("  ⚠ Unknown command '{}'. Use 'a', 'b', or 'q'.", other);
                continue;
            }
        };

        if let Err(e) = write.send(Message::Text(payload)).await {
            eprintln!("Failed to send message: {}. Connection may be lost.", e);
            break;
        }
    }
}
