use crate::utils::build_order_management;
use crate::utils::building_stuff::creature_stuff;
use crate::utils::game_state::{DebugFlag, GameState, SharedGameState};
use crate::utils::http_status_callbacks::SharedHttpStatusCallbacks;
use crate::utils::map_utils::{pathing, region_stuff};
use crate::utils::military::military_management;
use crate::utils::worker_management;
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

    println!("Game started on map: {}", game.map_file_name());

    let Some(mut game_state) = self.game_state.lock().ok() else {
      return;
    };

    let Some(initial_squad) = military_management::create_initial_squad(game) else {
      return;
    };

    game_state.military_squads.push(initial_squad);
  }

  fn on_frame(&mut self, game: &Game) {
    let Ok(mut locked_state) = self.game_state.lock() else {
      return;
    };

    update_game_speed(game, &locked_state);

    let all_my_overlords_that_are_not_moving = game
      .get_all_units()
      .into_iter()
      .filter(|u| {
        u.get_player().get_id() == game.self_().unwrap().get_id()
          && u.get_type() == UnitType::Zerg_Overlord
          && u.get_order() != Order::Move
      })
      .collect::<Vec<Unit>>();

    build_order_management::build_order_on_frame(game, &mut locked_state);

    worker_management::update_assignments(game, &mut locked_state);
    worker_management::enforce_assignments(game, &mut locked_state);

    military_management::military_onframe(game, &mut locked_state);

    draw_debug_lines(game, &locked_state);

    if let Ok(mut callbacks) = self.http_callbacks.lock() {
      if callbacks.has_pending() {
        callbacks.process_all(game, &*locked_state);
      }
    }
  }

  // creatures and new buildings -> on_unit_create
  // evolving buildings for zerg -> on_unit_morph
  // evolving larvae for zerg -> on_unit_morph
  // upgrades -> need to figure out in on_frame
  fn on_unit_create(&mut self, game: &Game, unit: Unit) {
    if game.get_frame_count() < 1 {
      return;
    }

    let Ok(mut locked_state) = self.game_state.lock() else {
      return;
    };

    build_order_management::build_order_on_unit_started(game, &unit, &mut locked_state);
  }
  fn on_unit_morph(&mut self, _game: &Game, unit: Unit) {
    println!(
      "unit started morphing: {:?} -> {:?}",
      unit.get_type(),
      unit.get_build_type()
    );
    let Ok(mut locked_state) = self.game_state.lock() else {
      return;
    };
    if unit.get_build_type() == UnitType::None {
      // unit finished morphing, remove larva responsibility

      creature_stuff::remove_larva_responsibility(&mut locked_state, &unit);
      return;
    }
  }

  fn on_unit_destroy(&mut self, _game: &Game, unit: Unit) {
    if military_management::is_military_unit(&unit) {
      military_management::remove_unit_from_squads(&unit, &mut self.game_state.lock().unwrap());
    }
  }

  fn on_unit_complete(&mut self, game: &Game, unit: Unit) {
    if military_management::is_military_unit(&unit) {
      military_management::assign_unit_to_squad(&game, &unit, &mut self.game_state.lock().unwrap());
    }
  }

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

fn update_game_speed(game: &Game, game_state: &GameState) {
  let speed = game_state.game_speed;

  // SAFETY: rsbwapi uses interior mutability (RefCell) for the command queue.
  // set_local_speed only adds a command to the queue, it doesn't modify game state.
  // This cast is safe in the single-threaded BWAPI callback context.
  unsafe {
    let game_ptr = game as *const Game as *mut Game;
    (*game_ptr).set_local_speed(speed);
  }
}

// fn update_path_to_enemy(game: &Game, game_state: &SharedGameState) {
//   let Some(mut game_state) = game_state.lock().ok() else {
//     return;
//   };

//   if game_state.path_to_enemy_base.is_none() {
//     let Some(self_player) = game.self_() else {
//       return;
//     };

//     let start_locations = game.get_start_locations();
//     let Some(my_starting_position) = start_locations.get(self_player.get_id() as usize) else {
//       return;
//     };

//     let Some(enemy_location) = start_locations
//       .iter()
//       .find(|&loc| loc != my_starting_position)
//     else {
//       return;
//     };

//     let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
//     let enemy_pos = (enemy_location.x * 32, enemy_location.y * 32);

//     game_state.path_to_enemy_base = pathing::get_path_between_points(game, my_pos, enemy_pos);
//   }
// }

fn draw_debug_lines(game: &Game, game_state: &GameState) {
  for flag in &game_state.debug_flags {
    match flag {
      DebugFlag::ShowWorkerAssignments => {
        worker_management::draw_worker_resource_lines(game, &game_state.worker_assignments.clone());
        worker_management::draw_worker_ids(game);
        worker_management::draw_building_ids(game);
      }
      DebugFlag::ShowMilitaryAssignments => {
        military_management::draw_military_assignments(game, &game_state);
      }
      DebugFlag::ShowPathToEnemyBase => {
        if let Some(path) = game_state.path_to_enemy_base.as_ref() {
          pathing::draw_path(game, path);
        }
      }
      DebugFlag::ShowRegions => {
        region_stuff::draw_region_boxes(game);
      }
    }
  }
}
