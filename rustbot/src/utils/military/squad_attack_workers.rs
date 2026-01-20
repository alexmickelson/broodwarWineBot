use crate::utils::{
  map_utils::pathing,
  military::{
    avoid_enemy_movement_utils::{self, ThreatAvoidanceMode},
    squad_models::{MilitarySquad, SquadRole, SquadStatus},
  },
};
use rsbwapi::*;

fn get_player_start_location(game: &Game, player: &Player) -> Option<TilePosition> {
  // Get all starting locations from the map
  let start_locations = game.get_start_locations();


  
  // Get player's resource depots (starting base)
  let resource_depots: Vec<Unit> = player
    .get_units()
    .iter()
    .filter(|u| u.get_type().is_resource_depot())
    .cloned()
    .collect();
  
  if let Some(depot) = resource_depots.first() {
    // Find the start location closest to the player's first depot
    let depot_tile = depot.get_tile_position();
    start_locations
      .iter()
      .min_by_key(|&&start_loc| {
        let dx = start_loc.x - depot_tile.x;
        let dy = start_loc.y - depot_tile.y;
        dx * dx + dy * dy
      })
      .copied()
  } else {
    // If no depot exists yet, can't determine starting location
    None
  }
}

fn get_average_resource_position_near_location(
  game: &Game,
  location: &TilePosition,
) -> (i32, i32) {
  const SEARCH_RADIUS: i32 = 800;
  let location_pixels = (location.x * 32, location.y * 32);
  let mut sum_x = 0;
  let mut sum_y = 0;
  let mut count = 0;

  // Check minerals
  for unit in game.get_static_minerals() {
    let unit_pos = unit.get_position();
    let dist_x = (unit_pos.x - location_pixels.0).abs();
    let dist_y = (unit_pos.y - location_pixels.1).abs();
    if dist_x <= SEARCH_RADIUS && dist_y <= SEARCH_RADIUS {
      sum_x += unit_pos.x;
      sum_y += unit_pos.y;
      count += 1;
    }
  }

  // Check geysers
  for unit in game.get_static_geysers() {
    let unit_pos = unit.get_position();
    let dist_x = (unit_pos.x - location_pixels.0).abs();
    let dist_y = (unit_pos.y - location_pixels.1).abs();
    if dist_x <= SEARCH_RADIUS && dist_y <= SEARCH_RADIUS {
      sum_x += unit_pos.x;
      sum_y += unit_pos.y;
      count += 1;
    }
  }

  if count > 0 {
    (sum_x / count, sum_y / count)
  } else {
    location_pixels
  }
}

pub fn attack_workers_squad(game: &Game, self_player: &Player) -> MilitarySquad {
  let start_locations: Vec<ScaledPosition<32>> = game.get_start_locations();
  let Some(my_starting_position) = get_player_start_location(game, self_player) else {
    return MilitarySquad {
      name: "Main Squad".to_string(),
      role: SquadRole::AttackWorkers,
      status: SquadStatus::Gathering,
      assigned_unit_ids: std::collections::HashSet::new(),
      target_position: None,
      target_path: None,
      target_path_index: None,
      leader_unit_id: None,
      unit_path_assignments: std::collections::HashMap::new(),
    };
  };

  let Some(enemy_location) = start_locations
    .iter()
    .find(|&&loc| loc != my_starting_position)
  else {
    return MilitarySquad {
      name: "Main Squad".to_string(),
      role: SquadRole::AttackWorkers,
      status: SquadStatus::Gathering,
      assigned_unit_ids: std::collections::HashSet::new(),
      target_position: None,
      target_path: None,
      target_path_index: None,
      leader_unit_id: None,
      unit_path_assignments: std::collections::HashMap::new(),
    };
  };

  let average_position_of_minerals_near_enemy_location =
    get_average_resource_position_near_location(game, enemy_location);

  let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
  let enemy_pos = average_position_of_minerals_near_enemy_location;

  let path_to_enemy = pathing::get_path_between_points(game, my_pos, enemy_pos, Some(false));

  let goal = if let Some(ref path) = path_to_enemy {
    // path.len() / 5
    path.len() / 2
  } else {
    println!("No path to enemy found when creating initial squad");
    0
  };

  MilitarySquad {
    name: "Main Squad".to_string(),
    role: SquadRole::AttackWorkers,
    status: SquadStatus::Gathering,
    assigned_unit_ids: std::collections::HashSet::new(),
    target_position: None,
    target_path: path_to_enemy,
    target_path_index: Some(goal),
    leader_unit_id: None,
    unit_path_assignments: std::collections::HashMap::new(),
  }
}

