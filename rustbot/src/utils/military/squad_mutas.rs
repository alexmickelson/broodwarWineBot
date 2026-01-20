use rsbwapi::*;

use crate::utils::{
  map_utils::pathing,
  military::{
    avoid_enemy_movement_utils::{self, ThreatAvoidanceMode},
    squad_models::MilitarySquad,
  },
};

pub fn muta_squad_control(game: &Game, squad: &mut MilitarySquad) {
  ensure_path_to_enemy(game, squad);
  initialize_target_position(squad);
  update_target_position(game, squad);
}

fn ensure_path_to_enemy(game: &Game, squad: &mut MilitarySquad) {
  if squad.target_path.is_some() {
    return;
  }

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

fn initialize_target_position(squad: &mut MilitarySquad) {
  if squad.target_position.is_some() {
    return;
  }

  println!("Setting initial muta squad target position");
  let Some(ref path) = squad.target_path else {
    return;
  };

  // Start at 1/3 of the way to the enemy
  let initial_index = path.len() / 3;
  if initial_index < path.len() {
    squad.target_position = Some(path[initial_index]);
    squad.target_path_index = Some(initial_index);
  }
}

fn update_target_position(game: &Game, squad: &mut MilitarySquad) {
  let (Some(path), Some(index)) = (squad.target_path.clone(), squad.target_path_index) else {
    return;
  };

  // Check for enemies near buildings and redirect if found
  if let Some(enemy_position) = check_enemies_near_buildings(game) {
    let squad_units: Vec<Unit> = squad
      .assigned_unit_ids
      .iter()
      .filter_map(|&unit_id| game.get_unit(unit_id))
      .collect();
    
    if let Some(target_pos) = calculate_muta_defense_position(&squad_units, enemy_position) {
      squad.target_position = Some(target_pos);
    }
    // Don't update path index when defending buildings
    return;
  }

  let squad_units: Vec<Unit> = squad
    .assigned_unit_ids
    .iter()
    .filter_map(|&unit_id| game.get_unit(unit_id))
    .collect();

  let current_target = squad.target_position.unwrap();
  let units_close_to_target = get_units_close_to_position(&squad_units, current_target, 200.0);

  // Move target towards nearby enemies if present
  adjust_target_for_nearby_enemies(game, squad, current_target);

  // Re-fetch current_target after potential adjustment
  let current_target = squad.target_position.unwrap();

  // Handle retreat if too few leading units
  if should_retreat(units_close_to_target, index, &path) {
    handle_retreat_for_catching_up_units(game, squad, &squad_units, &path, index, current_target);
    return;
  }

  // Handle advance if enough units grouped at target
  if should_advance(game, squad, &squad_units, units_close_to_target, index, &path) {
    handle_advance_for_leading_units(game, squad, &path, index);
  }
}

fn adjust_target_for_nearby_enemies(game: &Game, squad: &mut MilitarySquad, current_target: (i32, i32)) {
  let enemies_near_target = avoid_enemy_movement_utils::get_enemies_within(
    game,
    Position::new(current_target.0, current_target.1),
    500.0,
    game.self_().map_or(0, |p| p.get_id()),
  );
  
  // Filter out buildings and count real threats
  let threatening_enemies: Vec<_> = enemies_near_target
    .into_iter()
    .filter(|e| !e.get_type().is_building())
    .collect();
  
  if !threatening_enemies.is_empty() {
    // Calculate average position of enemies
    let sum_x: i32 = threatening_enemies.iter().map(|e| e.get_position().x).sum();
    let sum_y: i32 = threatening_enemies.iter().map(|e| e.get_position().y).sum();
    let count = threatening_enemies.len() as i32;
    let avg_enemy_x = sum_x / count;
    let avg_enemy_y = sum_y / count;
    
    // Move target 30% towards the enemy position
    let dx = avg_enemy_x - current_target.0;
    let dy = avg_enemy_y - current_target.1;
    let new_target_x = current_target.0 + (dx as f32 * 0.3) as i32;
    let new_target_y = current_target.1 + (dy as f32 * 0.3) as i32;
    
    squad.target_position = Some((new_target_x, new_target_y));
  }
}

fn get_leading_group_center(game: &Game, squad: &mut MilitarySquad, squad_units: &[Unit], current_target: (i32, i32)) -> (i32, i32) {
  // Check if current leader is still alive
  let current_leader = squad.leader_unit_id
    .and_then(|id| game.get_unit(id))
    .filter(|u| u.exists() && squad.assigned_unit_ids.contains(&u.get_id()));
  
  let leader_changed = current_leader.is_none() && squad.leader_unit_id.is_some();
  
  // Find the leading unit (use existing leader if alive, otherwise find closest to target)
  let leading_unit = if let Some(leader) = current_leader {
    Some(leader)
  } else {
    // Leader is dead or doesn't exist, find new leader
    let new_leader = squad_units
      .iter()
      .min_by_key(|u| {
        let pos = u.get_position();
        let dx = pos.x - current_target.0;
        let dy = pos.y - current_target.1;
        dx * dx + dy * dy
      });
    
    // Update squad leader and reset target to be close to new leader
    if let Some(leader) = new_leader {
      squad.leader_unit_id = Some(leader.get_id());
      
      // When leader changes, reset target position to be close to new leader
      if leader_changed {
        let leader_pos = leader.get_position();
        squad.target_position = Some((leader_pos.x, leader_pos.y));
      }
    }
    
    new_leader.cloned()
  };
  
  // Calculate center only from units following the leader (within 200px)
  if let Some(leader) = leading_unit {
    let leader_pos = leader.get_position();
    let proximity_radius_squared = 200.0 * 200.0;
    
    let following_leader_units: Vec<_> = squad_units
      .iter()
      .filter(|u| {
        let pos = u.get_position();
        let dx = (pos.x - leader_pos.x) as f32;
        let dy = (pos.y - leader_pos.y) as f32;
        let dist_squared = dx * dx + dy * dy;
        dist_squared <= proximity_radius_squared
      })
      .collect();
    
    if !following_leader_units.is_empty() {
      let sum_x: i32 = following_leader_units.iter().map(|u| u.get_position().x).sum();
      let sum_y: i32 = following_leader_units.iter().map(|u| u.get_position().y).sum();
      (sum_x / following_leader_units.len() as i32, sum_y / following_leader_units.len() as i32)
    } else {
      (leader_pos.x, leader_pos.y)
    }
  } else {
    // Fallback if no units
    current_target
  }
}

fn handle_retreat_for_catching_up_units(
  game: &Game,
  squad: &mut MilitarySquad,
  squad_units: &[Unit],
  path: &[(i32, i32)],
  index: usize,
  current_target: (i32, i32),
) {
  let (group_center_x, group_center_y) = get_leading_group_center(game, squad, squad_units, current_target);
  
  // Find the path index behind the current group position
  let min_retreat_index = path.len() / 3;
  let new_index = (min_retreat_index..index)
    .min_by_key(|&i| {
      let path_pos = path[i];
      let dx = path_pos.0 - group_center_x;
      let dy = path_pos.1 - group_center_y;
      dx * dx + dy * dy
    })
    .unwrap_or(index.saturating_sub(1));
  
  squad.target_path_index = Some(new_index);
  
  // Move target closer to the new index position instead of jumping to it
  let new_path_pos = path[new_index];
  
  let dx = new_path_pos.0 - current_target.0;
  let dy = new_path_pos.1 - current_target.1;
  
  // Move 30% of the way towards the new position
  let move_ratio = 0.3;
  let mut target_x = current_target.0 + (dx as f32 * move_ratio) as i32;
  let mut target_y = current_target.1 + (dy as f32 * move_ratio) as i32;
  
  // Don't let the new position go more than 200px away from the group center
  let max_distance = 200.0;
  let distance_x = (target_x - group_center_x) as f32;
  let distance_y = (target_y - group_center_y) as f32;
  let distance_from_group = (distance_x * distance_x + distance_y * distance_y).sqrt();
  
  if distance_from_group > max_distance {
    let ratio = max_distance / distance_from_group;
    target_x = group_center_x + (distance_x * ratio) as i32;
    target_y = group_center_y + (distance_y * ratio) as i32;
  }
  
  squad.target_position = Some((target_x, target_y));
}

fn handle_advance_for_leading_units(
  game: &Game,
  squad: &mut MilitarySquad,
  path: &[(i32, i32)],
  index: usize,
) {
  let new_index = (index + 1).min(path.len() - 1);
  squad.target_path_index = Some(new_index);
  
  // If we've reached the end of the path, create a patrol route through enemy buildings
  if new_index >= path.len() - 1 {
    if let Some(patrol_path) = create_building_patrol_path(game, squad) {
      squad.target_path = Some(patrol_path);
      squad.target_path_index = Some(0);
      if let Some(ref new_path) = squad.target_path {
        if !new_path.is_empty() {
          squad.target_position = Some(new_path[0]);
        }
      }
      return;
    }
  }
  
  squad.target_position = Some(path[new_index]);
}

fn create_building_patrol_path(game: &Game, squad: &MilitarySquad) -> Option<Vec<(i32, i32)>> {
  let Some(self_player) = game.self_() else {
    return None;
  };
  
  let player_id = self_player.get_id();
  
  // Get all visible enemy buildings (only units belonging to enemy players, not neutral)
  let enemy_buildings: Vec<(i32, i32)> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      let unit_player = u.get_player();
      unit_player.get_id() != player_id 
        && unit_player.is_enemy(&self_player)
        && u.get_type().is_building()
        && u.is_visible()
    })
    .map(|u| {
      let pos = u.get_position();
      (pos.x, pos.y)
    })
    .collect();
  
  if enemy_buildings.is_empty() {
    return None;
  }
  
  // Start from current squad position
  let current_pos = squad.target_position?;
  
  // Simple greedy path: visit nearest unvisited building each time
  let mut patrol_path = vec![current_pos];
  let mut remaining_buildings = enemy_buildings.clone();
  let mut current = current_pos;
  
  while !remaining_buildings.is_empty() {
    // Find nearest building to current position
    let nearest_idx = remaining_buildings
      .iter()
      .enumerate()
      .min_by_key(|(_, &(x, y))| {
        let dx = x - current.0;
        let dy = y - current.1;
        dx * dx + dy * dy
      })
      .map(|(idx, _)| idx)?;
    
    let next_building = remaining_buildings.remove(nearest_idx);
    patrol_path.push(next_building);
    current = next_building;
  }
  
  // Close the loop by returning to the first building
  if patrol_path.len() > 1 {
    patrol_path.push(patrol_path[1]);
  }
  
  Some(patrol_path)
}

