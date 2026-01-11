use crate::utils::game_state::{SharedGameState, WorkerAssignment, WorkerAssignmentType};
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

pub fn update_assignments(game: &Game, game_state: &SharedGameState) {
  let my_units = get_my_workers(game);
  let workers: Vec<_> = my_units.iter().collect();

  let mut assignments = if let Ok(game_state_lock) = game_state.lock() {
    game_state_lock.worker_assignments.clone()
  } else {
    return;
  };

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

  let unassigned_idle_workers: Vec<_> = workers
    .iter()
    .filter(|w| w.is_idle() && !assignments.contains_key(&w.get_id()))
    .copied()
    .collect();

  let mut mineral_worker_count = count_workers_per_resource(&assignments);

  for worker in unassigned_idle_workers {
    let best_mineral = find_least_saturated_mineral(&minerals, &mineral_worker_count, 2)
      .or_else(|| find_least_saturated_mineral(&minerals, &mineral_worker_count, 3));

    if let Some(mineral) = best_mineral {
      assign_to_minerals(
        game,
        worker,
        mineral,
        &mut assignments,
        &mut mineral_worker_count,
      );
    } else {
      assign_to_scout(game, worker, &mut assignments);
    }
  }

  if let Ok(mut game_state_lock) = game_state.lock() {
    game_state_lock.worker_assignments = assignments;
  }
}

pub fn enforce_assignments(game: &Game, game_state: &SharedGameState) {
  let my_units = get_my_workers(game);
  let workers: Vec<_> = my_units.iter().collect();

  let (assignments, build_order) = if let Ok(game_state_lock) = game_state.lock() {
    (
      game_state_lock.worker_assignments.clone(),
      game_state_lock.build_order.clone(),
    )
  } else {
    return;
  };

  for worker in workers {
    let worker_id = worker.get_id();
    if let Some(assignment) = assignments.get(&worker_id) {
      match assignment.assignment_type {
        WorkerAssignmentType::Gathering => {
          enforce_gathering_assignment(game, worker, assignment);
        }
        WorkerAssignmentType::Scouting => {
          enforce_scouting_assignment(worker, assignment);
        }
        WorkerAssignmentType::Building => {
          enforce_building_assignment(game, worker, assignment, &build_order);
        }
      }
    }
  }
}

fn assign_to_minerals(
  game: &Game,
  worker: &Unit,
  mineral: &Unit,
  assignments: &mut HashMap<usize, WorkerAssignment>,
  mineral_worker_count: &mut HashMap<usize, usize>,
) {
  let mineral_id = mineral.get_id();

  if let Err(e) = worker.gather(mineral) {
    game.draw_text_screen((10, 50), &format!("Worker gather error: {:?}", e));
  } else {
    assignments.insert(worker.get_id(), WorkerAssignment::gathering(mineral_id));
    *mineral_worker_count.entry(mineral_id).or_insert(0) += 1;
  }
}

