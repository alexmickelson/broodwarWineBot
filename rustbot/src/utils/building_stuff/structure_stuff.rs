use crate::utils::{building_stuff::build_location_utils, game_state::*};
use rsbwapi::*;
use std::collections::HashMap;

// pub fn build_building_if_can(
//   game: &Game,
//   game_state: &mut GameState,
//   player: &Player,
//   building_type: UnitType,
// ) {
//   let needed_minerals = building_type.mineral_price();
//   let needed_gas = building_type.gas_price();
//   game.draw_text_screen(
//     (0, 0),
//     &format!(
//       "next {:?}, {}/{} minerals, {}/{} gas",
//       building_type,
//       player.minerals(),
//       needed_minerals,
//       player.gas(),
//       needed_gas
//     ),
//   );

//   if player.minerals() < needed_minerals || player.gas() < needed_gas {
//     return;
//   }

//   make_building_assignment(game, game_state, building_type);
// }

// fn is_building_current_building(game_state: &GameState, building_type: UnitType) -> bool {
//   game_state.worker_assignments.iter().any(|(_, assignment)| {
//     if assignment.assignment_type != WorkerAssignmentType::Building {
//       return false;
//     }

//     let Some(build_order_idx) = assignment.build_order_index else {
//       return false;
//     };

//     let Some(assigned_building_item) = game_state.build_order.get(build_order_idx) else {
//       return false;
//     };

//     match assigned_building_item {
//       BuildOrderItem::Unit(assigned_building_type) => *assigned_building_type == building_type,
//       _ => false,
//     }
//   })
// }

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

  let building_id = building_of_type.get_id();
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

  let drone_id = drone.get_id();
  let build_position = (build_location.x, build_location.y);

  game_state.worker_assignments.insert(
    drone_id,
    WorkerAssignment::building(None, build_position, current_build_idx),
  );

  println!(
    "Assigned drone {} to build {:?} at tile position {:?} for build order index {}",
    drone_id, unit_type, build_position, current_build_idx
  );
}