fn should_retreat(units_close_to_target: usize, index: usize, path: &[(i32, i32)]) -> bool {
  units_close_to_target < 3 && index > path.len() / 3
}

fn group_units_by_proximity(units: &[Unit], max_distance: f32) -> Vec<Vec<Unit>> {
  let mut groups: Vec<Vec<Unit>> = Vec::new();
  let max_distance_squared = max_distance * max_distance;
  
  for unit in units {
    let unit_pos = unit.get_position();
    
    // Find a group this unit belongs to
    let mut found_group = false;
    for group in groups.iter_mut() {
      // Check if unit is close to any member of this group
      let is_close_to_group = group.iter().any(|member| {
        let member_pos = member.get_position();
        let dx = (unit_pos.x - member_pos.x) as f32;
        let dy = (unit_pos.y - member_pos.y) as f32;
        let distance_squared = dx * dx + dy * dy;
        distance_squared <= max_distance_squared
      });
      
      if is_close_to_group {
        group.push(unit.clone());
        found_group = true;
        break;
      }
    }
    
    // If no group found, create a new one
    if !found_group {
      groups.push(vec![unit.clone()]);
    }
  }
  
  groups
}

fn calculate_muta_defense_position(squad_units: &[Unit], enemy_position: (i32, i32)) -> Option<(i32, i32)> {
  if squad_units.is_empty() {
    return Some(enemy_position);
  }

  let muta_groups = group_units_by_proximity(squad_units, 200.0);
  
  // Find the group closest to the enemy position
  let closest_group = muta_groups
    .iter()
    .min_by_key(|group| {
      // Calculate average position of the group
      let sum_x: i32 = group.iter().map(|u| u.get_position().x).sum();
      let sum_y: i32 = group.iter().map(|u| u.get_position().y).sum();
      let avg_x = sum_x / group.len() as i32;
      let avg_y = sum_y / group.len() as i32;
      
      // Distance squared to enemy position
      let dx = avg_x - enemy_position.0;
      let dy = avg_y - enemy_position.1;
      dx * dx + dy * dy
    });
  
  if let Some(group) = closest_group {
    // Calculate center of the closest group
    let sum_x: i32 = group.iter().map(|u| u.get_position().x).sum();
    let sum_y: i32 = group.iter().map(|u| u.get_position().y).sum();
    let group_center_x = sum_x / group.len() as i32;
    let group_center_y = sum_y / group.len() as i32;
    
    // Set target position between the group and enemy, within attacking distance (mutas have range 3, so ~96 pixels)
    let dx = enemy_position.0 - group_center_x;
    let dy = enemy_position.1 - group_center_y;
    let distance = ((dx * dx + dy * dy) as f32).sqrt();
    
    let attack_range = 100.0; // Slightly more than muta range for positioning
    let ratio = (distance - attack_range) / distance;
    
    let target_x = group_center_x + (dx as f32 * ratio) as i32;
    let target_y = group_center_y + (dy as f32 * ratio) as i32;
    
    Some((target_x, target_y))
  } else {
    // Fallback if no groups found
    Some(enemy_position)
  }
}

