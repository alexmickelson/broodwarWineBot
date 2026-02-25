use crate::utils::build_orders::build_order_item::BuildOrderItem;
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
    .route(
      "/military-assignments/update-target-percentage",
      post(update_squad_target_percentage_handler),
    )
    .route(
      "/military-assignments/update-target-player",
      post(update_squad_target_player_handler),
    )
    .route("/start-locations", get(start_locations_handler))
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
  pub build_order: Vec<BuildOrderItem>,
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
        build_order: state.build_order.clone(),
        frame_count: _game.get_frame_count(),
      };
      let _ = tx.send(snapshot);
    },
  );

  let error_return = WorkerStatusSnapshot {
    worker_assignments: HashMap::new(),
    build_order: Vec::new(),
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
  pub assignment_details: HashMap<usize, String>,
  pub frame_count: i32,
}

async fn larvae_handler(
  State((_, callbacks)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  let (tx, rx) = oneshot::channel();

  let callback = Box::new(
    move |_game: &rsbwapi::Game, state: &crate::utils::game_state::GameState| {
      let mut assignment_details = HashMap::new();
      for (larva_id, build_order_idx) in &state.larva_responsibilities {
        if let Some(item) = state.build_order.get(*build_order_idx) {
          let detail = match item {
            BuildOrderItem::Unit {
              unit_type,
              base_index,
            } => match base_index {
              Some(idx) => format!("{:?} @base{}", unit_type, idx),
              None => format!("{:?}", unit_type),
            },
            BuildOrderItem::Upgrade(upgrade_type) => {
              format!("{:?}", upgrade_type)
            }
            BuildOrderItem::Squad { name, role, status } => {
              format!("Squad({}, {:?}, {:?})", name, role, status)
            }
          };
          assignment_details.insert(*larva_id, detail);
        }
      }
      let snapshot = LarvaeSnapshot {
        larva_responsibilities: state.larva_responsibilities.clone(),
        assignment_details,
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
      assignment_details: HashMap::new(),
      frame_count: -1,
    });
  }

  match rx.await {
    Ok(snapshot) => Json(snapshot),
    Err(_) => Json(LarvaeSnapshot {
      larva_responsibilities: HashMap::new(),
      assignment_details: HashMap::new(),
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
  pub role: String,
  pub status: String,
  pub units: Vec<MilitaryUnitInfo>,
  pub target_position: Option<(i32, i32)>,
  pub target_path: Option<Vec<(i32, i32)>>,
  pub target_path_index: Option<usize>,
  pub target_percentage: f32,
  pub target_player: Option<String>,
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

          // Calculate target player based on end of path
          let target_player = if let Some(ref path) = squad.target_path {
            if !path.is_empty() {
              let path_end = path[path.len() - 1];
              let start_locations = game.get_start_locations();

              // Find closest start location to path end
              let closest_start = start_locations.iter().min_by_key(|loc| {
                let loc_pixel_x = loc.x * 32;
                let loc_pixel_y = loc.y * 32;
                let dx = loc_pixel_x - path_end.0;
                let dy = loc_pixel_y - path_end.1;
                dx * dx + dy * dy
              });

              // Look up player from start_location_players mapping
              if let Some(closest) = closest_start {
                let location_key = (closest.x * 32, closest.y * 32);
                state.start_location_players.get(&location_key).cloned()
              } else {
                None
              }
            } else {
              None
            }
          } else {
            None
          };

          SquadData {
            name: squad.name.clone(),
            role: format!("{:?}", squad.role),
            status: format!("{:?}", squad.status),
            units,
            target_position: squad.target_position,
            target_path: squad.target_path.clone(),
            target_path_index: squad.target_path_index,
            target_percentage: squad.target_percentage,
            target_player,
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
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum BuildOrderItemDTO {
  Unit {
    unit_type: String,
    base_index: Option<usize>,
  },
  Upgrade {
    upgrade_type: String,
  },
  Squad {
    name: String,
    role: String,
    status: String,
  },
}

impl From<&BuildOrderItem> for BuildOrderItemDTO {
  fn from(item: &BuildOrderItem) -> Self {
    match item {
      BuildOrderItem::Unit {
        unit_type,
        base_index,
      } => BuildOrderItemDTO::Unit {
        unit_type: format!("{:?}", unit_type),
        base_index: *base_index,
      },
      BuildOrderItem::Upgrade(upgrade_type) => BuildOrderItemDTO::Upgrade {
        upgrade_type: format!("{:?}", upgrade_type),
      },
      BuildOrderItem::Squad { name, role, status } => BuildOrderItemDTO::Squad {
        name: name.clone(),
        role: format!("{:?}", role),
        status: format!("{:?}", status),
      },
    }
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildOrderSnapshot {
  pub build_order: Vec<BuildOrderItemDTO>,
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
          .map(|item| BuildOrderItemDTO::from(item))
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

#[derive(Clone, Debug, Serialize)]
pub struct PlayerLocationInfo {
  pub starting_location: (i32, i32),
  pub player_name: String,
  pub path_from_my_base: Option<Vec<(i32, i32)>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StartLocationsSnapshot {
  pub locations: Vec<PlayerLocationInfo>,
  pub frame_count: i32,
}

async fn start_locations_handler(
  State((game_state, _)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
) -> impl IntoResponse {
  if let Ok(state) = game_state.lock() {
    let mut locations: Vec<PlayerLocationInfo> = state
      .all_players
      .iter()
      .map(|player_info| PlayerLocationInfo {
        starting_location: player_info.starting_location,
        player_name: player_info.player_name.clone(),
        path_from_my_base: player_info.path_from_my_base.clone(),
      })
      .collect();

    // Sort by player name for consistency
    locations.sort_by(|a, b| a.player_name.cmp(&b.player_name));

    Json(StartLocationsSnapshot {
      locations,
      frame_count: 0,
    })
  } else {
    Json(StartLocationsSnapshot {
      locations: Vec::new(),
      frame_count: -1,
    })
  }
}

#[derive(Debug, Deserialize)]
pub struct UpdateSquadTargetPercentageRequest {
  pub squad_name: String,
  pub target_percentage: f32,
}

async fn update_squad_target_percentage_handler(
  State((game_state, _)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
  Json(req): Json<UpdateSquadTargetPercentageRequest>,
) -> impl IntoResponse {
  if let Ok(mut state) = game_state.lock() {
    if let Some(squad) = state
      .military_squads
      .iter_mut()
      .find(|s| s.name == req.squad_name)
    {
      squad.target_percentage = req.target_percentage.clamp(0.0, 1.0);

      let screen_position = if let Some(ref path) = squad.target_path {
        if !path.is_empty() {
          let index = ((path.len() - 1) as f32 * squad.target_percentage).round() as usize;
          let index = index.min(path.len() - 1);
          let target_pos = path[index];
          squad.target_position = Some(target_pos);
          squad.target_path_index = Some(index);
          Some((target_pos.0, target_pos.1))
        } else {
          None
        }
      } else {
        None
      };

      println!(
        "Updated squad '{}' target percentage to {}",
        req.squad_name, squad.target_percentage
      );

      if let Some(pos) = screen_position {
        state.move_screen_position = Some(pos);
      }
      "OK"
    } else {
      "Squad not found"
    }
  } else {
    "Error updating squad target percentage"
  }
}

#[derive(Debug, Deserialize)]
pub struct UpdateSquadTargetPlayerRequest {
  pub squad_name: String,
  pub target_player: String,
}

async fn update_squad_target_player_handler(
  State((game_state, _)): State<(SharedGameState, SharedHttpStatusCallbacks)>,
  Json(req): Json<UpdateSquadTargetPlayerRequest>,
) -> &'static str {
  if let Ok(mut state) = game_state.lock() {
    // Find the target location first
    let target_location = state
      .start_location_players
      .iter()
      .find(|(_, name)| **name == req.target_player)
      .map(|(pos, _)| *pos);

    if let Some(target_pos) = target_location {
      // Now find and update the squad
      if let Some(squad) = state
        .military_squads
        .iter_mut()
        .find(|s| s.name == req.squad_name)
      {
        // Set the target position
        squad.target_position = Some(target_pos);

        // Clear existing path so it gets recalculated
        squad.target_path = None;
        squad.target_path_index = None;

        println!(
          "Updated squad '{}' to target player '{}' at {:?}",
          req.squad_name, req.target_player, target_pos
        );

        "OK"
      } else {
        "Squad not found"
      }
    } else {
      "Target player location not found"
    }
  } else {
    "Error updating squad target player"
  }
}
