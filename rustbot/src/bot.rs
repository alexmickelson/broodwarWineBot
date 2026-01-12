use crate::utils::game_state::SharedGameState;
use crate::utils::http_status_callbacks::SharedHttpStatusCallbacks;
use crate::utils::worker_management;
use crate::utils::{build_order_management, military_management};
use rsbwapi::*;

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

  fn update_game_speed(&self, game: &Game) {
    let speed = self.game_state.lock().unwrap().game_speed;

    // SAFETY: rsbwapi uses interior mutability (RefCell) for the command queue.
    // set_local_speed only adds a command to the queue, it doesn't modify game state.
    // This cast is safe in the single-threaded BWAPI callback context.
    unsafe {
      let game_ptr = game as *const Game as *mut Game;
      (*game_ptr).set_local_speed(speed);
    }
  }
}

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

    println!("Game started on map: {}", game.map_file_name());
  }

  fn on_frame(&mut self, game: &Game) {
    self.update_game_speed(game);

    build_order_management::build_order_onframe(game, &self.game_state);
    worker_management::update_assignments(game, &self.game_state);
    worker_management::enforce_assignments(game, &self.game_state);

    military_management::military_onframe(game, &mut self.game_state);

    worker_management::draw_worker_resource_lines(
      game,
      &self.game_state.lock().unwrap().worker_assignments.clone(),
    );
    worker_management::draw_worker_ids(game);
    worker_management::draw_building_ids(game);
    military_management::draw_military_assignments(game, &self.game_state.lock().unwrap());

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
