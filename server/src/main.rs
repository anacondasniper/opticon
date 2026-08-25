use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    extract::Request,
    middleware::Next,
    http::StatusCode,
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;
use tokio::sync::broadcast;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tower_http::services::ServeDir;


#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    db: SqlitePool,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct EventRequest {
    doorbell_id: String,
}

#[derive(Deserialize)]
struct PushSubscription {
    endpoint: String,
    keys: PushKeys,
}

#[derive(Deserialize)]
struct PushKeys {
    p256dh: String,
    auth: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let (tx, _rx) = broadcast::channel::<String>(100);

    let db = SqlitePool::connect(
        &std::env::var("DATABASE_URL").expect("DATABASE_URL not set"),
    ).await.expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations!");

    let state = AppState { tx, db };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(true);

    let protected = Router::new()
        .route("/ws", get(ws_handler))
        .route("/subscribe", post(subscribe))
        .route_layer(axum::middleware::from_fn(require_login));

    let app = Router::new()
        .route("/health", get(health))
        .route("/login", post(login))
        .route("/ring", post(ring))
        .route("/motion", post(motion))
        .merge(protected)
        .fallback_service(ServeDir::new("../app"))
        .layer(session_layer)
        .with_state(state);

    let config = RustlsConfig::from_pem_file(
        "10.0.0.41+2.pem", 
        "10.0.0.41+2-key.pem",
    ).await.unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Opticon listening on https://{}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn login(
    session: Session,
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse {
    let row = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, password_hash FROM users WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let (user_id, password_hash) = match row {
        Some(r) => r,
        None => return (StatusCode::UNAUTHORIZED, "invalid credentials").into_response(),
    };

    let parsed = match PasswordHash::new(&password_hash) {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "bad hash").into_response(),
    };

    if Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed)
        .is_ok()
    {
        session.insert("user_id", user_id).await.unwrap();
        (StatusCode::OK, "ok").into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "invalid credentials").into_response()
    }
}

async fn require_login(
    session: Session,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let user_id: Option<i64> = session.get("user_id").await.unwrap_or(None);

    match user_id {
        Some(_) => next.run(request).await,
        None => (StatusCode::UNAUTHORIZED, "not logged in").into_response(),
    }
}

async fn verify_doorbell(
    db: &SqlitePool,
    headers: &HeaderMap,
    doorbell_id: &str,
) -> bool {
    let token = match headers.get("x-device-token").and_then(|v| v.to_str().ok()) {
        Some(t) => t,
        None => return false,
    };

    let row = sqlx::query_as::<_, (String,)>(
        "SELECT token FROM doorbells WHERE id = ?",
    )
    .bind(doorbell_id)
    .fetch_optional(db)
    .await
    .unwrap_or(None);

    match row {
        Some((stored,)) => stored == token,
        None => false,
    }
}

async fn ring(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<EventRequest>,
) -> impl IntoResponse {
    if !verify_doorbell(&state.db, &headers, &body.doorbell_id).await {
        return (StatusCode::UNAUTHORIZED, "invalid device").into_response();
    }
    let event = format!(r#"{{"kind":"ring","doorbell_id":"{}"}}"#, body.doorbell_id);
    let _ = state.tx.send(event);
    println!("ring from {}", body.doorbell_id);
    (StatusCode::OK, "ok").into_response()
}

async fn motion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<EventRequest>,
) -> impl IntoResponse {
    if !verify_doorbell(&state.db, &headers, &body.doorbell_id).await {
        return (StatusCode::UNAUTHORIZED, "invalid device").into_response();
    }
    let event = format!(r#"{{"kind":"motion","doorbell_id":"{}"}}"#, body.doorbell_id);
    let _ = state.tx.send(event);
    println!("motion from {}", body.doorbell_id);
    (StatusCode::OK, "ok").into_response()
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();
    let tx = state.tx.clone();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task =  tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                let _ = tx.send(text.to_string());
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

async fn subscribe(
    session: Session,
    State(state): State<AppState>,
    Json(sub): Json<PushSubscription>,
) -> impl IntoResponse {
    let user_id: Option<i64> = session.get("user_id").await.unwrap_or(None);
    let user_id = match user_id {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "not logged in").into_response(),
    };

    let result = sqlx::query(
        "INSERT OR REPLACE INTO push_subscriptions (user_id, endpoint, p256dh, auth) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(&sub.endpoint)
    .bind(&sub.keys.p256dh)
    .bind(&sub.keys.auth)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (StatusCode::OK, "subscribed").into_response(),
        Err(e) => {
            eprintln!("subscribe error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}
