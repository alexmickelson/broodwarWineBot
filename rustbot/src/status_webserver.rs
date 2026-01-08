use crate::map::generate_map_svg;
pub use crate::map::{MapData, ResourceInfo, UnitInfo};
use crate::utils::game_state::{SharedGameState, UnitOrder, WorkerAssignment};
use crate::utils::http_status_callbacks::SharedHttpStatusCallbacks;
use axum::{
  extract::State,
  response::IntoResponse,
  routing::{get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;

pub async fn start_server(game_state: SharedGameState, callbacks: SharedHttpStatusCallbacks) {
  let web_dir = std::env::current_dir().unwrap().join("web");

  let combined_state = (game_state, callbacks);

  let app = Router::new()
    .route("/status", get(status_handler))
    .route("/command", post(command_handler))
    .route("/worker-status", get(worker_status_handler))
    .route("/unit-orders", get(unit_orders_handler))
    .nest_service("/", ServeDir::new(web_dir))
    .with_state(combined_state);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3333")
    .await
    .unwrap();

  println!("Status server running on http://127.0.0.1:3333");
  axum::serve(listener, app).await.unwrap();
}

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

async fn status_handler(
  State((game_state, _)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> Json<StatusUpdate> {
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
  State((game_state, _)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
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

#[derive(Clone, Debug, Serialize)]
pub struct WorkerStatusSnapshot {
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub frame_count: i32,
}

async fn worker_status_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |_game: &rsbwapi::Game, state: &crate::utils::game_state::GameState| {
      let snapshot = WorkerStatusSnapshot {
        worker_assignments: state.worker_assignments.clone(),
        frame_count: _game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(WorkerStatusSnapshot {
      worker_assignments: HashMap::new(),
      frame_count: -1,
    });
  }

  // Wait for the callback to be invoked from the game thread
  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(WorkerStatusSnapshot {
      worker_assignments: HashMap::new(),
      frame_count: -1,
    }),
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct UnitOrdersSnapshot {
  pub unit_orders: HashMap<usize, UnitOrder>,
  pub frame_count: i32,
}

async fn unit_orders_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |_game: &rsbwapi::Game, state: &crate::utils::game_state::GameState| {
      let snapshot = UnitOrdersSnapshot {
        unit_orders: state.unit_orders.clone(),
        frame_count: _game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(UnitOrdersSnapshot {
      unit_orders: HashMap::new(),
      frame_count: -1,
    });
  }

  // Wait for the callback to be invoked from the game thread
  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(UnitOrdersSnapshot {
      unit_orders: HashMap::new(),
      frame_count: -1,
    }),
  }
}
