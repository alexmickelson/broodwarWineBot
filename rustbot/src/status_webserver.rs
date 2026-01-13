use crate::utils::game_state::{DebugFlag, SharedGameState, WorkerAssignment};
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
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::services::ServeDir;

pub async fn start_server(game_state: SharedGameState, callbacks: SharedHttpStatusCallbacks) {
  let web_dir = std::env::current_dir().unwrap().join("web");

  let combined_state = (game_state, callbacks);

  // Configure CORS to allow only localhost and 127.0.0.1 origins
  let cors = CorsLayer::new()
    .allow_origin(AllowOrigin::predicate(
      |origin: &axum::http::HeaderValue, _| {
        origin
          .to_str()
          .map(|s| {
            s.starts_with("http://localhost")
              || s.starts_with("https://localhost")
              || s.starts_with("http://127.0.0.1")
              || s.starts_with("https://127.0.0.1")
          })
          .unwrap_or(false)
      },
    ))
    .allow_methods(Any)
    .allow_headers(Any);

  let app = Router::new()
    .route("/command", post(command_handler))
    .route("/worker-status", get(worker_status_handler))
    .route("/unit-orders", get(unit_orders_handler))
    .route("/military-assignments", get(military_assignments_handler))
    .route("/larvae", get(larvae_handler))
    .route("/build-order", get(build_order_handler))
    .route("/map", get(map_handler))
    .route("/game-speed", get(game_speed_handler))
    .route("/debug-flags", get(debug_flags_handler))
    .route("/debug-flags", post(update_debug_flags_handler))
    .nest_service("/", ServeDir::new(web_dir))
    .layer(cors)
    .with_state(combined_state);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3333")
    .await
    .unwrap();

  println!("Status server running on http://127.0.0.1:3333");
  axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
pub struct GameSpeedCommand {
  pub command: String,
  pub value: i32,
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

  let error_return = WorkerStatusSnapshot {
    worker_assignments: HashMap::new(),
    frame_count: -1,
  };

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(error_return);
  }

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(error_return),
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitOrder {
  pub unit_id: usize,
  pub unit_type: String,
  pub order_name: String,
  pub target_id: Option<usize>,
  pub target_type: Option<String>,
  pub current_position: (i32, i32),
  pub target_position: Option<(i32, i32)>,
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
    move |game: &rsbwapi::Game, _state: &crate::utils::game_state::GameState| {
      let mut unit_orders = HashMap::new();

      if let Some(player) = game.self_() {
        let my_units: Vec<_> = player.get_units().into_iter().collect();

        for unit in my_units {
          let unit_id = unit.get_id();
          let current_pos = unit.get_position();
          let order = unit.get_order();

          let target_id = unit.get_order_target().map(|t| t.get_id());
          let target_type = unit
            .get_order_target()
            .map(|t| format!("{:?}", t.get_type()));
          let target_position = unit.get_target_position().map(|p| (p.x, p.y));

          unit_orders.insert(
            unit_id,
            UnitOrder {
              unit_id,
              unit_type: format!("{:?}", unit.get_type()),
              order_name: format!("{:?}", order),
              target_id,
              target_type,
              current_position: (current_pos.x, current_pos.y),
              target_position,
            },
          );
        }
      }

      let snapshot = UnitOrdersSnapshot {
        unit_orders,
        frame_count: game.get_frame_count(),
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

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(UnitOrdersSnapshot {
      unit_orders: HashMap::new(),
      frame_count: -1,
    }),
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct LarvaeSnapshot {
  pub larva_responsibilities: HashMap<usize, usize>,
  pub frame_count: i32,
}

async fn larvae_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |_game: &rsbwapi::Game, state: &crate::utils::game_state::GameState| {
      let snapshot = LarvaeSnapshot {
        larva_responsibilities: state.larva_responsibilities.clone(),
        frame_count: _game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(LarvaeSnapshot {
      larva_responsibilities: HashMap::new(),
      frame_count: -1,
    });
  }

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(LarvaeSnapshot {
      larva_responsibilities: HashMap::new(),
      frame_count: -1,
    }),
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct MilitaryUnitInfo {
  pub unit_id: usize,
  #[serde(rename = "unitType")]
  pub unit_type: String,
  pub order: String,
  pub order_target_position: Option<(i32, i32)>,
  pub current_position: (i32, i32),
}

#[derive(Clone, Debug, Serialize)]
pub struct SquadData {
  pub name: String,
  pub units: Vec<MilitaryUnitInfo>,
  pub target_position: Option<(i32, i32)>,
  pub target_unit: Option<usize>,
  pub target_path: Option<Vec<(i32, i32)>>,
  pub target_path_current_index: Option<usize>,
  pub target_path_goal_index: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MilitaryAssignmentsSnapshot {
  pub squads: Vec<SquadData>,
  pub frame_count: i32,
}

async fn military_assignments_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |game: &rsbwapi::Game, state: &crate::utils::game_state::GameState| {
      let squads: Vec<SquadData> = state
        .military_squads
        .iter()
        .map(|squad| {
          let units: Vec<MilitaryUnitInfo> = squad
            .assigned_unit_ids
            .iter()
            .filter_map(|unit_id| {
              game.get_unit(*unit_id).map(|unit| {
                let order = unit.get_order();
                let order_target_position = unit.get_order_target_position().map(|p| (p.x, p.y));
                let current_pos = unit.get_position();

                MilitaryUnitInfo {
                  unit_id: *unit_id,
                  unit_type: format!("{:?}", unit.get_type()),
                  order: format!("{:?}", order),
                  order_target_position,
                  current_position: (current_pos.x, current_pos.y),
                }
              })
            })
            .collect();

          SquadData {
            name: squad.name.clone(),
            units,
            target_position: squad.target_position,
            target_unit: squad.target_unit,
            target_path: squad.target_path.clone(),
            target_path_current_index: squad.target_path_current_index,
            target_path_goal_index: squad.target_path_goal_index,
          }
        })
        .collect();

      let snapshot = MilitaryAssignmentsSnapshot {
        squads,
        frame_count: game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(MilitaryAssignmentsSnapshot {
      squads: Vec::new(),
      frame_count: -1,
    });
  }

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(MilitaryAssignmentsSnapshot {
      squads: Vec::new(),
      frame_count: -1,
    }),
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildOrderSnapshot {
  pub build_order: Vec<String>,
  pub build_order_index: usize,
  pub frame_count: i32,
}

async fn build_order_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |_game: &rsbwapi::Game, state: &crate::utils::game_state::GameState| {
      let snapshot = BuildOrderSnapshot {
        build_order: state
          .build_order
          .iter()
          .map(|ut| format!("{:?}", ut))
          .collect(),
        build_order_index: state.build_order_index,
        frame_count: _game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(BuildOrderSnapshot {
      build_order: Vec::new(),
      build_order_index: 0,
      frame_count: -1,
    });
  }

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(BuildOrderSnapshot {
      build_order: Vec::new(),
      build_order_index: 0,
      frame_count: -1,
    }),
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct MapSnapshot {
  pub map_data: crate::map::MapData,
  pub frame_count: i32,
}

async fn map_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |game: &rsbwapi::Game, _state: &crate::utils::game_state::GameState| {
      let map_data = crate::map::collect_map_data(game);
      let snapshot = MapSnapshot {
        map_data,
        frame_count: game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(MapSnapshot {
      map_data: crate::map::MapData::default(),
      frame_count: -1,
    });
  }

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(MapSnapshot {
      map_data: crate::map::MapData::default(),
      frame_count: -1,
    }),
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct GameSpeedSnapshot {
  pub game_speed: i32,
  pub frame_count: i32,
}

async fn game_speed_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |_game: &rsbwapi::Game, state: &crate::utils::game_state::GameState| {
      let snapshot = GameSpeedSnapshot {
        game_speed: state.game_speed,
        frame_count: _game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(GameSpeedSnapshot {
      game_speed: 0,
      frame_count: -1,
    });
  }

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(GameSpeedSnapshot {
      game_speed: 0,
      frame_count: -1,
    }),
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct DebugFlagsSnapshot {
  pub debug_flags: std::collections::HashSet<DebugFlag>,
  pub frame_count: i32,
}

async fn debug_flags_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |_game: &rsbwapi::Game, state: &crate::utils::game_state::GameState| {
      let snapshot = DebugFlagsSnapshot {
        debug_flags: state.debug_flags.clone(),
        frame_count: _game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  if let Ok(mut callbacks_lock) = callbacks.lock() {
    callbacks_lock.add_callback(callback);
  } else {
    return Json(DebugFlagsSnapshot {
      debug_flags: std::collections::HashSet::new(),
      frame_count: -1,
    });
  }

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(DebugFlagsSnapshot {
      debug_flags: std::collections::HashSet::new(),
      frame_count: -1,
    }),
  }
}

#[derive(Debug, Deserialize)]
pub struct UpdateDebugFlagsRequest {
  pub debug_flags: std::collections::HashSet<DebugFlag>,
}

async fn update_debug_flags_handler(
  State((game_state, _)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
  Json(req): Json<UpdateDebugFlagsRequest>,
) -> impl IntoResponse {
  if let Ok(mut state) = game_state.lock() {
    state.debug_flags = req.debug_flags;
    println!("Debug flags updated");
    "OK"
  } else {
    "Error updating debug flags"
  }
}
