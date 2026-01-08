use crate::map::generate_map_svg;
use crate::utils::game_state::{SharedGameState, UnitOrder, WorkerAssignment};
use axum::{
  extract::State,
  response::IntoResponse,
  routing::{get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::services::ServeDir;

// Re-export map types
pub use crate::map::{MapData, ResourceInfo, UnitInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
  pub map_svg: String,
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub game_speed: i32,
  pub build_order: Vec<String>,
  pub build_order_index: usize,
  pub larva_responsibilities: HashMap<usize, usize>,
  pub unit_orders: HashMap<usize, UnitOrder>,
}

#[derive(Debug, Deserialize)]
pub struct GameSpeedCommand {
  pub command: String,
  pub value: i32,
}

async fn status_handler(State(game_state): State<SharedGameState>) -> Json<StatusUpdate> {
  let status = game_state.lock().unwrap();
  let map_svg = generate_map_svg(&status.map_data);

  Json(StatusUpdate {
    map_svg,
    worker_assignments: status.worker_assignments.clone(),
    game_speed: status.game_speed,
    build_order: status
      .build_order
      .iter()
      .map(|ut| format!("{:?}", ut))
      .collect(),
    build_order_index: status.build_order_index,
    larva_responsibilities: status.larva_responsibilities.clone(),
    unit_orders: status.unit_orders.clone(),
  })
}

async fn command_handler(
  State(game_state): State<SharedGameState>,
  Json(cmd): Json<GameSpeedCommand>,
) -> impl IntoResponse {
  if cmd.command == "set_game_speed" {
    if let Ok(mut status) = game_state.lock() {
      status.game_speed = cmd.value;
      println!("Game speed set to: {}", cmd.value);
    }
  }
  "OK"
}

pub async fn start_server(game_state: SharedGameState) {
  let web_dir = std::env::current_dir().unwrap().join("web");

  let app = Router::new()
    .route("/status", get(status_handler))
    .route("/command", post(command_handler))
    .nest_service("/", ServeDir::new(web_dir))
    .with_state(game_state);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3333")
    .await
    .unwrap();

  println!("Status server running on http://127.0.0.1:3333");
  axum::serve(listener, app).await.unwrap();
}
