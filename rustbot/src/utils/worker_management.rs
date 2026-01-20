use crate::utils::{
  build_orders::build_order_item::BuildOrderItem,
  building_stuff::build_location_utils,
  game_state::{GameState, WorkerAssignment, WorkerAssignmentType},
};
use rand::seq::SliceRandom;
use rsbwapi::*;
use std::collections::HashMap;

fn get_my_workers(game: &Game) -> Vec<Unit> {
  let self_player = match game.self_() {
    Some(p) => p,
    None => return Vec::new(),
  };

  self_player
    .get_units()
    .into_iter()
    .filter(|u| u.get_type().is_worker())
    .collect()
}

pub fn update_assignments(game: &Game, game_state: &mut GameState) {
  let my_units = get_my_workers(game);
  let workers: Vec<_> = my_units.iter().collect();

  let mut assignments = game_state.worker_assignments.clone();

  remove_dead_workers(&mut assignments, &workers);

  let mut claimed_base_indices = std::collections::HashSet::new();
  claimed_base_indices.insert(0); // Main base is always claimed

  for (idx, item) in game_state.build_order.iter().enumerate() {
    if idx >= game_state.build_order_index {
      break; // Only look at completed items
    }
    if let BuildOrderItem::Unit {
      unit_type,
      base_index,
    } = item
    {
      if *unit_type == UnitType::Zerg_Hatchery {
        if let Some(base_idx) = base_index {
          claimed_base_indices.insert(*base_idx);
        }
      }
    }
  }

  // Collect minerals and extractors from all claimed bases
  let static_minerals = game.get_static_minerals();
  let mut minerals: Vec<&Unit> = Vec::new();

  for &base_idx in &claimed_base_indices {
    if let Some(base_location) = game_state.base_locations.get(base_idx) {
      let base_pos = Position::new(base_location.x * 32 + 64, base_location.y * 32 + 48);

      let base_minerals: Vec<_> = static_minerals
        .iter()
        .filter(|m| {
          let mineral_pos = m.get_position();
          let dx = (mineral_pos.x - base_pos.x) as f32;
          let dy = (mineral_pos.y - base_pos.y) as f32;
          let distance = (dx * dx + dy * dy).sqrt();
          distance <= 12.0 * 32.0
        })
        .collect();

      minerals.extend(base_minerals);
    }
  }

  let extractors: Vec<_> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      if u.get_type() != UnitType::Zerg_Extractor
        || u.get_player().get_id() != game.self_().map(|p| p.get_id()).unwrap_or(0)
        || !u.is_completed()
      {
        return false;
      }

      // Check if extractor is near any claimed base
      let extractor_pos = u.get_position();
      for &base_idx in &claimed_base_indices {
        if let Some(base_location) = game_state.base_locations.get(base_idx) {
          let base_pos = Position::new(base_location.x * 32 + 64, base_location.y * 32 + 48);
          let dx = (extractor_pos.x - base_pos.x) as f32;
          let dy = (extractor_pos.y - base_pos.y) as f32;
          let distance = (dx * dx + dy * dy).sqrt();
          if distance <= 12.0 * 32.0 {
            return true;
          }
        }
      }
      false
    })
    .collect();

  let unassigned_idle_workers: Vec<_> = workers
    .iter()
    .filter(|w| w.is_idle() && !assignments.contains_key(&w.get_id()))
    .copied()
    .collect();

  let mut mineral_worker_count = count_workers_per_resource(&assignments);

  // Determine extractor saturation based on worker count
  let total_workers = workers.len();
  let extractor_saturation = if total_workers > 22 { 3 } else { 2 };

  for worker in unassigned_idle_workers {
    let undersaturated_extractor = extractors.iter().find(|extractor| {
      let extractor_id = extractor.get_id();
      let worker_count = assignments
        .values()
        .filter(|a| {
          a.assignment_type == WorkerAssignmentType::Gathering
            && a.target_unit == Some(extractor_id)
        })
        .count();
      worker_count < extractor_saturation
    });

    if let Some(extractor) = undersaturated_extractor {
      let extractor_id = extractor.get_id();
      assignments.insert(worker.get_id(), WorkerAssignment::gathering(extractor_id));
    } else {
      let best_mineral = find_least_saturated_mineral(&minerals, &mineral_worker_count, 2)
        .or_else(|| find_least_saturated_mineral(&minerals, &mineral_worker_count, 3));

      if let Some(mineral) = best_mineral {
        let mineral_id = mineral.get_id();
        assignments.insert(worker.get_id(), WorkerAssignment::gathering(mineral_id));
        *mineral_worker_count.entry(mineral_id).or_insert(0) += 1;
      }
    }
  }

  // If there are still undersaturated extractors, reassign workers from minerals
  for extractor in &extractors {
    let extractor_id = extractor.get_id();
    let worker_count = assignments
      .values()
      .filter(|a| {
        a.assignment_type == WorkerAssignmentType::Gathering && a.target_unit == Some(extractor_id)
      })
      .count();

    if worker_count < extractor_saturation {
      // Find a worker assigned to minerals
      let worker_to_reassign = assignments.iter().find_map(|(worker_id, assignment)| {
        if assignment.assignment_type == WorkerAssignmentType::Gathering {
          if let Some(target_id) = assignment.target_unit {
            // Check if the target is a mineral (not an extractor)
            if !extractors.iter().any(|e| e.get_id() == target_id) {
              return Some(*worker_id);
            }
          }
        }
        None
      });

      if let Some(worker_id) = worker_to_reassign {
        assignments.insert(worker_id, WorkerAssignment::gathering(extractor_id));
        println!(
          "Reassigned worker {} from minerals to extractor {}",
          worker_id, extractor_id
        );
      }
    }
  }

  game_state.worker_assignments = assignments;
}