fn check_enemies_near_buildings(game: &Game) -> Option<(i32, i32)> {
  let Some(self_player) = game.self_() else {
    return None;
  };

  let player_id = self_player.get_id();
  
  // Get all player's buildings
  let buildings: Vec<Unit> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_player().get_id() == player_id && u.get_type().is_building()
    })
    .collect();

  // Check each building for nearby enemies
  for building in buildings {
    let base_pos = building.get_position();
    let enemies = avoid_enemy_movement_utils::get_enemies_within(game, base_pos, 300.0, player_id);
    
    // Filter out workers and buildings, count real threats
    let threatening_enemies: Vec<_> = enemies
      .into_iter()
      .filter(|e| {
        let unit_type = e.get_type();
        !unit_type.is_building()
          && unit_type != UnitType::Terran_SCV
          && unit_type != UnitType::Protoss_Probe
          && unit_type != UnitType::Zerg_Drone
          && unit_type != UnitType::Zerg_Overlord
      })
      .collect();
    
    // If there are 3 or more threatening units near this building, return average enemy position
    if threatening_enemies.len() >= 3 {
      let sum_x: i32 = threatening_enemies.iter().map(|e| e.get_position().x).sum();
      let sum_y: i32 = threatening_enemies.iter().map(|e| e.get_position().y).sum();
      let count = threatening_enemies.len() as i32;
      return Some((sum_x / count, sum_y / count));
    }
  }

  None
}

