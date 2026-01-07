use crate::map::generate_map_svg;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use tower_http::services::ServeDir;

// Re-export map types
pub use crate::map::{MapData, ResourceInfo, UnitInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerStatus {
    pub total: usize,
    pub gathering: usize,
    pub idle: usize,
    pub building: usize,
}

impl Default for WorkerStatus {
    fn default() -> Self {
        Self {
            total: 0,
            gathering: 0,
            idle: 0,
            building: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub worker_status: WorkerStatus,
    pub map_svg: String,
}

#[derive(Clone, Debug, Default)]
pub struct GameStatus {
    pub worker_status: WorkerStatus,
    pub map_data: MapData,
}

pub type SharedStatus = Arc<Mutex<GameStatus>>;

pub fn create_shared_status() -> SharedStatus {
    Arc::new(Mutex::new(GameStatus::default()))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedStatus>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: SharedStatus) {
    let (mut sender, mut receiver) = socket.split();

    let send_task = tokio::spawn(async move {
        let mut update_interval = interval(Duration::from_millis(500));

        loop {
            update_interval.tick().await;

            let status_update = {
                let status = state.lock().unwrap();
                let map_svg = generate_map_svg(&status.map_data);

                StatusUpdate {
                    worker_status: status.worker_status.clone(),
                    map_svg,
                }
            };

            match serde_json::to_string(&status_update) {
                Ok(json) => {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error serializing status update: {}", e);
                    break;
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

pub async fn start_server(status: SharedStatus) {
    let web_dir = std::env::current_dir().unwrap().join("web");

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .nest_service("/", ServeDir::new(web_dir))
        .with_state(status);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Status server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