pub fn enforce_assignments(game: &Game, game_state: &mut GameState) {
  let my_units = get_my_workers(game);
  let workers: Vec<_> = my_units.iter().collect();

  let build_order = game_state.build_order.clone();
  let base_locations = game_state.base_locations.clone();

  let mut workers_to_clear: Vec<usize> = Vec::new();

  for worker in workers {
    let worker_id = worker.get_id();
    if let Some(assignment) = game_state.worker_assignments.get_mut(&worker_id) {
      match assignment.assignment_type {
        WorkerAssignmentType::Gathering => {
          let should_clear =
            enforce_gathering_assignment(game, worker, assignment);
          if should_clear {
            workers_to_clear.push(worker_id);
          }
        }
        WorkerAssignmentType::Building => {
          enforce_building_assignment(game, worker, assignment, &build_order, &base_locations);
        }
      }
    }
  }

  // Get minerals and extractors for reassignment
  let static_minerals = game.get_static_minerals();
  let minerals: Vec<_> = static_minerals.iter().collect();
  let extractors: Vec<_> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_type() == UnitType::Zerg_Extractor
        && u.get_player().get_id() == game.self_().map(|p| p.get_id()).unwrap_or(0)
        && u.is_completed()
    })
    .collect();

  // Try to reassign workers that need it
  for worker_id in &workers_to_clear {
    if !assign_worker_to_other_resource(
      game,
      *worker_id,
      &mut game_state.worker_assignments,
      &minerals,
      &extractors,
    ) {
      // If couldn't reassign, clear the assignment
      game_state.worker_assignments.remove(worker_id);
    }
  }
}

fn count_workers_per_resource(
  assignments: &HashMap<usize, WorkerAssignment>,
) -> HashMap<usize, usize> {
  assignments
    .values()
    .filter(|a| a.assignment_type == WorkerAssignmentType::Gathering)
    .filter_map(|a| a.target_unit)
    .fold(HashMap::new(), |mut acc, mineral_id| {
      *acc.entry(mineral_id).or_insert(0) += 1;
      acc
    })
}

fn remove_dead_workers(assignments: &mut HashMap<usize, WorkerAssignment>, workers: &[&Unit]) {
  let worker_ids: Vec<usize> = workers.iter().map(|w| w.get_id()).collect();
  // Don't remove building assignments - the drone may have morphed into a building
  assignments.retain(|id, assignment| {
    worker_ids.contains(id) || assignment.assignment_type == WorkerAssignmentType::Building
  });
}