pub fn attack_nearby_worker(
  game: &Game,
  unit: &Unit,
  enemy_workers_close_to_squad: &[Unit],
) -> bool {
  if can_attack_worker_close_to_unit(game, unit) {
    return true;
  }

  if can_attack_worker_close_to_squad_target(unit, enemy_workers_close_to_squad) {
    return true;
  }

  if avoid_enemy_movement_utils::handle_threat_avoidance(game, unit, None, ThreatAvoidanceMode::Aggressive) {
    return true;
  }

  false
}

fn can_attack_worker_close_to_unit(game: &Game, unit: &Unit) -> bool {
  let workers_close_to_this_unit =
    avoid_enemy_movement_utils::get_enemies_within(game, unit.get_position(), 120.0, unit.get_player().get_id());

  if let Some(closest_worker) = workers_close_to_this_unit.first() {
    let unit_order = unit.get_order();
    let order_target = unit.get_target();

    if unit_order == Order::AttackUnit {
      return true;
    }

    let order_unit_target_id = order_target.and_then(|u: Unit| {
      let id = u.get_id();
      if id < 10000 {
        Some(id)
      } else {
        None
      }
    });
    if unit_order != Order::AttackUnit || order_unit_target_id != Some(closest_worker.get_id()) {
      let _ = unit.attack(closest_worker);
    }
    return true;
  }

  false
}

fn can_attack_worker_close_to_squad_target(
  unit: &Unit,
  enemy_workers_close_to_squad: &[Unit],
) -> bool {
  // Sort workers by distance to this specific unit
  let unit_pos = unit.get_position();
  let mut workers_with_distance: Vec<(&Unit, f32)> = enemy_workers_close_to_squad
    .iter()
    .map(|worker| {
      let worker_pos = worker.get_position();
      let dx = (worker_pos.x - unit_pos.x) as f32;
      let dy = (worker_pos.y - unit_pos.y) as f32;
      let distance_squared = dx * dx + dy * dy;
      (worker, distance_squared)
    })
    .collect();

  workers_with_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

  if let Some((closest_worker, distance_squared)) = workers_with_distance.first() {
    let distance = distance_squared.sqrt();

    if unit.is_in_weapon_range(closest_worker) || distance < 120.0 {
      let unit_order = unit.get_order();
      let order_target = unit.get_target();

      if unit_order == Order::AttackUnit {
        return true;
      }

      let order_unit_target_id = order_target.and_then(|u: Unit| {
        let id = u.get_id();
        if id < 10000 {
          Some(id)
        } else {
          None
        }
      });

      if unit_order != Order::AttackUnit || order_unit_target_id != Some(closest_worker.get_id()) {
        let _ = unit.attack(*closest_worker);
      }
      return true;
    }
  }

  false
}

pub fn move_to_target(unit: &Unit, target_x: i32, target_y: i32) {
  let target_position = Position::new(target_x, target_y);
  let unit_order = unit.get_order();
  let order_target = unit.get_order_target_position();
  if unit_order != Order::Move || order_target != Some(target_position) {
    let _ = unit.move_(target_position);
  }
}

pub fn get_worker_enemies_within(
  game: &Game,
  position: Position,
  radius: f32,
  player_id: usize,
) -> Vec<Unit> {
  let radius_squared = radius * radius;
  
  // Get all enemy attacking units (buildings and units that can attack)
  let enemy_attackers: Vec<Unit> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      if u.get_player().get_id() == player_id {
        return false;
      }
      let unit_type = u.get_type();
      let ground_weapon = unit_type.ground_weapon();
      let air_weapon = unit_type.air_weapon();
      ground_weapon != WeaponType::None || air_weapon != WeaponType::None
    })
    .collect();
  
  let mut workers: Vec<(Unit, f32, bool)> = game
    .get_all_units()
    .into_iter()
    .filter_map(|u| {
      if u.get_player().get_id() == player_id {
        return None;
      }
      let unit_type = u.get_type();
      // Check if it's a worker unit
      if unit_type != UnitType::Terran_SCV
        && unit_type != UnitType::Protoss_Probe
        && unit_type != UnitType::Zerg_Drone
      {
        return None;
      }
      let enemy_pos = u.get_position();
      let dx = (position.x - enemy_pos.x) as f32;
      let dy = (position.y - enemy_pos.y) as f32;
      let distance_squared = dx * dx + dy * dy;
      if distance_squared <= radius_squared {
        // Check if worker is in range of any enemy attacker
        let in_danger = enemy_attackers.iter().any(|attacker| {
          let attacker_pos = attacker.get_position();
          let attacker_type = attacker.get_type();
          let weapon_range = if attacker_type.ground_weapon() != WeaponType::None {
            attacker_type.ground_weapon().max_range() as f32
          } else {
            attacker_type.air_weapon().max_range() as f32
          };
          
          let adx = (attacker_pos.x - enemy_pos.x) as f32;
          let ady = (attacker_pos.y - enemy_pos.y) as f32;
          let dist_to_attacker = (adx * adx + ady * ady).sqrt();
          
          dist_to_attacker <= weapon_range + 32.0 // Add small buffer
        });
        
        Some((u, distance_squared, in_danger))
      } else {
        None
      }
    })
    .collect();

  // Sort by: 1) not in danger first, 2) then by distance
  workers.sort_by(|a, b| {
    match (a.2, b.2) {
      (false, true) => std::cmp::Ordering::Less,  // a is safe, b is in danger
      (true, false) => std::cmp::Ordering::Greater, // a is in danger, b is safe
      _ => a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal), // same safety, sort by distance
    }
  });
  
  workers.into_iter().map(|(u, _, _)| u).collect()
}

