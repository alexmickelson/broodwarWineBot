use rsbwapi::*;

use crate::utils::{map_utils::pathing, military::squad_models::MilitarySquad};

pub fn muta_squad_control(game: &Game, squad: &mut MilitarySquad) {
  // Get path to enemy base if we don't have one
  if squad.target_path.is_none() {
    println!("Calculating muta squad path to enemy base");
    let Some(self_player) = game.self_() else {
      return;
    };

    let start_locations: Vec<ScaledPosition<32>> = game.get_start_locations();
    let Some(my_starting_position) = start_locations.get(self_player.get_id() as usize) else {
      return;
    };

    let Some(enemy_location) = start_locations
      .iter()
      .find(|&loc| loc != my_starting_position)
    else {
      return;
    };

    let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
    let enemy_pos = (enemy_location.x * 32, enemy_location.y * 32);

    let path_to_enemy = pathing::get_path_between_points(game, my_pos, enemy_pos, Some(true));
    squad.target_path = path_to_enemy;
  }

  // Pick initial target position from path
  if squad.target_position.is_none() {
    println!("Setting initial muta squad target position");
    if let Some(ref path) = squad.target_path {
      // Start at 1/3 of the way to the enemy
      let initial_index = path.len() / 3;
      if initial_index < path.len() {
        squad.target_position = Some(path[initial_index]);
        squad.target_path_index = Some(initial_index);
      }
    }
  }

  // Advance towards enemy base if units are close to current target
  if let (Some(ref path), Some(index)) = (&squad.target_path, squad.target_path_index) {
    if index < path.len() - 1 {
      let squad_units: Vec<Unit> = squad
        .assigned_unit_ids
        .iter()
        .filter_map(|&unit_id| game.get_unit(unit_id))
        .collect();

      let units_close_to_target =
        get_units_close_to_position(&squad_units, squad.target_position.unwrap(), 100.0);

      // Check for enemy units near the target
      let target_pos = squad.target_position.unwrap();
      let enemies_near_target = get_enemies_within(
        game,
        Position::new(target_pos.0, target_pos.1),
        150.0,
        game.self_().map_or(0, |p| p.get_id() as i32),
      );

      // Filter to only count non-building enemies
      let enemy_units_near_target: Vec<_> = enemies_near_target
        .into_iter()
        .filter(|u| !u.get_type().is_building())
        .collect();

      // If most units are at target and no enemy units nearby, advance further
      if units_close_to_target > squad_units.len() / 2 && enemy_units_near_target.is_empty() {
        let new_index = (index + 1).min(path.len() - 1);
        squad.target_path_index = Some(new_index);
        squad.target_position = Some(path[new_index]);
      }
    }
  }
}

fn get_units_close_to_position(units: &[Unit], position: (i32, i32), radius: f32) -> usize {
  let pos = Position::new(position.0, position.1);
  let radius_squared = radius * radius;

  units
    .iter()
    .filter(|u| {
      let unit_pos = u.get_position();
      let dx = (unit_pos.x - pos.x) as f32;
      let dy = (unit_pos.y - pos.y) as f32;
      let distance_squared = dx * dx + dy * dy;
      distance_squared <= radius_squared
    })
    .count()
}

pub fn muta_unit_control(game: &Game, unit: &Unit, squad: &mut MilitarySquad) {
  let Some((target_x, target_y)) = squad.target_position else {
    return;
  };

  let unit_pos = unit.get_position();

  // Calculate distance to target
  let dx = (unit_pos.x - target_x) as f32;
  let dy = (unit_pos.y - target_y) as f32;
  let distance_to_target = (dx * dx + dy * dy).sqrt();

  // Check if we're at the end of the path
  let at_end_of_path = if let (Some(ref path), Some(index)) = (&squad.target_path, squad.target_path_index) {
    index >= path.len() - 1
  } else {
    false
  };

  // If far from target, handle movement
  if distance_to_target > 150.0 {
    handle_movement_to_target(game, unit, squad, (target_x, target_y));
    return;
  }

  // If close to target, handle combat
  handle_combat_at_target(game, unit, (target_x, target_y), at_end_of_path);
}

fn handle_movement_to_target(
  game: &Game,
  unit: &Unit,
  squad: &mut MilitarySquad,
  target: (i32, i32),
) {
  let unit_pos = unit.get_position();
  let unit_id = unit.get_id();
  let target_pos = Position::new(target.0, target.1);

  // Check for nearby enemies
  let nearby_enemies = get_enemies_within(game, unit_pos, 200.0, unit.get_player().get_id() as i32);

  if nearby_enemies.is_empty() {
    // No enemies, just move directly to target
    move_to_position(unit, target_pos);
    // Clear any existing path
    squad.unit_path_assignments.remove(&unit_id);
  } else {
    // Enemies nearby, use pathfinding to avoid them
    let path_result = get_or_create_path_for_unit(game, unit, squad, target);

    if let Some((path, current_index)) = path_result {
      follow_path_to_target(unit, &path, current_index, squad);
    } else {
      // Fallback: move directly if pathfinding fails
      move_to_position(unit, target_pos);
    }
  }
}