fn enforce_gathering_assignment(
  game: &Game,
  worker: &Unit,
  assignment: &mut WorkerAssignment,
) -> bool {
  let Some(assigned_mineral_id) = assignment.target_unit else {
    return false;
  };

  let worker_order = worker.get_order();

  if worker_order == Order::ReturnMinerals
    || worker_order == Order::ReturnGas
    || worker_order == Order::ResetCollision
    || worker_order == Order::Move
  {
    return false;
  }

  let Some(mineral) = game.get_unit(assigned_mineral_id as usize) else {
    println!("Assigned mineral no longer exists"); // should probably handle somewhere else
    return false;
  };

  if worker_order == Order::PlayerGuard || worker_order == Order::Stop {
    match worker.gather(&mineral) {
      Ok(_) => {
        println!(
          "Worker {} was {:?}, issued gather command to resource {}",
          worker.get_id(),
          worker_order,
          assigned_mineral_id
        );
      }
      Err(e) => {
        println!(
          "Worker {} failed to gather from resource {}: {:?}",
          worker.get_id(),
          assigned_mineral_id,
          e
        );

        // If unreachable, try to assign to another resource
        if e == Error::Unreachable_Location {
          println!(
            "Worker {} cannot reach resource {}, attempting reassignment",
            worker.get_id(),
            assigned_mineral_id
          );
          return true; // Signal to clear this assignment so it can be reassigned
        }
      }
    }
    return false;
  }

  if worker_order == Order::Guard {
    let worker_pos = worker.get_position();
    let ideal_target = Position::new(worker_pos.x - 96, worker_pos.y);
    let mut move_target = ideal_target;
    if !game.is_walkable(ideal_target.to_walk_position()) {
      let mut found_walkable = false;
      for radius in 1..=10 {
        for angle_steps in 0..8 {
          let angle = (angle_steps as f32) * std::f32::consts::PI / 4.0;
          let test_x = ideal_target.x + (angle.cos() * (radius * 8) as f32) as i32;
          let test_y = ideal_target.y + (angle.sin() * (radius * 8) as f32) as i32;
          let test_pos = Position::new(test_x, test_y);
          let walk_pos = test_pos.to_walk_position();

          if walk_pos.x >= 0
            && walk_pos.y >= 0
            && walk_pos.x < 1024
            && walk_pos.y < 1024
            && game.is_walkable(walk_pos)
          {
            move_target = test_pos;
            found_walkable = true;
            break;
          }
        }
        if found_walkable {
          break;
        }
      }

      let _ = worker.move_(move_target);
      println!(
        "Worker {} guarding, moving to walkable position left of closest building",
        worker.get_id()
      );
    }
    return false;
  }

  if worker_order == Order::MoveToMinerals
    || worker_order == Order::WaitForMinerals
    || worker_order == Order::Harvest4
    || worker_order == Order::MiningMinerals
    || worker_order == Order::Harvest1
    || worker_order == Order::Harvest2
    || worker_order == Order::Harvest3
    || worker_order == Order::MoveToGas
    || worker_order == Order::WaitForGas
    || worker_order == Order::HarvestGas
  {
    let Some(target) = worker.get_order_target() else {
      // println!("Somehow moving or waiting for minerals without a target, order is: {:?}", worker_order);
      return false;
    };

    if target.get_id() != assigned_mineral_id {
      // println!("worker mining the wrong mineral patch, reissuing gather command");
      match worker.gather(&mineral) {
        Ok(_) => {}
        Err(e) => {
          if e == Error::Unreachable_Location {
            println!(
              "Worker {} cannot reach resource {}, will be reassigned",
              worker.get_id(),
              assigned_mineral_id
            );
            return true; // Signal to clear this assignment so it can be reassigned
          }
        }
      }
    }
    return false;
  }

  println!("worker with unknown order {:?}", worker_order);
  false
}