fn assign_to_scout(game: &Game, worker: &Unit, assignments: &mut HashMap<usize, WorkerAssignment>) {
  let map_width = game.map_width();
  let map_height = game.map_height();

  use std::time::{SystemTime, UNIX_EPOCH};
  let seed = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as usize;
  let random_x = ((seed + worker.get_id()) % map_width as usize) as i32 * 32;
  let random_y = ((seed * 7 + worker.get_id() * 11) % map_height as usize) as i32 * 32;
  let scout_position = (random_x, random_y);

  if let Err(e) = worker.move_(Position::new(random_x, random_y)) {
    game.draw_text_screen((10, 70), &format!("Worker scout error: {:?}", e));
  } else {
    assignments.insert(worker.get_id(), WorkerAssignment::scouting(scout_position));
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
  assignments.retain(|id, _| worker_ids.contains(id));
}

fn enforce_gathering_assignment(game: &Game, worker: &Unit, assignment: &WorkerAssignment) {
  let Some(assigned_mineral_id) = assignment.target_unit else {
    println!("No target unit for gathering assignment");
    return;
  };

  let worker_order = worker.get_order();

  if worker_order == Order::ReturnMinerals || worker_order == Order::ResetCollision {
    return;
  }

  let Some(mineral) = game.get_unit(assigned_mineral_id) else {
    println!("Assigned mineral no longer exists"); // should probably handle somewhere else
    return;
  };

  if worker_order == Order::MoveToMinerals
    || worker_order == Order::WaitForMinerals
    || worker_order == Order::Harvest4
  {
    let Some(target) = worker.get_order_target() else {
      println!("Somehow moving or waiting for minerals without a target");
      return;
    };

    if target.get_id() != assigned_mineral_id {
      println!("worker mining the wrong mineral patch, reissuing gather command");
      let _ = worker.gather(&mineral);
    }
    return;
  }
}

fn enforce_scouting_assignment(worker: &Unit, assignment: &WorkerAssignment) {
  if let Some((target_x, target_y)) = assignment.target_position {
    let worker_pos = worker.get_position();
    let distance =
      (((worker_pos.x - target_x).pow(2) + (worker_pos.y - target_y).pow(2)) as f32).sqrt() as i32;

    if distance < 100 || worker.is_idle() {
      let _ = worker.move_(Position::new(target_x, target_y));
    }
  }
}

fn enforce_building_assignment(
  game: &Game,
  worker: &Unit,
  assignment: &WorkerAssignment,
  build_order: &[UnitType],
) {
  let worker_order = worker.get_order();

  let Some(build_order_idx) = assignment.build_order_index else {
    println!(
      "Worker {} has building assignment but no build_order_index",
      worker.get_id()
    );
    return;
  };

  let Some((build_x, build_y)) = assignment.target_position else {
    println!(
      "Worker {} has building assignment but no target_position",
      worker.get_id()
    );
    return;
  };

  let Some(&building_type) = build_order.get(build_order_idx) else {
    println!(
      "Worker {} build_order_index {} is out of bounds (build_order length: {})",
      worker.get_id(),
      build_order_idx,
      build_order.len()
    );
    return;
  };

  if worker_order == Order::PlaceBuilding || worker_order == Order::ConstructingBuilding {
    return;
  }

  let desired_pos = TilePosition::new(build_x / 32, build_y / 32);

  let build_pos = find_valid_build_location(game, worker, building_type, desired_pos, 10);

  let Some(pos) = build_pos else {
    println!(
      "Worker {} could not find valid build location for {:?} near ({}, {})",
      worker.get_id(),
      building_type,
      build_x,
      build_y
    );
    return;
  };

  let build_successful = worker.build(building_type, pos);
  if !build_successful.is_ok() {
    println!(
      "Worker {} failed to issue build command for {:?} at ({}, {})",
      worker.get_id(),
      building_type,
      pos.x,
      pos.y
    );
  }
}

fn find_valid_build_location(
  game: &Game,
  builder: &Unit,
  building_type: UnitType,
  desired_pos: TilePosition,
  search_radius: i32,
) -> Option<TilePosition> {
  if game
    .can_build_here(builder, desired_pos, building_type, true)
    .unwrap_or(false)
  {
    return Some(desired_pos);
  }

  let location = (1..=search_radius)
    .flat_map(|radius| {
      (-radius..=radius).flat_map(move |dy| {
        (-radius..=radius).filter_map(move |dx| {
          Some(TilePosition {
            x: desired_pos.x + dx,
            y: desired_pos.y + dy,
          })
        })
      })
    })
    .find(|&tile_pos| {
      game
        .can_build_here(builder, tile_pos, building_type, true)
        .unwrap_or(false)
        && !has_adjacent_buildings(game, tile_pos, building_type)
        && !blocks_resource_path(game, tile_pos, building_type)
    });

  location
}

fn has_adjacent_buildings(game: &Game, pos: TilePosition, building_type: UnitType) -> bool {
  let width = building_type.tile_width();
  let height = building_type.tile_height();

  // Check all units on the map
  for unit in game.get_all_units() {
    let unit_type = unit.get_type();

    // Skip if not a building, or if it's a resource depot (we allow those to be adjacent)
    if !unit_type.is_building() || unit_type.is_resource_depot() {
      continue;
    }

    // Get the building's tile position and dimensions
    let unit_tile_pos = unit.get_tile_position();
    let unit_width = unit_type.tile_width() as i32;
    let unit_height = unit_type.tile_height() as i32;

    // Check if there's any overlap or adjacency between the two buildings
    // Buildings are adjacent if they're within 1 tile of each other
    let horizontal_gap = if pos.x + width as i32 <= unit_tile_pos.x {
      unit_tile_pos.x - (pos.x + width as i32)
    } else if unit_tile_pos.x + unit_width <= pos.x {
      pos.x - (unit_tile_pos.x + unit_width)
    } else {
      -1 // Overlapping in X dimension
    };

    let vertical_gap = if pos.y + height as i32 <= unit_tile_pos.y {
      unit_tile_pos.y - (pos.y + height as i32)
    } else if unit_tile_pos.y + unit_height <= pos.y {
      pos.y - (unit_tile_pos.y + unit_height)
    } else {
      -1 // Overlapping in Y dimension
    };

    // If both gaps are <= 0, the buildings are adjacent or overlapping
    if horizontal_gap <= 0 && vertical_gap <= 0 {
      return true;
    }
  }

  false
}

fn blocks_resource_path(game: &Game, pos: TilePosition, building_type: UnitType) -> bool {
  let Some(player) = game.self_() else {
    return false;
  };

  // Find resource depots (bases)
  let player_units = player.get_units();
  let resource_depots: Vec<_> = player_units
    .iter()
    .filter(|u| u.get_type().is_resource_depot())
    .collect();

  if resource_depots.is_empty() {
    return false;
  }

  // Get all resources near our bases
  let all_minerals = game.get_static_minerals();
  let all_geysers = game.get_static_geysers();

  let width = building_type.tile_width() as i32;
  let height = building_type.tile_height() as i32;

  // Convert building bounds to pixel coordinates for easier collision checking
  let build_left = pos.x * 32;
  let build_top = pos.y * 32;
  let build_right = build_left + width * 32;
  let build_bottom = build_top + height * 32;
  let build_center_x = (build_left + build_right) / 2;
  let build_center_y = (build_top + build_bottom) / 2;

  for depot in resource_depots {
    let depot_pos = depot.get_position();

    // Check minerals
    for mineral in all_minerals.iter() {
      let mineral_pos = mineral.get_position();

      // Only check resources reasonably close to this depot
      let depot_to_mineral_dx = (mineral_pos.x - depot_pos.x) as f32;
      let depot_to_mineral_dy = (mineral_pos.y - depot_pos.y) as f32;
      let depot_to_mineral_dist = (depot_to_mineral_dx * depot_to_mineral_dx
        + depot_to_mineral_dy * depot_to_mineral_dy)
        .sqrt();

      if depot_to_mineral_dist > 10.0 * 32.0 {
        continue; // Too far, not part of this base
      }

      // Check if building is between depot and resource
      if is_point_between(
        depot_pos.x,
        depot_pos.y,
        mineral_pos.x,
        mineral_pos.y,
        build_center_x,
        build_center_y,
        64.0,
      ) {
        return true;
      }
    }

    // Check geysers
    for geyser in all_geysers.iter() {
      let geyser_pos = geyser.get_position();

      let depot_to_geyser_dx = (geyser_pos.x - depot_pos.x) as f32;
      let depot_to_geyser_dy = (geyser_pos.y - depot_pos.y) as f32;
      let depot_to_geyser_dist =
        (depot_to_geyser_dx * depot_to_geyser_dx + depot_to_geyser_dy * depot_to_geyser_dy).sqrt();

      if depot_to_geyser_dist > 12.0 * 32.0 {
        continue;
      }

      if is_point_between(
        depot_pos.x,
        depot_pos.y,
        geyser_pos.x,
        geyser_pos.y,
        build_center_x,
        build_center_y,
        64.0,
      ) {
        return true;
      }
    }
  }

  false
}

fn is_point_between(x1: i32, y1: i32, x2: i32, y2: i32, px: i32, py: i32, threshold: f32) -> bool {
  // Check if point (px, py) is roughly between points (x1, y1) and (x2, y2)
  // using perpendicular distance from the line

  let line_dx = (x2 - x1) as f32;
  let line_dy = (y2 - y1) as f32;
  let line_length_sq = line_dx * line_dx + line_dy * line_dy;

  if line_length_sq < 1.0 {
    return false; // Points are basically the same
  }

  // Calculate the perpendicular distance from point to line
  let t = ((px - x1) as f32 * line_dx + (py - y1) as f32 * line_dy) / line_length_sq;

  // Check if the point projects onto the line segment (not beyond the endpoints)
  if t < 0.0 || t > 1.0 {
    return false;
  }

  // Calculate the closest point on the line segment
  let closest_x = x1 as f32 + t * line_dx;
  let closest_y = y1 as f32 + t * line_dy;

  // Calculate distance from point to the line
  let dist_dx = px as f32 - closest_x;
  let dist_dy = py as f32 - closest_y;
  let distance = (dist_dx * dist_dx + dist_dy * dist_dy).sqrt();

  // If the distance is less than threshold, it's blocking the path
  distance < threshold
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
    if assignment.assignment_type != WorkerAssignmentType::Gathering {
      continue;
    }

    if let Some(worker) = game.get_unit(*worker_id) {
      if let Some(mineral_id) = assignment.target_unit {
        if let Some(mineral) = game.get_unit(mineral_id) {
          let worker_pos = worker.get_position();
          let mineral_pos = mineral.get_position();
          game.draw_line_map(worker_pos, mineral_pos, Color::Cyan);
        }
      }
    }
  }

  // let my_units = self_player.get_units();
  // let workers: Vec<_> = my_units
  //   .iter()
  //   .filter(|u| u.get_type().is_worker())
  //   .collect();

  // for worker in workers {
  //   let worker_pos = worker.get_position();

  //   if let Some(target) = worker.get_target() {
  //     if is_resource(target.get_type()) {
  //       game.draw_line_map(worker_pos, target.get_position(), Color::Cyan);
  //     }
  //   }

  //   if let Some(order_target) = worker.get_order_target() {
  //     if is_resource(order_target.get_type()) {
  //       game.draw_line_map(worker_pos, order_target.get_position(), Color::Yellow);
  //     }
  //   }
  // }
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
