use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer, session_store};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;


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

    let app = Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
        .route("/login", post(login))
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