fn get_hatchery_build_position(
  game: &Game,
  worker: &Unit,
  base_locations: &[TilePosition],
  base_index: &Option<usize>,
  worker_id: usize,
) -> Option<TilePosition> {
  if let Some(idx) = base_index {
    if let Some(base_tile) = base_locations.get(*idx) {
      Some(TilePosition::new(base_tile.x, base_tile.y))
    } else {
      println!(
        "Worker {} base index {} out of bounds for hatchery (available bases: {})",
        worker_id,
        idx,
        base_locations.len()
      );
      None
    }
  } else {
    build_location_utils::get_buildable_location(
      game,
      worker,
      UnitType::Zerg_Hatchery,
      base_locations,
      *base_index,
    )
  }
}

fn get_or_find_build_position(
  game: &Game,
  worker: &Unit,
  assignment: &mut WorkerAssignment,
  building_type: UnitType,
  base_index: &Option<usize>,
  base_locations: &[TilePosition],
  worker_id: usize,
) -> Option<TilePosition> {
  match assignment.target_position {
    Some((x, y)) => Some(TilePosition::new(x, y)),
    None => {
      let tile_pos = if building_type == UnitType::Zerg_Hatchery {
        get_hatchery_build_position(game, worker, base_locations, base_index, worker_id)
      } else {
        build_location_utils::get_buildable_location(
          game,
          worker,
          building_type,
          base_locations,
          *base_index,
        )
      };

      let Some(tile_pos) = tile_pos else {
        println!(
          "Worker {} could not find valid build location for {:?} at base_index {:?}",
          worker_id, building_type, base_index
        );
        return None;
      };

      println!(
        "Worker {} found build location ({}, {}) for {:?}, setting target_position",
        worker_id, tile_pos.x, tile_pos.y, building_type
      );
      assignment.target_position = Some((tile_pos.x, tile_pos.y));
      None
    }
  }
}

fn enforce_building_assignment(
  game: &Game,
  worker: &Unit,
  assignment: &mut WorkerAssignment,
  build_order: &[BuildOrderItem],
  base_locations: &[TilePosition],
) {
  let worker_order = worker.get_order();
  let worker_id = worker.get_id();

  let Some(build_order_idx) = assignment.build_order_index else {
    println!(
      "Worker {} has building assignment but no build_order_index",
      worker_id
    );
    return;
  };

  let Some(building_item) = build_order.get(build_order_idx) else {
    println!(
      "Worker {} build_order_index {} is out of bounds (build_order length: {})",
      worker_id,
      build_order_idx,
      build_order.len()
    );
    return;
  };

  let BuildOrderItem::Unit {
    unit_type: building_type,
    base_index,
  } = building_item
  else {
    println!(
      "Worker {} build_order_index {} is not a unit (cannot build upgrades)",
      worker_id, build_order_idx
    );
    return;
  };

  if worker_order == Order::PlaceBuilding
    || worker_order == Order::ConstructingBuilding
    || worker_order == Order::DroneBuild
  {
    game.draw_text_screen(
      (0, 10),
      &format!(
        "Worker {} is already placing/constructing building",
        worker_id
      ),
    );
    return;
  }

  let Some(player) = game.self_() else {
    println!("Failed to get self player in enforce_building_assignment");
    return;
  };

  let current_minerals = player.minerals();
  let current_gas = player.gas();
  let required_minerals = building_type.mineral_price();
  let required_gas = building_type.gas_price();

  if required_minerals > current_minerals || required_gas > current_gas {
    game.draw_text_screen(
      (0, 10),
      &format!(
        "Worker {} waiting to build {:?} Minerals: {}/{} Gas: {}/{}",
        worker_id, building_type, current_minerals, required_minerals, current_gas, required_gas
      ),
    );
    return;
  }

  let Some(pos) = get_or_find_build_position(
    game,
    worker,
    assignment,
    *building_type,
    base_index,
    base_locations,
    worker_id,
  ) else {
    return;
  };

  if let Err(can_build_err) = game.can_build_here(worker, pos, *building_type, true) {
    game.draw_text_screen(
      (0, 20),
      &format!(
        "Worker {} cannot build {:?} at ({}, {}): {:?}",
        worker_id, building_type, pos.x, pos.y, can_build_err
      ),
    );
    return;
  }

  if !game
    .can_build_here(worker, pos, *building_type, true)
    .unwrap_or(false)
  {
    game.draw_text_screen(
      (0, 10),
      &format!(
        "Worker {} is exploring build position ({}, {}) for {:?}",
        worker_id, pos.x, pos.y, building_type
      ),
    );

    if worker_order != Order::Move {
      let target_position = Position::new((pos.x - 2) * 32, (pos.y + 3) * 32);
      // explore somewhere above build position
      // exactly on build position causes build to fail sometimes
      if worker.move_(target_position).is_ok() {
        println!(
          "Worker {} moving to unexplored build position ({}, {}) for {:?}",
          worker_id, pos.x, pos.y, building_type
        );
      }
    }
    return;
  }

  let build_successful = worker.build(*building_type, pos);
  match build_successful {
    Ok(true) => {
      // println!(
      //   "Worker {} successfully issued build command for {:?} at ({}, {})",
      //   worker_id, building_type, pos.x, pos.y
      // );
    }
    Ok(false) => {
      println!(
        "Worker {} build command returned false for {:?} at ({}, {})",
        worker_id, building_type, pos.x, pos.y
      );

      game.draw_text_screen(
        (0, 10),
        &format!(
          "Worker {} build command returned false | Minerals: {}/{} Gas: {}/{}",
          worker_id, current_minerals, required_minerals, current_gas, required_gas
        ),
      );
    }
    Err(e) => {
      println!(
        "Worker {} FAILED to build {:?} at ({}, {}), error: {:?}",
        worker_id, building_type, pos.x, pos.y, e
      );
      game.draw_text_screen(
        (0, 10),
        &format!(
          "Worker {} failed to build {:?} at ({}, {}), error: {:?} | Minerals: {}/{} Gas: {}/{}",
          worker_id,
          building_type,
          pos.x,
          pos.y,
          e,
          current_minerals,
          required_minerals,
          current_gas,
          required_gas
        ),
      );

      if e == Error::Invalid_Tile_Position {
        println!(
          "Worker {} got Invalid_Tile_Position, recalculating build location",
          worker_id
        );
        assignment.target_position = build_location_utils::get_buildable_location(
          game,
          worker,
          *building_type,
          base_locations,
          *base_index,
        )
        .map(|pos| (pos.x, pos.y));
      }
    }
  }
}

