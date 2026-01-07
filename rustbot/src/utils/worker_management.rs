use crate::utils::game_status::{SharedStatus, WorkerAssignment, WorkerAssignmentType};
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

pub fn update_assignments(game: &Game, status: &SharedStatus) {
  let my_units = get_my_workers(game);
  let workers: Vec<_> = my_units.iter().collect();

  let mut assignments = if let Ok(status) = status.lock() {
    status.worker_assignments.clone()
  } else {
    return;
  };

  remove_dead_workers(&mut assignments, &workers);

  let static_minerals = game.get_static_minerals();
  let minerals: Vec<_> = static_minerals
    .iter()
    .filter(|m: &&Unit| m.exists() && m.get_resources() > 0)
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

  if let Ok(mut status) = status.lock() {
    status.worker_assignments = assignments;
  }
}

pub fn enforce_assignments(game: &Game, status: &SharedStatus) {
  let my_units = get_my_workers(game);
  let workers: Vec<_> = my_units.iter().collect();

  let assignments = if let Ok(status) = status.lock() {
    status.worker_assignments.clone()
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
          enforce_building_assignment(worker);
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
  if let Some(assigned_mineral_id) = assignment.target_unit {
    let correct_target = worker
      .get_target()
      .map(|t| t.get_id() == assigned_mineral_id)
      .unwrap_or(false)
      || worker
        .get_order_target()
        .map(|t| t.get_id() == assigned_mineral_id)
        .unwrap_or(false);

    if !correct_target && worker.is_idle() {
      if let Some(mineral) = game.get_unit(assigned_mineral_id) {
        if mineral.exists() && mineral.get_resources() > 0 {
          let _ = worker.gather(&mineral);
        }
      }
    }
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

fn enforce_building_assignment(_worker: &Unit) {
  // Building workers manage themselves, no enforcement needed
}

fn is_resource(unit_type: UnitType) -> bool {
  unit_type.is_mineral_field() || unit_type == UnitType::Resource_Vespene_Geyser
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

pub fn draw_worker_resource_lines(game: &Game) {
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

    if let Some(target) = worker.get_target() {
      if is_resource(target.get_type()) {
        game.draw_line_map(worker_pos, target.get_position(), Color::Cyan);
      }
    }

    if let Some(order_target) = worker.get_order_target() {
      if is_resource(order_target.get_type()) {
        game.draw_line_map(worker_pos, order_target.get_position(), Color::Yellow);
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
