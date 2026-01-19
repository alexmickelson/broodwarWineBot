use crate::utils::build_order_management;
use crate::utils::build_orders::build_order_item::BuildOrderItem;
use crate::utils::build_orders::pool_speed_expand;
use crate::utils::building_stuff::{creature_stuff, researching_stuff};
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

    game_state.build_order = pool_speed_expand::build_order();

    let Some(initial_squad) = military_management::create_initial_squad(game) else {
      return;
    };

    game_state.military_squads.push(initial_squad);

    println!("Making initial build order assignment");
    build_order_management::make_assignment_for_current_build_order_item(game, &mut game_state);
  }

  fn on_frame(&mut self, game: &Game) {
    // println!("Frame {}", game.get_frame_count());
    let Ok(mut locked_state) = self.game_state.lock() else {
      return;
    };

    if researching_stuff::has_started_current_upgrade(game, &locked_state) {
      let upgrade_name = if let Some(BuildOrderItem::Upgrade(upgrade_type)) = 
        locked_state.build_order.get(locked_state.build_order_index) {
        format!("{:?}", upgrade_type)
      } else {
        "Unknown".to_string()
      };
      build_order_management::advance_build_order(
        game, 
        &mut locked_state, 
        &format!("Upgrade {:} started", upgrade_name)
      );
    }

    update_game_speed(game, &locked_state);

    build_order_management::build_order_enforce_assignments(game, &mut locked_state);

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

  // new buildings -> on_unit_create
  // evolving buildings for zerg -> on_unit_morph
  // evolving larvae for zerg -> on_unit_morph
  // upgrades -> need to figure out in on_frame
  fn on_unit_create(&mut self, game: &Game, unit: Unit) {
    println!("unit created: {:?}", unit.get_type());
    if game.get_frame_count() < 1 {
      return;
    }

    let Ok(mut locked_state) = self.game_state.lock() else {
      return;
    };

    build_order_management::build_order_on_unit_started(game, &unit, &mut locked_state);
  }
  fn on_unit_morph(&mut self, game: &Game, unit: Unit) {
    println!(
      "unit {} started morphing: {:?} -> {:?}",
      unit.get_id(),
      unit.get_type(),
      unit.get_build_type()
    );
    let Ok(mut locked_state) = self.game_state.lock() else {
      return;
    };
    if unit.get_type() == UnitType::Zerg_Egg {
      // unit started morphing, remove larva responsibility
      creature_stuff::remove_larva_responsibility(&mut locked_state, &unit);

      // Check if this morph matches the current build order item
      let should_advance = if let Some(current_item) =
        locked_state.build_order.get(locked_state.build_order_index)
      {
        match current_item {
          BuildOrderItem::Unit(expected_unit_type) => unit.get_build_type() == *expected_unit_type,
          BuildOrderItem::Upgrade(_) => {
            // Don't advance on unit morphs if waiting for an upgrade
            false
          }
        }
      } else {
        false
      };

      if should_advance {
        build_order_management::advance_build_order(
          game,
          &mut locked_state,
          &format!("Zerg_Egg started morphing into {:?}", unit.get_build_type()),
        );
      } else {
        println!(
          "Zerg_Egg morphing into {:?}, but not advancing build order (current item: {:?})",
          unit.get_build_type(),
          locked_state.build_order.get(locked_state.build_order_index)
        );
      }
      return;
    }

    if unit.get_type().is_building() {
      build_order_management::remove_drone_assignment_after_started_buidling(
        &unit,
        &mut locked_state,
      );

      // Check if this building matches the current build order item
      let should_advance =
        if let Some(current_item) = locked_state.build_order.get(locked_state.build_order_index) {
          match current_item {
            BuildOrderItem::Unit(expected_unit_type) => unit.get_type() == *expected_unit_type,
            BuildOrderItem::Upgrade(_) => {
              // Don't advance on building construction if waiting for an upgrade
              false
            }
          }
        } else {
          false
        };

      if should_advance {
        build_order_management::advance_build_order(
          game,
          &mut locked_state,
          &format!("Building {:?} started construction", unit.get_type()),
        );
      } else {
        println!(
          "{:?} started construction, but not advancing build order (current item: {:?})",
          unit.get_type(),
          locked_state.build_order.get(locked_state.build_order_index)
        );
      }
    }
  }

  fn on_unit_destroy(&mut self, _game: &Game, unit: Unit) {
    if military_management::is_military_unit(&unit) {
      military_management::remove_unit_from_squads(&unit, &mut self.game_state.lock().unwrap());
    }
  }

  fn on_unit_complete(&mut self, game: &Game, unit: Unit) {
    let Some(player) = game.self_() else {
      return;
    };

    // Only assign our own units to squads, not enemy units
    if unit.get_player().get_id() == player.get_id() && military_management::is_military_unit(&unit) {
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
    
    let frame_skip_value = if speed == 0 { 15 } else { 0 };
    (*game_ptr).set_frame_skip(frame_skip_value);
  }
}

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
