use crate::utils::build_location_utils::get_buildable_location;
use crate::utils::game_state::{
  BuildOrderItem, GameState, SharedGameState, WorkerAssignment, WorkerAssignmentType,
};
use rsbwapi::*;
use std::collections::HashMap;

pub fn build_order_onframe(game: &Game, game_state: &SharedGameState) {
  let Ok(mut game_state) = game_state.lock() else {
    println!("Failed to lock game_state in build_order_onframe");
    return;
  };

  let Some(player) = game.self_() else {
    println!("Failed to get self player in build_order_onframe");
    return;
  };

  advance_build_order_for_morphed_larvae(game, &mut game_state, &player);
  advance_build_order_if_building_building(game, &mut game_state, &player);

  if game_state.build_order_index >= game_state.build_order.len() {
    println!("build order empty");
    figure_out_what_to_build(game, &mut game_state);
    return;
  }

  let BuildOrderItem::Unit(unit_type) = game_state.build_order[game_state.build_order_index] else {
    println!("Current build order item is not a unit, skipping");
    game_state.build_order_index += 1;
    return;
  };

  let needed_minerals = unit_type.mineral_price();

  let larva_units: Vec<Unit> = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Zerg_Larva && u.get_player().get_id() == player.get_id())
    .collect();

  game.draw_text_screen(
    (0, 0),
    &format!(
      "next {:?}, {}/{} minerals, {} larva",
      unit_type,
      player.minerals(),
      needed_minerals,
      larva_units.len()
    ),
  );

  if is_building_current_building(&game_state, unit_type) {
    game.draw_text_screen(
      (0, 20),
      &format!("Worker already assigned to build {:?}", unit_type),
    );
    return;
  }

  if player.minerals() < needed_minerals {
    return;
  }

  match unit_type {
    ut if ut.is_building() => {
      build_building(game, &mut game_state, unit_type);
    }
    _ => {
      build_unit_from_larva(game, &mut game_state, &larva_units, unit_type);
    }
  }
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

fn build_unit_from_larva(
  game: &Game,
  game_state: &mut GameState,
  larva_units: &[Unit],
  unit_type: UnitType,
) {
  // Find a larva that doesn't have a responsibility yet
  let available_larva = larva_units.iter().find(|larva| {
    !game_state
      .larva_responsibilities
      .contains_key(&(larva.get_id() as usize))
  });

  let Some(larva) = available_larva else {
    game.draw_text_screen((10, 10), "No available larva to train unit");
    return;
  };

  let larva_id = larva.get_id() as usize;
  let current_build_idx = game_state.build_order_index;

  if larva.train(unit_type).is_ok() {
    // Assign this larva the responsibility of morphing this build order item
    game_state
      .larva_responsibilities
      .insert(larva_id, current_build_idx);
    println!(
      "Assigned larva {} to build order index {}",
      larva_id, current_build_idx
    );
  }
}

fn build_building(game: &Game, game_state: &mut GameState, unit_type: UnitType) {
  let current_build_idx = game_state.build_order_index;

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

  let chosen_drone = game_state
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
    });
  let Some(drone) = chosen_drone else {
    game.draw_text_screen((10, 10), "No available drone to build building");
    return;
  };

  let Some(build_location) = get_buildable_location(game, &drone, unit_type) else {
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

fn figure_out_what_to_build(game: &Game, game_state: &mut GameState) {
  let Some(player) = game.self_() else {
    println!("Failed to get self player in figure_out_what_to_build");
    return;
  };
  let supply_total = player.supply_total() / 2;
  let supply_used = player.supply_used() / 2;
  let supply_remaining = supply_total - supply_used;

  let overlords_in_production = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_type() == UnitType::Zerg_Egg
        && u.get_player().get_id() == player.get_id()
        && u.get_build_type() == UnitType::Zerg_Overlord
    })
    .count();

  if supply_remaining < 4 && overlords_in_production == 0 {
    println!(
      "queuing overlord because supply is {} and overlords in production is {}",
      supply_remaining, overlords_in_production
    );
    game_state.build_order.push(BuildOrderItem::Unit(UnitType::Zerg_Overlord));
    return;
  }

  let total_drones = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Zerg_Drone && u.get_player().get_id() == player.get_id())
    .count();
  if total_drones < 20 {
    println!("queuing drone because total drones is {}", total_drones);
    game_state.build_order.push(BuildOrderItem::Unit(UnitType::Zerg_Drone));
  } else {
    println!("queuing zergling");
    game_state.build_order.push(BuildOrderItem::Unit(UnitType::Zerg_Zergling));
  }
}

fn advance_build_order_if_building_building(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
) {
  if game_state.build_order_index >= game_state.build_order.len() {
    return;
  }

  let BuildOrderItem::Unit(current_building_type) = game_state.build_order[game_state.build_order_index] else {
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


fn advance_build_order_for_morphed_larvae(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
) {
  let current_larva_ids: std::collections::HashSet<usize> = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Zerg_Larva && u.get_player().get_id() == player.get_id())
    .map(|u| u.get_id() as usize)
    .collect();

  let mut morphed_indices = Vec::new();
  game_state
    .larva_responsibilities
    .retain(|larva_id, build_idx| {
      if !current_larva_ids.contains(larva_id) {
        morphed_indices.push(*build_idx);
        false
      } else {
        true
      }
    });

  for morphed_idx in morphed_indices {
    if morphed_idx == game_state.build_order_index {
      game_state.build_order_index += 1;
      println!(
        "Larva morphed, advancing build order index to {}",
        game_state.build_order_index
      );
    }
  }
}
