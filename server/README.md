# opticon-server

Rust + Axum, runs on the N100 minipc. The switchboard, brain, and storage.

**Does:** signaling WebSocket (WebRTC setup), accounts + sessions (4 fixed
logins), `/ring` + `/motion` event endpoints, web-push (VAPID), clip storage +
retention, the database (users, doorbells, subscriptions, clips), serves the app.
coturn/TURN runs alongside for off-network viewing.

**Start here** — buildable entirely without hardware.

```sh
cargo run        # boots the server
```

First target: a `/health` route that returns ok, then the `/ws` signaling
handler that relays messages between two clients.
