use futures_util::StreamExt;
use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const SERVER_URL: &str = "ws://127.0.0.1:3000/ws/subscribe";

#[derive(Debug, Deserialize)]
struct Score {
    team_a: u32,
    team_b: u32,
}

fn render_scoreboard(score: &Score) {
    println!();
    println!("  ╔═══════════════════════════════════╗");
    println!("  ║          ** SCOREBOARD **         ║");
    println!("  ╠═════════════════╦═════════════════╣");
    println!("  ║     TEAM  A     ║     TEAM  B     ║");
    println!("  ╠═════════════════╬═════════════════╣");
    println!("  ║{:^17}║{:^17}║", score.team_a, score.team_b);
    println!("  ╚═════════════════╩═════════════════╝");
    println!();
}

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║        Scoreboard Subscriber         ║");
    println!("╚══════════════════════════════════════╝");
    println!();
    println!("Connecting to {} ...", SERVER_URL);

    let (ws_stream, _) = connect_async(SERVER_URL)
        .await
        .expect("Failed to connect to the broker. Is the server running?");

    let (_write, mut read) = ws_stream.split();

    println!("Connected! Waiting for score updates...");

    while let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(Message::Text(text)) => match serde_json::from_str::<Score>(&text) {
                Ok(score) => render_scoreboard(&score),
                Err(e) => eprintln!("  ⚠ Failed to parse score update: {} (raw: {})", e, text),
            },

            Ok(Message::Ping(_)) => {
                // tungstenite handles pong automatically
            }

            Ok(Message::Close(frame)) => {
                if let Some(f) = frame {
                    println!("Server closed connection: {} {}", f.code, f.reason);
                } else {
                    println!("Server closed connection.");
                }
                break;
            }

            Ok(_) => {}

            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    println!("Disconnected. Goodbye!");
}
