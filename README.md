# Scoreboard Broker

A real-time scoreboard system built with Rust using WebSockets. The broker server manages the score state and broadcasts updates to all connected subscribers whenever the publisher increments a team's score.

## Architecture

```
score_board/Broker/README.md#L1-1
Publisher  ──(ws)──▶  Broker Server  ──(broadcast)──▶  Subscriber(s)
```

- **Broker** (`src/main.rs`) — Axum WebSocket server that holds the score state and fans out updates.
- **Publisher** (`examples/publisher.rs`) — CLI client that sends score increment commands to the broker.
- **Subscriber** (`examples/subscriber.rs`) — CLI client that listens for score updates and renders the scoreboard.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021, stable toolchain)

## How to Run

You need **three separate terminal windows**.

---

### Terminal 1 — Start the Broker Server

```score_board/Broker/README.md#L1-1
cd score_board/Broker
cargo run
```

The server will start and listen on `0.0.0.0:3000`.

```score_board/Broker/README.md#L1-1
╔══════════════════════════════════════╗
║   Scoreboard Broker running          ║
║   Publisher  → ws://0.0.0.0:3000/ws/publish  ║
║   Subscriber → ws://0.0.0.0:3000/ws/subscribe ║
╚══════════════════════════════════════╝
Server Started!!
```

---

### Terminal 2 — Start a Subscriber

```score_board/Broker/README.md#L1-1
cd score_board/Broker
cargo run --example subscriber
```

The subscriber will connect and immediately display the current score. It will re-render the scoreboard every time a score update is received.

```score_board/Broker/README.md#L1-1
╔══════════════════════════════════════╗
║        Scoreboard Subscriber         ║
╚══════════════════════════════════════╝

Connecting to ws://127.0.0.1:3000/ws/subscribe ...
Connected! Waiting for score updates...

  ╔═══════════════════════════════════╗
  ║          ** SCOREBOARD **         ║
  ╠═════════════════╦═════════════════╣
  ║     TEAM  A     ║     TEAM  B     ║
  ╠═════════════════╬═════════════════╣
  ║        0        ║        0        ║
  ╚═════════════════╩═════════════════╝
```

> You can open as many subscriber terminals as you like — all of them will receive live updates simultaneously.

---

### Terminal 3 — Start the Publisher

```score_board/Broker/README.md#L1-1
cd score_board/Broker
cargo run --example publisher
```

Use the interactive prompt to increment scores:

| Command | Action          |
|---------|-----------------|
| `a`     | +1 for Team A   |
| `b`     | +1 for Team B   |
| `q`     | Quit / disconnect |

```score_board/Broker/README.md#L1-1
╔══════════════════════════════════════╗
║        Scoreboard Publisher          ║
╚══════════════════════════════════════╝

Connected!

┌─────────────────────────────────────┐
│  Commands                           │
│  a  →  +1 for Team A               │
│  b  →  +1 for Team B               │
│  q  →  quit                         │
└─────────────────────────────────────┘

> a
  ➕ +1 for Team A
> b
  ➕ +1 for Team B
```

Every command you type will trigger an instant update on all connected subscriber windows.

---

## WebSocket Endpoints

| Endpoint              | Role       | Description                                  |
|-----------------------|------------|----------------------------------------------|
| `ws://127.0.0.1:3000/ws/publish`   | Publisher  | Send `{"team":"A"}` or `{"team":"B"}` to increment a score |
| `ws://127.0.0.1:3000/ws/subscribe` | Subscriber | Receive JSON score updates `{"team_a":N,"team_b":N}` |

## Dependencies

| Crate               | Purpose                              |
|---------------------|--------------------------------------|
| `axum`              | Web framework with WebSocket support |
| `tokio`             | Async runtime                        |
| `serde` / `serde_json` | JSON serialisation / deserialisation |
| `tracing`           | Structured logging                   |
| `tokio-tungstenite` | WebSocket client (examples only)     |
| `futures-util`      | Stream/sink utilities (examples only)|