fn create_building_patrol_path_for_workers(
  game: &Game,
  squad: &MilitarySquad,
) -> Option<Vec<(i32, i32)>> {
  let Some(self_player) = game.self_() else {
    return None;
  };

  let player_id = self_player.get_id();

  // Get all visible enemy buildings
  let enemy_buildings: Vec<(i32, i32)> = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_player().get_id() != player_id && u.get_type().is_building() && u.is_visible()
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

pub fn update_attack_workers_squad(game: &Game, squad: &mut MilitarySquad) {
  // First ensure we have a path
  if squad.target_path.is_none() {
    let Some(self_player) = game.self_() else {
      return;
    };

    let start_locations: Vec<ScaledPosition<32>> = game.get_start_locations();
    let Some(my_starting_position) = get_player_start_location(game, &self_player) else {
      return;
    };

    let Some(enemy_location) = start_locations
      .iter()
      .find(|&&loc| loc != my_starting_position)
    else {
      return;
    };

    let average_position_of_minerals_near_enemy_location =
      get_average_resource_position_near_location(game, enemy_location);

    let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
    let enemy_pos = average_position_of_minerals_near_enemy_location;

    let path_to_enemy = pathing::get_path_between_points(game, my_pos, enemy_pos, Some(false));

    if let Some(ref path) = path_to_enemy {
      let goal = path.len() / 2;
      squad.target_path = path_to_enemy;
      squad.target_path_index = Some(goal);
    } else {
      println!("Squad {} no path to enemy found", squad.name);
      return;
    }
  }

  // Then ensure we have a target position
  if squad.target_position.is_none() {
    if let (Some(ref path), Some(index)) = (&squad.target_path, squad.target_path_index) {
      if index < path.len() {
        squad.target_position = Some(path[index]);
      }
    }

    if squad.target_position.is_none() {
      game.draw_text_screen((0, 70), &format!("Squad {} has no target pos", squad.name));
      return;
    }
  }
  
  // Update target position along path if needed
  if let (Some(ref path), Some(index)) = (&squad.target_path, squad.target_path_index) {
    if index < path.len() {
      squad.target_position = Some(path[index]);
    }
  }

  let squad_units: Vec<Unit> = squad
    .assigned_unit_ids
    .iter()
    .filter_map(|&unit_id| game.get_unit(unit_id))
    .collect();

  let squad_count_close_to_target =
    get_units_close_to_position(&squad_units, squad.target_position.unwrap(), 80.0);

  if squad_count_close_to_target < 4 {
    return;
  }

  let Some(ref path) = squad.target_path else {
    println!(
      "Squad {} cannot switch to attacking: no target path",
      squad.name
    );
    return;
  };

  if path.is_empty() {
    println!(
      "Squad {} cannot switch to attacking: path is empty",
      squad.name
    );
    return;
  }

  // Check if we're at the end of the path
  let current_index = squad.target_path_index.unwrap_or(0);
  if current_index >= path.len() - 1 && squad.status == SquadStatus::Attacking {
    // We've reached the end, try to create a building patrol route
    if let Some(patrol_path) = create_building_patrol_path_for_workers(game, squad) {
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

  squad.status = SquadStatus::Attacking;
  squad.target_path_index = Some(path.len() - 1);
  squad.target_position = Some(path[path.len() - 1]);
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