fn find_least_saturated_mineral<'a>(
  minerals: &[&'a Unit],
  mineral_worker_count: &HashMap<usize, usize>,
  max_workers: usize,
) -> Option<&'a Unit> {
  minerals
    .iter()
    .map(|m| {
      let count = mineral_worker_count.get(&m.get_id()).copied().unwrap_or(0);
      (m, count)
    })
    .filter(|(_, count)| *count < max_workers)
    .min_by_key(|(_, count)| *count)
    .map(|(mineral, _)| *mineral)
}

fn find_random_least_saturated_mineral<'a>(
  minerals: &[&'a Unit],
  mineral_worker_count: &HashMap<usize, usize>,
  max_workers: usize,
) -> Option<&'a Unit> {
  let eligible: Vec<_> = minerals
    .iter()
    .map(|m| {
      let count = mineral_worker_count.get(&m.get_id()).copied().unwrap_or(0);
      (m, count)
    })
    .filter(|(_, count)| *count < max_workers)
    .collect();
  
  if eligible.is_empty() {
    return None;
  }
  
  // Find minimum saturation level
  let min_count = eligible.iter().map(|(_, count)| *count).min()?;
  
  // Get all minerals with minimum saturation
  let min_saturated: Vec<_> = eligible
    .into_iter()
    .filter(|(_, count)| *count == min_count)
    .map(|(mineral, _)| *mineral)
    .collect();
  
  // Pick randomly from least saturated minerals
  min_saturated.choose(&mut rand::thread_rng()).copied()
}