fn get_or_create_path_for_unit(
  game: &Game,
  unit: &Unit,
  squad: &mut MilitarySquad,
  target: (i32, i32),
) -> Option<(Vec<(i32, i32)>, usize)> {
  let unit_id = unit.get_id();
  let unit_pos = unit.get_position();

  // Check if we already have a path for this unit
  if let Some((existing_path, current_index)) = squad.unit_path_assignments.get(&unit_id) {
    // Check if we're still following a valid path
    if *current_index < existing_path.len() {
      return Some((existing_path.clone(), *current_index));
    }
  }

  // Create a new path avoiding enemies
  let new_path = pathing::get_path_avoiding_enemies(
    game,
    (unit_pos.x, unit_pos.y),
    target,
    true, // is_flier
    unit.get_player().get_id(),
  );

  if let Some(path) = new_path {
    squad
      .unit_path_assignments
      .insert(unit_id, (path.clone(), 0));
    return Some((path, 0));
  }

  None
}

fn follow_path_to_target(
  unit: &Unit,
  path: &[(i32, i32)],
  current_index: usize,
  squad: &mut MilitarySquad,
) {
  let unit_id = unit.get_id();
  let unit_pos = unit.get_position();

  if current_index >= path.len() {
    squad.unit_path_assignments.remove(&unit_id);
    return;
  }

  // Look ahead 10 points in the path
  let target_index = (current_index + 10).min(path.len() - 1);
  let target_point = path[target_index];
  let target_pos = Position::new(target_point.0, target_point.1);

  // Check if we're close to the current target point
  let dx = (unit_pos.x - target_point.0) as f32;
  let dy = (unit_pos.y - target_point.1) as f32;
  let distance = (dx * dx + dy * dy).sqrt();

  if distance < 50.0 {
    // Close to target point, advance to next segment
    let new_index = target_index;
    squad
      .unit_path_assignments
      .insert(unit_id, (path.to_vec(), new_index));
  }

  move_to_position(unit, target_pos);
}

fn handle_combat_at_target(game: &Game, unit: &Unit, target: (i32, i32), at_end_of_path: bool) {
  let unit_pos = unit.get_position();
  let target_pos = Position::new(target.0, target.1);

  // Get all nearby enemies
  let all_enemies = get_enemies_within(game, unit_pos, 150.0, unit.get_player().get_id() as i32);
  
  // Filter and prioritize enemies
  let mut prioritized_enemies: Vec<(Unit, i32)> = all_enemies
    .into_iter()
    .filter_map(|enemy| {
      let unit_type = enemy.get_type();
      let is_building = unit_type.is_building();
      
      // Skip buildings unless we're at the end of the path
      if is_building && !at_end_of_path {
        return None;
      }
      
      let air_weapon = unit_type.air_weapon();
      let has_air_weapon = air_weapon != WeaponType::None;
      
      // Priority: 0 = highest (anti-air units), 1 = units without anti-air, 2 = buildings
      let priority = if has_air_weapon {
        0
      } else if !is_building {
        1
      } else {
        2
      };
      
      Some((enemy, priority))
    })
    .collect();
  
  // Sort by priority (lower number = higher priority)
  prioritized_enemies.sort_by_key(|(_, priority)| *priority);
  
  // Attack highest priority target if available
  if let Some((target_enemy, _)) = prioritized_enemies.first() {
    attack_unit_if_needed(unit, target_enemy);
    return;
  }

  // No enemies nearby, attack move to target position
  attack_move_to_position(unit, target_pos);
}

fn move_to_position(unit: &Unit, target_pos: Position) {
  let unit_order = unit.get_order();
  let order_target = unit.get_order_target_position();

  if unit_order != Order::Move || order_target != Some(target_pos) {
    let _ = unit.move_(target_pos);
  }
}

fn attack_move_to_position(unit: &Unit, target_pos: Position) {
  let unit_order = unit.get_order();
  let order_target = unit.get_order_target_position();

  if unit_order != Order::AttackMove || order_target != Some(target_pos) {
    let _ = unit.attack(target_pos);
  }
}

fn attack_unit_if_needed(unit: &Unit, target: &Unit) {
  let unit_order = unit.get_order();
  let order_target = unit.get_target();

  let mut already_attacking_valid_target = false;
  let order_target_id = order_target.and_then(|u: Unit| {
    let id = u.get_id();
    if id < 10000 {
      if unit_order == Order::AttackUnit {
        already_attacking_valid_target = true;
      }
      Some(id)
    } else {
      None
    }
  });

  if !already_attacking_valid_target {
    if unit_order != Order::AttackUnit || order_target_id != Some(target.get_id()) {
      let _ = unit.attack(target);
    }
  }
}

fn get_enemies_within(game: &Game, position: Position, radius: f32, player_id: i32) -> Vec<Unit> {
  let radius_squared = radius * radius;

  game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      if u.get_player().get_id() as i32 == player_id {
        return false;
      }

      if !u.exists() {
        return false;
      }

      let enemy_pos = u.get_position();
      let dx = (enemy_pos.x - position.x) as f32;
      let dy = (enemy_pos.y - position.y) as f32;
      let distance_squared = dx * dx + dy * dy;

      distance_squared <= radius_squared
    })
    .collect()
}
