use crate::utils::game_state::{DebugFlag, SharedGameState};
use crate::utils::http_status_callbacks::SharedHttpStatusCallbacks;
use crate::utils::{build_order_management, military_management, region_stuff};
use crate::utils::{pathing, worker_management};
use rsbwapi::*;

impl AiModule for RustBot {
  fn on_start(&mut self, game: &Game) {
    game.send_text("RustBot initialized!");

    // SAFETY: rsbwapi uses interior mutability (RefCell) for the command queue.
    // enable_flag only adds a command to the queue.
    // This cast is safe in the single-threaded BWAPI callback context.
    unsafe {
      let game_ptr = game as *const Game as *mut Game;
      (*game_ptr).enable_flag(Flag::UserInput as i32);
    }

    update_path_to_enemy(game, &self.game_state);

    println!("Game started on map: {}", game.map_file_name());
  }

  fn on_frame(&mut self, game: &Game) {
    update_game_speed(game, &self.game_state);

    build_order_management::build_order_onframe(game, &self.game_state);
    worker_management::update_assignments(game, &self.game_state);
    worker_management::enforce_assignments(game, &self.game_state);

    military_management::military_onframe(game, &mut self.game_state);

    draw_debug_lines(game, &self.game_state);

    if let Ok(mut callbacks) = self.http_callbacks.lock() {
      if callbacks.has_pending() {
        if let Ok(state) = self.game_state.lock() {
          callbacks.process_all(game, &*state);
        }
      }
    }
  }

  fn on_unit_create(&mut self, _game: &Game, _unit: Unit) {}

  fn on_unit_destroy(&mut self, _game: &Game, _unit: Unit) {}

  fn on_unit_complete(&mut self, _game: &Game, _unit: Unit) {}

  fn on_end(&mut self, _game: &Game, is_winner: bool) {
    if is_winner {
      println!("Victory!");
    } else {
      println!("Defeat!");
    }
  }
}
pub struct RustBot {
  game_state: SharedGameState,
  http_callbacks: SharedHttpStatusCallbacks,
}

impl RustBot {
  pub fn new(game_state: SharedGameState, http_callbacks: SharedHttpStatusCallbacks) -> Self {
    Self {
      game_state,
      http_callbacks,
    }
  }
}

fn update_game_speed(game: &Game, game_state: &SharedGameState) {
  let speed = game_state.lock().unwrap().game_speed;

  // SAFETY: rsbwapi uses interior mutability (RefCell) for the command queue.
  // set_local_speed only adds a command to the queue, it doesn't modify game state.
  // This cast is safe in the single-threaded BWAPI callback context.
  unsafe {
    let game_ptr = game as *const Game as *mut Game;
    (*game_ptr).set_local_speed(speed);
  }
}

fn update_path_to_enemy(game: &Game, game_state: &SharedGameState) {
  let Some(mut game_state) = game_state.lock().ok() else {
    return;
  };

  if game_state.path_to_enemy_base.is_none() {
    let Some(self_player) = game.self_() else {
      return;
    };

    let start_locations = game.get_start_locations();
    let Some(my_starting_position) = start_locations.get(self_player.get_id() as usize) else {
      return;
    };

    let Some(enemy_location) = start_locations
      .iter()
      .find(|&loc| loc != my_starting_position)
    else {
      return;
    };

    let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
    let enemy_pos = (enemy_location.x * 32, enemy_location.y * 32);

    game_state.path_to_enemy_base = pathing::get_path_between_points(game, my_pos, enemy_pos);
  }
}

fn draw_debug_lines(game: &Game, game_state: &SharedGameState) {
  let Ok(state) = game_state.lock() else {
    return;
  };

  for flag in &state.debug_flags {
    match flag {
      DebugFlag::ShowWorkerAssignments => {
        worker_management::draw_worker_resource_lines(game, &state.worker_assignments.clone());
        worker_management::draw_worker_ids(game);
        worker_management::draw_building_ids(game);
      }
      DebugFlag::ShowMilitaryAssignments => {
        military_management::draw_military_assignments(game, &state);
      }
      DebugFlag::ShowPathToEnemyBase => {
        if let Some(path) = state.path_to_enemy_base.as_ref() {
          pathing::draw_path(game, path);
        }
      }
      DebugFlag::ShowRegions => {
        region_stuff::draw_region_boxes(game);
      }
    }
  }
}
