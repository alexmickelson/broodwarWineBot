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
  // Get the preferred base index from the build order item
  let base_index = game_state
    .build_order
    .get(current_build_idx)
    .and_then(|item| {
      if let crate::utils::build_orders::build_order_item::BuildOrderItem::Unit { base_index, .. } = item {
        *base_index
      } else {
        None
      }
    });

  // Find all buildings of the required type
  let all_buildings_of_type: Vec<_> = game.get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_type() == builder_type
        && u.get_player().get_id() == game.self_().map_or(0, |p| p.get_id())
    })
    .collect();

  println!(
    "Looking for {:?} to morph into {:?}. Found {} total buildings of type {:?}, preferred base_index: {:?}",
    builder_type,
    unit_type,
    all_buildings_of_type.len(),
    builder_type,
    base_index
  );

  // Check which ones are completed
  let completed_buildings: Vec<_> = all_buildings_of_type
    .iter()
    .filter(|u| u.is_completed())
    .collect();

  println!("  {} are completed", completed_buildings.len());

  // Check which completed ones are already assigned
  let assigned_completed: Vec<_> = completed_buildings
    .iter()
    .filter(|u| game_state.building_assignments.contains_key(&u.get_id()))
    .collect();

  println!("  {} completed are already assigned", assigned_completed.len());

  // Helper function to find closest building to a base location
  let find_closest_to_base = |buildings: Vec<&Unit>, base_idx: usize| -> Option<Unit> {
    let base_tile = game_state.base_locations.get(base_idx)?;
    let base_pos = Position::new(base_tile.x * 32 + 64, base_tile.y * 32 + 48); // Center of hatchery (4x3 tiles)
    
    buildings
      .into_iter()
      .filter(|u| !game_state.building_assignments.contains_key(&u.get_id()))
      .min_by_key(|u| {
        let building_pos = u.get_position();
        let dx = (building_pos.x - base_pos.x) as f32;
        let dy = (building_pos.y - base_pos.y) as f32;
        (dx * dx + dy * dy) as i32
      })
      .cloned()
  };

  // First try to find a completed, unassigned building at the preferred base
  let building_of_type = if let Some(idx) = base_index {
    println!("  Looking for building at base {}", idx);
    find_closest_to_base(completed_buildings.clone(), idx)
      .or_else(|| {
        println!("  No completed building at preferred base, trying any completed building");
        completed_buildings
          .into_iter()
          .find(|u| !game_state.building_assignments.contains_key(&u.get_id()))
          .cloned()
      })
  } else {
    completed_buildings
      .into_iter()
      .find(|u| !game_state.building_assignments.contains_key(&u.get_id()))
      .cloned()
  }
  .or_else(|| {
    // If no completed buildings are available, try to find an uncompleted, unassigned one
    println!("  No completed unassigned buildings, looking for uncompleted ones...");
    let uncompleted_buildings: Vec<_> = all_buildings_of_type
      .iter()
      .filter(|u| !u.is_completed())
      .collect();
    
    println!("  {} are uncompleted", uncompleted_buildings.len());
    
    let assigned_uncompleted: Vec<_> = uncompleted_buildings
      .iter()
      .filter(|u| game_state.building_assignments.contains_key(&u.get_id()))
      .collect();
    
    println!("  {} uncompleted are already assigned", assigned_uncompleted.len());
    
    if let Some(idx) = base_index {
      println!("  Looking for uncompleted building at base {}", idx);
      find_closest_to_base(uncompleted_buildings.clone(), idx)
        .or_else(|| {
          println!("  No uncompleted building at preferred base, trying any uncompleted building");
          uncompleted_buildings
            .into_iter()
            .find(|u| !game_state.building_assignments.contains_key(&u.get_id()))
            .cloned()
        })
    } else {
      uncompleted_buildings
        .into_iter()
        .find(|u| !game_state.building_assignments.contains_key(&u.get_id()))
        .cloned()
    }
  });

  let Some(building_of_type) = building_of_type else {
    println!(
      "ERROR: No available {:?} found to morph into {:?}",
      builder_type, unit_type
    );
    if all_buildings_of_type.is_empty() {
      println!("  No {:?} buildings exist at all!", builder_type);
    }
    game.draw_text_screen(
      (10, 10),
      &format!(
        "No available {:?} to morph into {:?}",
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

  let base_index = game_state
    .build_order
    .get(current_build_idx)
    .and_then(|item| {
      if let crate::utils::build_orders::build_order_item::BuildOrderItem::Unit { base_index, .. } = item {
        *base_index
      } else {
        None
      }
    });

  let build_position = if unit_type == UnitType::Zerg_Hatchery {
    if let Some(idx) = base_index {
      if let Some(base_tile) = game_state.base_locations.get(idx) {
        (base_tile.x, base_tile.y)
      } else {
        println!(
          "Base index {} out of bounds for hatchery (available bases: {})",
          idx,
          game_state.base_locations.len()
        );
        return;
      }
    } else {
      let Some(build_location) = build_location_utils::get_buildable_location(
        game,
        &drone,
        unit_type,
        &game_state.base_locations,
        base_index,
      ) else {
        println!(
          "No valid build location found for {:?} at base_index {:?} (build_order_index {})",
          unit_type, base_index, current_build_idx
        );
        return;
      };
      (build_location.x, build_location.y)
    }
  } else {
    let Some(build_location) = build_location_utils::get_buildable_location(
      game,
      &drone,
      unit_type,
      &game_state.base_locations,
      base_index,
    ) else {
      println!(
        "No valid build location found for {:?} at base_index {:?} (build_order_index {})",
        unit_type, base_index, current_build_idx
      );
      return;
    };
    (build_location.x, build_location.y)
  };

  let drone_id = drone.get_id();

  game_state.worker_assignments.insert(
    drone_id,
    WorkerAssignment::building(None, build_position, current_build_idx),
  );

  println!(
    "Assigned drone {} to build {:?} at tile position {:?} for build order index {}",
    drone_id, unit_type, build_position, current_build_idx
  );
}

pub fn enforce_structure_assignment(game: &Game, game_state: &mut GameState) {
  let Some(player) = game.self_() else {
    return;
  };

  let current_build_idx = game_state.build_order_index;
  
  // Get the building type we're trying to build
  let unit_type = match game_state.build_order.get(current_build_idx) {
    Some(crate::utils::build_orders::build_order_item::BuildOrderItem::Unit { unit_type, .. }) => *unit_type,
    _ => return,
  };

  // Check if there's a building assigned to this build order index (for morphing buildings)
  let building_assigned_for_current_index = game_state
    .building_assignments
    .iter()
    .find_map(|(&building_id, assignment)| {
      if assignment.build_order_index == current_build_idx {
        Some((building_id, game.get_unit(building_id)))
      } else {
        None
      }
    });

  if let Some((_building_id, Some(building_unit))) = building_assigned_for_current_index {
    let needed_minerals = unit_type.mineral_price();
    let needed_gas = unit_type.gas_price();

    game.draw_text_screen(
      (0, 60),
      &format!(
        "Morphing {:?}, {}/{} minerals, {}/{} gas",
        unit_type,
        player.minerals(),
        needed_minerals,
        player.gas(),
        needed_gas
      ),
    );

    if player.minerals() >= needed_minerals && player.gas() >= needed_gas {
      if building_unit.is_completed() && !building_unit.is_morphing() {
        if let Err(e) = building_unit.morph(unit_type) {
          game.draw_text_screen(
            (0, 80),
            &format!("Failed to morph building into {:?}: {:?}", unit_type, e),
          );
        }
      }
    }
    return;
  }

  // Check if there's a worker assigned to this build order index
  let worker_assigned_for_current_index = game_state
    .worker_assignments
    .iter()
    .find_map(|(&worker_id, assignment)| {
      if assignment.build_order_index == Some(current_build_idx) {
        game.get_unit(worker_id)
      } else {
        None
      }
    });

  if worker_assigned_for_current_index.is_none() {
    game.draw_text_screen(
      (0, 30),
      format!(
        "No worker/building assigned for build order index {}, trying to assign",
        current_build_idx
      )
      .as_str(),
    );
    crate::utils::build_order_management::make_assignment_for_current_build_order_item(game, game_state);
  }
}
