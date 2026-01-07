use crate::map::generate_map_svg;
use crate::utils::game_status::{SharedStatus, WorkerAssignment};
use axum::extract::ws::{Message, WebSocket};
use axum::{
  extract::{State, WebSocketUpgrade},
  response::IntoResponse,
  routing::get,
  Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tower_http::services::ServeDir;

// Re-export map types
pub use crate::map::{MapData, ResourceInfo, UnitInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
  pub map_svg: String,
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub game_speed: i32,
  pub build_order: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GameSpeedCommand {
  pub command: String,
  pub value: i32,
}

async fn websocket_handler(
  ws: WebSocketUpgrade,
  State(state): State<SharedStatus>,
) -> impl IntoResponse {
  ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: SharedStatus) {
  let (mut sender, mut receiver) = socket.split();

  let state_clone = state.clone();
  let send_task = tokio::spawn(async move {
    let mut update_interval = interval(Duration::from_millis(500));

    loop {
      update_interval.tick().await;

      let status_update = {
        let status = state_clone.lock().unwrap();
        let map_svg = generate_map_svg(&status.map_data);

        StatusUpdate {
          map_svg,
          worker_assignments: status.worker_assignments.clone(),
          game_speed: status.game_speed,
          build_order: status
            .build_order
            .iter()
            .map(|ut| format!("{:?}", ut))
            .collect(),
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
      match msg {
        Message::Text(text) => {
          if let Ok(cmd) = serde_json::from_str::<GameSpeedCommand>(&text) {
            if cmd.command == "set_game_speed" {
              if let Ok(mut status) = state.lock() {
                status.game_speed = cmd.value;
                println!("Game speed set to: {}", cmd.value);
              }
            }
          }
        }
        Message::Close(_) => break,
        _ => {}
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

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3333")
    .await
    .unwrap();

  println!("Status server running on http://127.0.0.1:3333");
  axum::serve(listener, app).await.unwrap();
}
