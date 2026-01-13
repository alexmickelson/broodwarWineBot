use crate::utils::{building_stuff::build_location_utils, game_state::*};
use rsbwapi::*;
use std::collections::HashMap;

pub fn build_building_onframe(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
  building_type: UnitType,
) {
  let needed_minerals = building_type.mineral_price();
  let needed_gas = building_type.gas_price();
  game.draw_text_screen(
    (0, 0),
    &format!(
      "next {:?}, {}/{} minerals, {}/{} gas",
      building_type,
      player.minerals(),
      needed_minerals,
      player.gas(),
      needed_gas
    ),
  );

  if is_building_current_building(game_state, building_type) {
    game.draw_text_screen(
      (0, 20),
      &format!("Worker already assigned to build {:?}", building_type),
    );
    return;
  }

  if player.minerals() < needed_minerals || player.gas() < needed_gas {
    return;
  }

  build_building(game, game_state, building_type);
}

fn is_building_current_building(game_state: &GameState, building_type: UnitType) -> bool {
  game_state.worker_assignments.iter().any(|(_, assignment)| {
    if assignment.assignment_type != WorkerAssignmentType::Building {
      return false;
    }

    let Some(build_order_idx) = assignment.build_order_index else {
      return false;
    };

    let Some(assigned_building_item) = game_state.build_order.get(build_order_idx) else {
      return false;
    };

    match assigned_building_item {
      BuildOrderItem::Unit(assigned_building_type) => *assigned_building_type == building_type,
      _ => false,
    }
  })
}

fn choose_drone_to_build(game: &Game, game_state: &GameState) -> Option<Unit> {
  let mineral_patch_with_most_workers = game_state
    .worker_assignments
    .iter()
    .filter_map(|(_, assignment)| {
      if let WorkerAssignmentType::Gathering = assignment.assignment_type {
        assignment.target_unit
      } else {
        None
      }
    })
    .fold(HashMap::new(), |mut acc, mineral_id| {
      *acc.entry(mineral_id).or_insert(0) += 1;
      acc
    })
    .into_iter()
    .max_by_key(|&(_, count)| count)
    .map(|(mineral_id, _)| mineral_id);

  game_state
    .worker_assignments
    .iter()
    .find_map(|(&worker_id, assignment)| {
      if let WorkerAssignmentType::Gathering = assignment.assignment_type {
        if let Some(mineral_id) = mineral_patch_with_most_workers {
          if assignment.target_unit == Some(mineral_id) {
            return game.get_unit(worker_id);
          }
        }
      }
      None
    })
}

fn build_building(game: &Game, game_state: &mut GameState, unit_type: UnitType) {
  let current_build_idx = game_state.build_order_index;

  let (builder_type, _) = unit_type.what_builds();
  if builder_type.is_building() {
    let Some(building_of_type) = game.get_all_units().into_iter().find(|u| {
      u.get_type() == builder_type
        && u.get_player().get_id() == game.self_().map_or(0, |p| p.get_id())
        && u.is_completed()
    }) else {
      game.draw_text_screen(
        (10, 10),
        &format!("A building of type {:?} cannot be found to build {:?}", builder_type, unit_type),
      );
      return;
    };

    let _ = building_of_type.train(unit_type);
    println!("Commanded building {} to train {:?}", building_of_type.get_id(), unit_type);
    return;
  }

  // Check if there's already a worker assigned to this build order index
  let already_assigned = game_state.worker_assignments.values().any(|assignment| {
    assignment.assignment_type == WorkerAssignmentType::Building
      && assignment.build_order_index == Some(current_build_idx)
  });

  if already_assigned {
    game.draw_text_screen(
      (0, 10),
      &format!("Worker assigned to build {:?}", unit_type),
    );
    return;
  }

  let Some(drone) = choose_drone_to_build(game, game_state) else {
    game.draw_text_screen((10, 10), "No available drone to build building");
    return;
  };

  let Some(build_location) = build_location_utils::get_buildable_location(game, &drone, unit_type) else {
    game.draw_text_screen((10, 10), "No valid build location found");
    return;
  };

  let drone_id = drone.get_id() as usize;
  let build_position = (build_location.x * 32, build_location.y * 32);

  game_state.worker_assignments.insert(
    drone_id,
    WorkerAssignment::building(None, build_position, current_build_idx),
  );

  println!(
    "Assigned drone {} to build {:?} at position {:?} for build order index {}",
    drone_id, unit_type, build_position, current_build_idx
  );
}

pub fn advance_build_order_if_building_building(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
) {
  if game_state.build_order_index >= game_state.build_order.len() {
    return;
  }

  let BuildOrderItem::Unit(current_building_type) =
    game_state.build_order[game_state.build_order_index]
  else {
    return;
  };
  if !current_building_type.is_building() {
    return;
  }

  let morphing_building_ids: Vec<usize> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_type() == current_building_type
        && u.get_player().get_id() == player.get_id()
        && !u.is_completed()
    })
    .map(|u| u.get_id() as usize)
    .collect();

  if morphing_building_ids.is_empty() {
    return;
  }

  game.draw_text_screen(
    (0, 40),
    &format!(
      "Buildings morphing into {:?}: {:?}",
      current_building_type, morphing_building_ids
    ),
  );

  let has_assignment_for_current_index = game_state
    .worker_assignments
    .values()
    .any(|assignment| assignment.build_order_index == Some(game_state.build_order_index));

  if has_assignment_for_current_index {
    return;
  }

  if morphing_building_ids.is_empty() {
    println!(
      "Worker assignment for {:?} disappeared but no building exists - worker may have been killed",
      current_building_type
    );
    return;
  }

  // Building exists, so the drone successfully morphed into it
  game_state.build_order_index += 1;
  println!(
    "Building started morphing (worker morphed into building), advancing build order index to {}",
    game_state.build_order_index
  );
}