fn assign_worker_to_other_resource(
  game: &Game,
  worker_id: usize,
  assignments: &mut HashMap<usize, WorkerAssignment>,
  minerals: &[&Unit],
  extractors: &[Unit],
) -> bool {
  let mineral_worker_count = count_workers_per_resource(assignments);
  
  // Get all hatcheries and lairs
  let bases: Vec<Unit> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      let unit_type = u.get_type();
      (unit_type == UnitType::Zerg_Hatchery
        || unit_type == UnitType::Zerg_Lair
        || unit_type == UnitType::Zerg_Hive)
        && u.get_player().get_id() == game.self_().map(|p| p.get_id()).unwrap_or(0)
        && u.is_completed()
    })
    .collect();
  
  // Try to find an extractor with less than 2 workers
  let undersaturated_extractor = extractors.iter().find(|extractor| {
    let extractor_id = extractor.get_id();
    let worker_count = mineral_worker_count.get(&extractor_id).copied().unwrap_or(0);
    worker_count < 2
  });
  
  if let Some(extractor) = undersaturated_extractor {
    let extractor_id = extractor.get_id();
    assignments.insert(worker_id, WorkerAssignment::gathering(extractor_id));
    println!(
      "Worker {} reassigned to extractor {} (was unreachable)",
      worker_id, extractor_id
    );
    return true;
  }
  
  // Filter minerals to only those near a base (within 12 tiles)
  let minerals_near_bases: Vec<&Unit> = minerals
    .iter()
    .filter(|mineral| {
      let mineral_pos = mineral.get_position();
      bases.iter().any(|base| {
        let base_pos = base.get_position();
        let dx = (mineral_pos.x - base_pos.x) as f32;
        let dy = (mineral_pos.y - base_pos.y) as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= 12.0 * 32.0
      })
    })
    .copied()
    .collect();
  
  // Try to find a mineral with less than 2 workers near a base (pick randomly to avoid gridlocks)
  if let Some(mineral) = find_random_least_saturated_mineral(&minerals_near_bases, &mineral_worker_count, 2) {
    let mineral_id = mineral.get_id();
    assignments.insert(worker_id, WorkerAssignment::gathering(mineral_id));
    println!(
      "Worker {} reassigned to mineral {} near base (was unreachable)",
      worker_id, mineral_id
    );
    return true;
  }
  
  false
}

pub fn draw_worker_resource_lines(
  game: &Game,
  worker_assignments: &HashMap<usize, WorkerAssignment>,
) {
  for (worker_id, assignment) in worker_assignments {
    let Some(worker) = game.get_unit(*worker_id) else {
      continue;
    };

    match assignment.assignment_type {
      WorkerAssignmentType::Gathering => {
        if let Some(mineral_id) = assignment.target_unit {
          if let Some(mineral) = game.get_unit(mineral_id) {
            let worker_pos = worker.get_position();
            let mineral_pos = mineral.get_position();
            game.draw_line_map(worker_pos, mineral_pos, Color::Cyan);
          }
        }
      }
      WorkerAssignmentType::Building => {
        if let Some((target_x, target_y)) = assignment.target_position {
          let worker_pos = worker.get_position();
          // target_position is stored in tile coordinates, convert to pixels for drawing
          let target_pos = Position::new(target_x * 32 + 16, target_y * 32 + 16);
          game.draw_line_map(worker_pos, target_pos, Color::Yellow);
        }
      }
    }
  }
}

pub fn draw_worker_ids(game: &Game) {
  let self_player = match game.self_() {
    Some(p) => p,
    None => return,
  };

  let my_units = self_player.get_units();
  let workers: Vec<_> = my_units
    .iter()
    .filter(|u| u.get_type().is_worker())
    .collect();

  for worker in workers {
    let worker_pos = worker.get_position();
    let worker_id = worker.get_id();
    game.draw_text_map(worker_pos, &format!("{}", worker_id));
  }
}

pub fn draw_building_ids(game: &Game) {
  let self_player = match game.self_() {
    Some(p) => p,
    None => return,
  };

  let my_units = self_player.get_units();
  let buildings: Vec<_> = my_units
    .iter()
    .filter(|u| u.get_type().is_building())
    .collect();

  for building in buildings {
    let building_pos = building.get_position();
    let building_id = building.get_id();
    game.draw_text_map(building_pos, &format!("{}", building_id));
  }
}
