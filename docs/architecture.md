# Opticon — Architecture Reference

A DIY video-doorbell system. Two door units, self-hosted, no cloud. This doc is the
shared reference for building it and splitting the work.

## The three layers

| Layer | Language | Runs on | Job |
|---|---|---|---|
| **opticon-firmware** | C (ESP-IDF) | the ESP32-P4 doorbells | drives hardware, detection, originates all streams + events |
| **opticon-server** | Rust (Axum) | the N100 minipc | switchboard + brain + storage |
| **opticon-app** | HTML/CSS/JS (PWA) | each phone | the face: live view, talk, clips, notifications |

Key idea: **video flows peer-to-peer (doorbell ↔ phone) over WebRTC.** The server only
sets up the call and sends notifications — it never relays the video.

---

## The two contracts (pin these down first)

These are the interfaces where firmware and server meet. Agree on them early and the two
layers can be built independently.

### 1. Signaling protocol (WebRTC call setup)
A WebSocket message format both the firmware peer and the app peer speak, brokered by the
server. Roughly:
- `join`   — a peer announces itself (doorbell id, or user session)
- `offer` / `answer` — SDP exchange
- `ice`    — ICE candidates
- `bye`    — teardown

### 2. Event format (board → server)
JSON the board POSTs (or sends over the socket) on a trigger:
```json
{ "doorbell_id": "front-1", "type": "ring" | "motion", "ts": 1699999999, "snapshot": "<optional base64/jpeg>" }
```
The server decides what to do (notify, record, log). Keep this tiny and stable.

---

## opticon-firmware (C / ESP-IDF)

Start from Espressif's `esp-webrtc-solution` doorbell demo and adapt it.

**Responsibilities:** camera capture + H.264 encode; on-board person/motion detection;
ring-button GPIO; IR-cut + IR-LED day/night control; two-way audio (mic + speaker);
WebRTC peer; signaling to server; fire ring/motion events; optional SD-card clip write.

```
opticon-firmware/
├── CMakeLists.txt
├── sdkconfig.defaults          # P4 target, Ethernet enabled
├── partitions.csv
├── main/
│   ├── CMakeLists.txt
│   ├── main.c                  # init + main loop
│   ├── settings.h              # GPIO pins, sensor, server URLs, device id
│   ├── camera.c / .h           # OV5647 init + capture
│   ├── detection.c / .h        # person/motion detection
│   ├── ir_control.c / .h       # IR-cut filter + IR LEDs
│   ├── button.c / .h           # ring button (avoid GPIO35 w/ Ethernet)
│   ├── webrtc.c / .h           # WebRTC peer (from demo)
│   ├── signaling.c / .h        # WebSocket client to server
│   ├── events.c / .h           # ring/motion event send
│   └── recording.c / .h        # SD-card clips (optional)
├── components/                 # managed deps (esp-webrtc-solution)
└── README.md
```
Note: needs the physical board to test. Buildable/readable now, testable later.

---

## opticon-server (Rust / Axum)

**Responsibilities:** signaling WebSocket; accounts + sessions (4 fixed logins);
`/ring` + `/motion` endpoints; web-push; clip storage + retention; the database;
serve the app; coturn/TURN alongside for off-network.

```
opticon-server/
├── Cargo.toml
├── .env                        # DATABASE_URL, VAPID keys, TURN creds
├── migrations/                 # sqlx: users, doorbells, subscriptions, clips
├── src/
│   ├── main.rs                 # router + shared state
│   ├── config.rs
│   ├── db.rs                   # pool + queries
│   ├── models.rs               # User, Doorbell, Subscription, Clip
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── routes.rs           # login / logout
│   │   └── session.rs          # cookie/session middleware
│   ├── signaling.rs            # WebSocket handler
│   ├── events.rs               # /ring, /motion
│   ├── push.rs                 # web-push sender (VAPID)
│   ├── recordings.rs           # clip ingest, storage, retention
│   └── admin.rs                # add-doorbell (admin only)
├── static/                     # (optional) serve the app from here
└── README.md
```
Buildable entirely without hardware — this is the productive first move.

### Data model (minimal)
- `users` — 4 rows (username, argon2 hash)
- `doorbells` — 2 rows (id, name)
- `push_subscriptions` — endpoint + keys, linked to a user
- `clips` — doorbell id, timestamp, trigger type, file path

Everyone sees every doorbell — no per-user access table needed.

---

## opticon-app (PWA / HTML+CSS+JS)

**Responsibilities:** live view + two-way talk; recent-events/clip playback; login form;
service worker (push); PWA install (manifest + add-to-home-screen); permission flow.

```
opticon-app/
├── index.html                  # app shell
├── manifest.webmanifest        # name, icons, display: standalone
├── service-worker.js           # receives push, shows notification (REQUIRED for push)
├── css/
│   └── style.css
├── js/
│   ├── main.js                 # entry + view routing
│   ├── auth.js                 # login form -> server
│   ├── signaling.js            # WebSocket signaling client
│   ├── webrtc.js               # RTCPeerConnection: live view + talk
│   ├── clips.js                # recent events + playback
│   └── push.js                 # subscribe + permission
├── icons/                      # PWA icons
└── README.md
```
Served over HTTPS (required for service workers, push, and camera/mic access).

---

## Repo layout

A monorepo keeps the three layers and the shared contracts together:
```
opticon/
├── firmware/     # opticon-firmware
├── server/       # opticon-server
├── app/          # opticon-app
├── docs/         # this file, the blueprint diagram, the contracts
└── README.md
```

## Suggested build order
1. **Server**: Axum skeleton — signaling WS, accounts, `/ring` + `/motion`, schema.
2. **App**: PWA shell + webcam stand-in peer → prove live view + talk + push end to end.
3. **Recording + clips view** across server + app.
4. **Firmware**: adapt the demo once boards arrive — camera, button pin, IR control, point at server.

Steps 1–3 need no hardware.
