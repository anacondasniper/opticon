# Opticon

A self-hosted DIY video doorbell. Two door units, no cloud — video streams
peer-to-peer from the doorbell to your phone; a small server on a minipc handles
setup, accounts, notifications, and recordings.

## Layers

| Dir | Language | Runs on | Job |
|---|---|---|---|
| [`firmware/`](firmware/) | C (ESP-IDF) | ESP32-P4 doorbells | drives hardware, detection, originates streams + events |
| [`server/`](server/) | Rust (Axum) | N100 minipc | signaling, accounts, events, push, recordings, DB |
| [`app/`](app/) | HTML/CSS/JS (PWA) | phones | live view, two-way talk, clips, notifications |

**Core idea:** the video never flows through the server. WebRTC connects the
doorbell and the phone directly; the server only brokers the handshake and sends
push notifications.

## Docs

- [`docs/architecture.md`](docs/architecture.md) — full architecture, responsibilities, and the two interface contracts
- [`docs/blueprint.mermaid`](docs/blueprint.mermaid) — system data-flow diagram

## Features

- PoE power (always-on, event-driven streaming — no sleep)
- Motion (person) + ring detection
- Event-clip recording on motion/ring (stored on the minipc, SD as optional backup)
- Two-way live audio (mic + speaker)
- IR-CUT night-vision camera
- Message-taking — planned v2 (visitor leaves a message when nobody answers)

## Build order

1. **server** — Axum skeleton: health route → signaling WebSocket → accounts → `/ring` + `/motion` → schema
2. **app** — PWA shell + webcam stand-in to prove live view + talk + push end to end
3. recording + clips view across server + app
4. **firmware** — adapt Espressif's doorbell demo once the boards arrive

Steps 1–3 need no hardware.

## Getting started

```sh
# server (start here)
cd server
cargo run        # boots the Axum server

# app (served over HTTPS or localhost)
cd app
# open index.html via a local static server
```
