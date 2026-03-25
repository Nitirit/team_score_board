use axum::{
    extract::ws::{Message, WebSocket},
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::info;

// ── Shared state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct Score {
    team_a: u32,
    team_b: u32,
}

/// Message the publisher sends to increment a team's score.
#[derive(Debug, Deserialize)]
struct ScoreUpdate {
    /// Expected value: "A" or "B"
    team: String,
}

struct AppState {
    score: Mutex<Score>,
    /// Every score change is broadcast as a JSON string to all subscribers.
    tx: broadcast::Sender<String>,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// WebSocket endpoint for the publisher.
/// URL: ws://127.0.0.1:3000/ws/publish
async fn publish_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_publisher(socket, state))
}

/// Reads score-update messages from the publisher, mutates shared score,
/// then broadcasts the new score to all connected subscribers.
async fn handle_publisher(mut socket: WebSocket, state: Arc<AppState>) {
    info!("Publisher connected");

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                println!("[BROKER] Received from publisher: {text}");
                let Ok(update) = serde_json::from_str::<ScoreUpdate>(&text) else {
                    tracing::warn!("Publisher sent invalid JSON: {text}");
                    continue;
                };

                // Mutate score and serialise inside the lock so the broadcast
                // always carries the value that was actually committed.
                let score_json = {
                    let mut score = state.score.lock().await;
                    match update.team.to_uppercase().as_str() {
                        "A" => score.team_a += 1,
                        "B" => score.team_b += 1,
                        other => {
                            tracing::warn!("Unknown team: {other}");
                            continue;
                        }
                    }
                    serde_json::to_string(&*score).expect("Score serialisation is infallible")
                };

                info!("Score updated → {score_json}");

                // It is OK if there are no subscribers yet; ignore the error.
                let _ = state.tx.send(score_json);
            }

            // Respond to pings so the connection stays alive.
            Message::Ping(payload) => {
                let _ = socket.send(Message::Pong(payload)).await;
            }

            // Publisher requested a clean close.
            Message::Close(_) => {
                info!("Publisher disconnected gracefully");
                break;
            }

            _ => {}
        }
    }

    info!("Publisher connection closed");
}

/// WebSocket endpoint for subscribers.
/// URL: ws://127.0.0.1:3000/ws/subscribe
async fn subscribe_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_subscriber(socket, state))
}

/// Subscribes to the broadcast channel and forwards every score update to the
/// connected subscriber. The current score is sent immediately on connection so
/// the client always has a valid starting state.
async fn handle_subscriber(mut socket: WebSocket, state: Arc<AppState>) {
    info!("Subscriber connected");

    // Subscribe *before* reading the current score to avoid missing an update
    // that arrives between the two operations.
    let mut rx = state.tx.subscribe();

    // Send the current score right away.
    let current_score = {
        let score = state.score.lock().await;
        serde_json::to_string(&*score).expect("Score serialisation is infallible")
    };

    if socket.send(Message::Text(current_score)).await.is_err() {
        info!("Subscriber disconnected before initial score could be sent");
        return;
    }

    // Forward every subsequent broadcast until the client disconnects.
    loop {
        match rx.recv().await {
            Ok(score_json) => {
                println!("[BROKER] Sending to subscriber: {score_json}");
                if socket.send(Message::Text(score_json)).await.is_err() {
                    info!("Subscriber disconnected");
                    break;
                }
            }

            // The channel fell behind; skip the lost messages and keep going.
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("Subscriber lagged, skipped {n} messages");
            }

            // Sender was dropped — server is shutting down.
            Err(broadcast::error::RecvError::Closed) => {
                info!("Broadcast channel closed; dropping subscriber");
                break;
            }
        }
    }

    info!("Subscriber connection closed");
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "broker=info,tower_http=debug".into()),
        )
        .init();

    // Broadcast channel capacity: 256 messages in flight.
    let (tx, _) = broadcast::channel::<String>(256);

    let state = Arc::new(AppState {
        score: Mutex::new(Score {
            team_a: 0,
            team_b: 0,
        }),
        tx,
    });

    let app = Router::new()
        .route("/ws/publish", get(publish_handler))
        .route("/ws/subscribe", get(subscribe_handler))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    info!("╔══════════════════════════════════════╗");
    info!("║   Scoreboard Broker running          ║");
    info!("║   Publisher  → ws://{addr}/ws/publish  ║");
    info!("║   Subscriber → ws://{addr}/ws/subscribe ║");
    info!("╚══════════════════════════════════════╝");
    println!("Server Started!!");
    axum::serve(listener, app).await.expect("Server error");
}
