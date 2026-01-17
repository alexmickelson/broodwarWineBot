use crate::utils::{
  building_stuff::build_location_utils,
  game_state::{BuildOrderItem, GameState, WorkerAssignment, WorkerAssignmentType},
};
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

  let start_location = game
    .self_()
    .and_then(|p: Player| {
      let start_locations = game.get_start_locations();
      start_locations.get(p.get_id() as usize).copied()
    })
    .expect("Failed to get start location");

  let static_minerals = game.get_static_minerals();
  let minerals: Vec<_> = static_minerals
    .iter()
    .filter(|m: &&Unit| {
      let start_pos = Position::new(start_location.x * 32 + 16, start_location.y * 32 + 16);
      let mineral_pos = m.get_position();
      let dx = (mineral_pos.x - start_pos.x) as f32;
      let dy = (mineral_pos.y - start_pos.y) as f32;
      let distance = (dx * dx + dy * dy).sqrt();
      distance <= 10.0 * 32.0
    })
    .collect();

  let extractors: Vec<_> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_type() == UnitType::Zerg_Extractor
        && u.get_player().get_id() == game.self_().map(|p| p.get_id()).unwrap_or(0)
        && u.is_completed()
    })
    .collect();

  let unassigned_idle_workers: Vec<_> = workers
    .iter()
    .filter(|w| w.is_idle() && !assignments.contains_key(&w.get_id()))
    .copied()
    .collect();

  let mut mineral_worker_count = count_workers_per_resource(&assignments);

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
      worker_count < 2
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

    if worker_count < 2 {
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

  for worker in workers {
    let worker_id = worker.get_id();
    if let Some(assignment) = game_state.worker_assignments.get_mut(&worker_id) {
      match assignment.assignment_type {
        WorkerAssignmentType::Gathering => {
          enforce_gathering_assignment(game, worker, assignment);
        }
        WorkerAssignmentType::Building => {
          enforce_building_assignment(game, worker, assignment, &build_order);
        }
      }
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

fn enforce_gathering_assignment(game: &Game, worker: &Unit, assignment: &WorkerAssignment) {
  let Some(assigned_mineral_id) = assignment.target_unit else {
    println!("No target unit for gathering assignment");
    return;
  };

  let worker_order = worker.get_order();

  if worker_order == Order::ReturnMinerals
    || worker_order == Order::ReturnGas
    || worker_order == Order::ResetCollision
  {
    return;
  }

  let Some(mineral) = game.get_unit(assigned_mineral_id) else {
    println!("Assigned mineral no longer exists"); // should probably handle somewhere else
    return;
  };

  if worker_order == Order::PlayerGuard || worker_order == Order::Stop {
    let _ = worker.gather(&mineral);
    return;
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
      return;
    };

    if target.get_id() != assigned_mineral_id {
      // println!("worker mining the wrong mineral patch, reissuing gather command");
      let _ = worker.gather(&mineral);
    }
    return;
  }

  println!("worker with unknown order {:?}", worker_order);
}

fn enforce_building_assignment(
  game: &Game,
  worker: &Unit,
  assignment: &mut WorkerAssignment,
  build_order: &[BuildOrderItem],
) {
  let worker_order = worker.get_order();

  let Some(build_order_idx) = assignment.build_order_index else {
    println!(
      "Worker {} has building assignment but no build_order_index",
      worker.get_id()
    );
    return;
  };

  let Some(building_item) = build_order.get(build_order_idx) else {
    println!(
      "Worker {} build_order_index {} is out of bounds (build_order length: {})",
      worker.get_id(),
      build_order_idx,
      build_order.len()
    );
    return;
  };

  let BuildOrderItem::Unit(building_type) = building_item else {
    println!(
      "Worker {} build_order_index {} is not a unit (cannot build upgrades)",
      worker.get_id(),
      build_order_idx
    );
    return;
  };

  if worker_order == Order::PlaceBuilding || worker_order == Order::ConstructingBuilding {
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
        worker.get_id(),
        building_type,
        current_minerals,
        required_minerals,
        current_gas,
        required_gas
      ),
    );
    return;
  }

  let pos = match assignment.target_position {
    Some((x, y)) => TilePosition::new(x, y),
    None => {
      let Some(p) = build_location_utils::get_buildable_location(game, worker, *building_type)
      else {
        println!(
          "Worker {} could not find valid build location for {:?}",
          worker.get_id(),
          building_type
        );
        return;
      };
      p
    }
  };

  let build_successful = worker.build(*building_type, pos);
  if let Err(e) = build_successful {
    game.draw_text_screen(
      (0, 10),
      &format!(
        "Worker {} failed to build {:?} at ({}, {}), error: {:?} | Minerals: {}/{} Gas: {}/{}",
        worker.get_id(),
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
        worker.get_id()
      );
      assignment.target_position =
        build_location_utils::get_buildable_location(game, worker, *building_type)
          .map(|pos| (pos.x, pos.y));
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
          let target_pos = Position::new(target_x, target_y);
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
