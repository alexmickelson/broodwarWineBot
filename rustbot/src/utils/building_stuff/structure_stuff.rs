use crate::utils::{building_stuff::build_location_utils, game_state::*};
use rsbwapi::*;
use std::collections::HashMap;

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

pub fn make_building_assignment(game: &Game, game_state: &mut GameState, unit_type: UnitType) {
  let current_build_idx = game_state.build_order_index;

  let (builder_type, _) = unit_type.what_builds();

  if builder_type.is_building() {
    assign_building_to_morph_into_building(
      game,
      game_state,
      unit_type,
      builder_type,
      current_build_idx,
    );
  } else {
    assign_drone_to_build_building(game, game_state, unit_type, current_build_idx);
  }
}

fn assign_building_to_morph_into_building(
  game: &Game,
  game_state: &mut GameState,
  unit_type: UnitType,
  builder_type: UnitType,
  current_build_idx: usize,
) -> bool {
  let Some(building_of_type) = game.get_all_units().into_iter().find(|u| {
    u.get_type() == builder_type
      && u.get_player().get_id() == game.self_().map_or(0, |p| p.get_id())
      && u.is_completed()
  }) else {
    game.draw_text_screen(
      (10, 10),
      &format!(
        "A building of type {:?} cannot be found to build {:?}",
        builder_type, unit_type
      ),
    );
    return false;
  };

  let building_id = building_of_type.get_id() as usize;
  game_state.building_assignments.insert(
    building_id,
    BuildingAssignment::new(unit_type, current_build_idx),
  );
  println!(
    "Assigned building {} to train {:?} for build order index {}",
    building_id, unit_type, current_build_idx
  );
  true
}

fn assign_drone_to_build_building(
  game: &Game,
  game_state: &mut GameState,
  unit_type: UnitType,
  current_build_idx: usize,
) {
  let Some(drone) = choose_drone_to_build(game, game_state) else {
    game.draw_text_screen((10, 10), "No available drone to build building");
    return;
  };

  let Some(build_location) = build_location_utils::get_buildable_location(game, &drone, unit_type)
  else {
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