fn should_advance(
  game: &Game,
  squad: &MilitarySquad,
  squad_units: &[Unit],
  units_close_to_target: usize,
  index: usize,
  path: &[(i32, i32)],
) -> bool {
  if units_close_to_target < 6 || index >= path.len() - 1 {
    return false;
  }

  let target_pos = squad.target_position.unwrap();
  let enemies_near_target = avoid_enemy_movement_utils::get_enemies_within(
    game,
    Position::new(target_pos.0, target_pos.1),
    150.0,
    game.self_().map_or(0, |p| p.get_id()),
  );

  // Filter to only count non-building enemies
  let enemy_units_near_target: Vec<_> = enemies_near_target
    .into_iter()
    .filter(|u| !u.get_type().is_building())
    .collect();

  // If most units are at target and no enemy units nearby, advance further
  units_close_to_target > squad_units.len() / 2 && enemy_units_near_target.is_empty()
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

  // Check for enemies within 300 pixels and attack them immediately
  let nearby_enemies = avoid_enemy_movement_utils::get_enemies_within(game, unit_pos, 300.0, unit.get_player().get_id());
  
  if !nearby_enemies.is_empty() {
    // Filter and prioritize enemies
    let mut prioritized_enemies: Vec<(Unit, i32)> = nearby_enemies
      .into_iter()
      .filter_map(|enemy| {
        let unit_type = enemy.get_type();
        
        // Skip eggs and larvae
        if unit_type == UnitType::Zerg_Egg || unit_type == UnitType::Zerg_Larva {
          return None;
        }
        
        let is_building = unit_type.is_building();
        let air_weapon = unit_type.air_weapon();
        let has_air_weapon = air_weapon != WeaponType::None;
        
        let is_worker = unit_type == UnitType::Terran_SCV
          || unit_type == UnitType::Protoss_Probe
          || unit_type == UnitType::Zerg_Drone;
        
        let is_healer = unit_type == UnitType::Terran_Medic
          || unit_type == UnitType::Terran_Science_Vessel;
        
        let is_static_defense = unit_type == UnitType::Terran_Bunker
          || unit_type == UnitType::Terran_Missile_Turret
          || unit_type == UnitType::Protoss_Photon_Cannon
          || unit_type == UnitType::Zerg_Sunken_Colony
          || unit_type == UnitType::Zerg_Spore_Colony;
        
        // Priority: 0 = highest (anti-air units), 1 = static defense, healers and workers, 2 = other military units, 3 = buildings
        let priority = if has_air_weapon {
          0
        } else if is_static_defense || is_healer || is_worker {
          1
        } else if !is_building {
          2
        } else {
          3
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
  }

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
  _squad: &mut MilitarySquad,
  target: (i32, i32),
) {
  let target_pos = Position::new(target.0, target.1);

  // Try threat avoidance with kiting mode (move toward target while avoiding)
  if avoid_enemy_movement_utils::handle_threat_avoidance(game, unit, Some(target), ThreatAvoidanceMode::Kiting) {
    return;
  }

  // No threats, just move directly to target
  move_to_position(unit, target_pos);
}

fn handle_combat_at_target(game: &Game, unit: &Unit, target: (i32, i32), at_end_of_path: bool) {
  let unit_pos = unit.get_position();
  let target_pos = Position::new(target.0, target.1);

  // Get all nearby enemies
  let all_enemies = avoid_enemy_movement_utils::get_enemies_within(game, unit_pos, 150.0, unit.get_player().get_id());
  
  // Filter and prioritize enemies
  let mut prioritized_enemies: Vec<(Unit, i32)> = all_enemies
    .into_iter()
    .filter_map(|enemy| {
      let unit_type = enemy.get_type();
      
      // Skip eggs and larvae
      if unit_type == UnitType::Zerg_Egg || unit_type == UnitType::Zerg_Larva {
        return None;
      }
      
      let is_building = unit_type.is_building();
      
      // Skip buildings unless we're at the end of the path
      if is_building && !at_end_of_path {
        return None;
      }
      
      let air_weapon = unit_type.air_weapon();
      let has_air_weapon = air_weapon != WeaponType::None;
      
      let is_worker = unit_type == UnitType::Terran_SCV
        || unit_type == UnitType::Protoss_Probe
        || unit_type == UnitType::Zerg_Drone;
      
      let is_healer = unit_type == UnitType::Terran_Medic
        || unit_type == UnitType::Terran_Science_Vessel;
      
      let is_static_defense = unit_type == UnitType::Terran_Bunker
        || unit_type == UnitType::Terran_Missile_Turret
        || unit_type == UnitType::Protoss_Photon_Cannon
        || unit_type == UnitType::Zerg_Sunken_Colony
        || unit_type == UnitType::Zerg_Spore_Colony;
      
      // Priority: 0 = highest (anti-air units), 1 = static defense, healers and workers, 2 = other military units, 3 = buildings
      let priority = if has_air_weapon {
        0
      } else if is_static_defense || is_healer || is_worker {
        1
      } else if !is_building {
        2
      } else {
        3
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


