use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use tower_http::services::ServeDir;
use tokio::sync::broadcast;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    db: SqlitePool,
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

    let app = Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("../app"))
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
