use crate::utils::{
  map_utils::pathing,
  military::squad_models::{MilitarySquad, SquadRole, SquadStatus},
};
use rsbwapi::*;

pub fn create_initial_attack_workers_squad(game: &Game, self_player: &Player) -> MilitarySquad {
  let start_locations: Vec<ScaledPosition<32>> = game.get_start_locations();
  let Some(my_starting_position) = start_locations.get(self_player.get_id() as usize) else {
    return MilitarySquad {
      name: "Main Squad".to_string(),
      role: SquadRole::AttackWorkers,
      status: SquadStatus::Gathering,
      assigned_unit_ids: std::collections::HashSet::new(),
      target_position: None,
      target_path: None,
      target_path_index: None,
    };
  };

  let Some(enemy_location) = start_locations
    .iter()
    .find(|&loc| loc != my_starting_position)
  else {
    return MilitarySquad {
      name: "Main Squad".to_string(),
      role: SquadRole::AttackWorkers,
      status: SquadStatus::Gathering,
      assigned_unit_ids: std::collections::HashSet::new(),
      target_position: None,
      target_path: None,
      target_path_index: None,
    };
  };

  let average_position_of_minerals_near_enemy_location = {
    let mut sum_x = 0;
    let mut sum_y = 0;
    let mut count = 0;

    for unit in game.get_static_minerals() {
      let unit_pos = unit.get_position();
      let dist_x = (unit_pos.x - enemy_location.x * 32).abs();
      let dist_y = (unit_pos.y - enemy_location.y * 32).abs();
      if dist_x <= 300 && dist_y <= 300 {
        sum_x += unit_pos.x;
        sum_y += unit_pos.y;
        count += 1;
      }
    }

    if count > 0 {
      (sum_x / count, sum_y / count)
    } else {
      (enemy_location.x * 32, enemy_location.y * 32)
    }
  };

  let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
  let enemy_pos = average_position_of_minerals_near_enemy_location;

  let path_to_enemy = pathing::get_path_between_points(game, my_pos, enemy_pos);

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
  }
}

#[derive(Debug, Clone, Copy)]
struct ThreatAvoidanceWeights {
  backward: f32,
  lateral: f32,
  target: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ThreatAvoidanceMode {
  Evasive,
  Kiting,
  Aggressive,
}

impl ThreatAvoidanceMode {
  fn weights(&self) -> ThreatAvoidanceWeights {
    match self {
      ThreatAvoidanceMode::Evasive => ThreatAvoidanceWeights {
        backward: 90.0,
        lateral: 60.0,
        target: 0.0,
      },
      ThreatAvoidanceMode::Kiting => ThreatAvoidanceWeights {
        backward: 30.0,
        lateral: 65.0,
        target: 0.05,
      },
      ThreatAvoidanceMode::Aggressive => ThreatAvoidanceWeights {
        backward: 10.0,
        lateral: 60.0,
        target: 10.0,
      },
    }
  }
}

pub fn calculate_threat_avoidance_move(
  game: &Game,
  unit_pos: Position,
  threats: &[Unit],
  target_x: Option<i32>,
  target_y: Option<i32>,
  backward_weight: f32,
  lateral_weight: f32,
  target_weight: f32,
) -> Option<Position> {
  // Determine movement direction if we have a target
  let (move_dir_x, move_dir_y) = if let (Some(tx), Some(ty)) = (target_x, target_y) {
    let dx = tx as f32 - unit_pos.x as f32;
    let dy = ty as f32 - unit_pos.y as f32;
    let length = (dx * dx + dy * dy).sqrt();
    if length > 0.0 {
      (dx / length, dy / length)
    } else {
      (0.0, 0.0)
    }
  } else {
    (0.0, 0.0)
  };

  // Filter threats to only those in our path or very close
  let relevant_threats: Vec<&Unit> = threats
    .iter()
    .filter(|threat| {
      let threat_pos = threat.get_position();
      let dx = threat_pos.x as f32 - unit_pos.x as f32;
      let dy = threat_pos.y as f32 - unit_pos.y as f32;
      let distance = (dx * dx + dy * dy).sqrt();

      // If no target, consider all threats
      if move_dir_x == 0.0 && move_dir_y == 0.0 {
        return true;
      }

      // Normalize threat direction
      let threat_dir_x = dx / distance;
      let threat_dir_y = dy / distance;

      // Dot product to check if threat is in front of us
      let dot = threat_dir_x * move_dir_x + threat_dir_y * move_dir_y;

      // Only consider threats somewhat in our path (within 120 degrees in front) or very close
      dot > -0.5 || distance < 50.0
    })
    .collect();

  if relevant_threats.is_empty() {
    return None;
  }

  let mut avg_threat_x = 0.0;
  let mut avg_threat_y = 0.0;
  for threat in &relevant_threats {
    let threat_pos = threat.get_position();
    avg_threat_x += threat_pos.x as f32;
    avg_threat_y += threat_pos.y as f32;
  }
  avg_threat_x /= relevant_threats.len() as f32;
  avg_threat_y /= relevant_threats.len() as f32;

  let avoid_x = unit_pos.x as f32 - avg_threat_x;
  let avoid_y = unit_pos.y as f32 - avg_threat_y;

  let avoid_length = (avoid_x * avoid_x + avoid_y * avoid_y).sqrt();
  let (norm_avoid_x, norm_avoid_y) = if avoid_length > 0.0 {
    (avoid_x / avoid_length, avoid_y / avoid_length)
  } else {
    (1.0, 0.0)
  };

  // Add perpendicular component for arc movement (circle around threats)
  // Calculate both possible perpendicular directions
  let perp_left_x = -norm_avoid_y;
  let perp_left_y = norm_avoid_x;
  let perp_right_x = norm_avoid_y;
  let perp_right_y = -norm_avoid_x;

  // Choose which perpendicular direction to use
  let (perp_x, perp_y) = if let (Some(tx), Some(ty)) = (target_x, target_y) {
    // Calculate which perpendicular direction gets us closer to the target
    let to_target_x = tx as f32 - unit_pos.x as f32;
    let to_target_y = ty as f32 - unit_pos.y as f32;

    // Dot product to see which perpendicular aligns better with target direction
    let dot_left = perp_left_x * to_target_x + perp_left_y * to_target_y;
    let dot_right = perp_right_x * to_target_x + perp_right_y * to_target_y;

    if dot_left > dot_right {
      (perp_left_x, perp_left_y)
    } else {
      (perp_right_x, perp_right_y)
    }
  } else {
    // No explicit target - choose perpendicular that avoids the closest threat better
    // Calculate which side has fewer/more distant threats
    let mut left_threat_score = 0.0;
    let mut right_threat_score = 0.0;

    for threat in &relevant_threats {
      let threat_pos = threat.get_position();
      let to_threat_x = threat_pos.x as f32 - unit_pos.x as f32;
      let to_threat_y = threat_pos.y as f32 - unit_pos.y as f32;
      let threat_distance = (to_threat_x * to_threat_x + to_threat_y * to_threat_y).sqrt();

      if threat_distance > 0.0 {
        let norm_to_threat_x = to_threat_x / threat_distance;
        let norm_to_threat_y = to_threat_y / threat_distance;

        // Check which perpendicular moves us away from this threat
        let left_dot = perp_left_x * norm_to_threat_x + perp_left_y * norm_to_threat_y;
        let right_dot = perp_right_x * norm_to_threat_x + perp_right_y * norm_to_threat_y;

        // Negative dot means moving away from threat - that's good
        // Weight by inverse distance (closer threats matter more)
        let weight = 1.0 / (threat_distance + 1.0);
        left_threat_score -= left_dot * weight;
        right_threat_score -= right_dot * weight;
      }
    }

    // Choose the direction with better score (higher means moving away from threats)
    if left_threat_score > right_threat_score {
      (perp_left_x, perp_left_y)
    } else {
      (perp_right_x, perp_right_y)
    }
  };

  let mut move_x = unit_pos.x as f32 + norm_avoid_x * backward_weight + perp_x * lateral_weight;
  let mut move_y = unit_pos.y as f32 + norm_avoid_y * backward_weight + perp_y * lateral_weight;

  // Add target component if provided
  if let (Some(tx), Some(ty)) = (target_x, target_y) {
    let target_dx = tx as f32 - unit_pos.x as f32;
    let target_dy = ty as f32 - unit_pos.y as f32;
    move_x += target_dx * target_weight;
    move_y += target_dy * target_weight;
  }

  let move_pos = Position::new(move_x as i32, move_y as i32);

  // Helper to check if a position is valid and walkable
  let is_valid_and_walkable = |pos: Position| -> bool {
    let walk_pos = pos.to_walk_position();
    // Check if walk position coordinates are within reasonable bounds
    // Walk positions should be positive and within map size
    if walk_pos.x < 0 || walk_pos.y < 0 || walk_pos.x > 1024 || walk_pos.y > 1024 {
      return false;
    }
    game.is_walkable(walk_pos)
  };

  // Validate that the position is walkable, if not try to find a nearby walkable position
  if is_valid_and_walkable(move_pos) {
    Some(move_pos)
  } else {
    // Try to find a walkable position nearby by checking in a spiral pattern
    let mut found_walkable = None;
    for radius in 1..=5 {
      let check_distance = radius * 8; // Check in increments of 8 pixels
      for angle_steps in 0..8 {
        let angle = (angle_steps as f32) * std::f32::consts::PI / 4.0;
        let test_x = move_pos.x + (angle.cos() * check_distance as f32) as i32;
        let test_y = move_pos.y + (angle.sin() * check_distance as f32) as i32;
        let test_pos = Position::new(test_x, test_y);

        if is_valid_and_walkable(test_pos) {
          found_walkable = Some(test_pos);
          break;
        }
      }
      if found_walkable.is_some() {
        break;
      }
    }

    // If we found a walkable position, use it; otherwise return None
    found_walkable
  }
}

pub fn handle_threat_avoidance(
  game: &Game,
  unit: &Unit,
  target_position: Option<(i32, i32)>,
  mode: ThreatAvoidanceMode,
) -> bool {
  let nearby_threats =
    get_threats_with_range_awareness(game, unit.get_position(), unit.get_player().get_id());

  if nearby_threats.is_empty() {
    return false;
  }

  let weights = mode.weights();
  let (target_x, target_y) = match target_position {
    Some((x, y)) => (Some(x), Some(y)),
    None => (None, None),
  };

  if let Some(final_move_pos) = calculate_threat_avoidance_move(
    game,
    unit.get_position(),
    &nearby_threats,
    target_x,
    target_y,
    weights.backward,
    weights.lateral,
    weights.target,
  ) {
    let unit_order = unit.get_order();
    let order_target = unit.get_order_target_position();

    if should_reissue_move_command(unit, unit_order, order_target, final_move_pos) {
      let _ = unit.move_(final_move_pos);
    }

    return true;
  }

  false
}

fn should_reissue_move_command(
  unit: &Unit,
  unit_order: Order,
  order_target: Option<Position>,
  new_target: Position,
) -> bool {
  if unit_order == Order::Move {
    if let Some(current_target) = order_target {
      let unit_pos = unit.get_position();

      // Check distance to current target - reissue if close to prevent stopping
      let dist_to_current_x = (current_target.x - unit_pos.x) as f32;
      let dist_to_current_y = (current_target.y - unit_pos.y) as f32;
      let dist_to_current =
        (dist_to_current_x * dist_to_current_x + dist_to_current_y * dist_to_current_y).sqrt();

      // Reissue command if getting close to prevent deceleration
      if dist_to_current < 48.0 {
        return true;
      }

      // Calculate if current target direction differs from new target direction
      let current_dir_x = (current_target.x - unit_pos.x) as f32;
      let current_dir_y = (current_target.y - unit_pos.y) as f32;
      let current_length = (current_dir_x * current_dir_x + current_dir_y * current_dir_y).sqrt();

      let new_dir_x = (new_target.x - unit_pos.x) as f32;
      let new_dir_y = (new_target.y - unit_pos.y) as f32;
      let new_length = (new_dir_x * new_dir_x + new_dir_y * new_dir_y).sqrt();

      if current_length > 0.0 && new_length > 0.0 {
        // Normalize and calculate dot product
        let dot = (current_dir_x / current_length) * (new_dir_x / new_length)
          + (current_dir_y / current_length) * (new_dir_y / new_length);

        // If directions are similar (within ~60 degrees), keep old command
        dot < 0.5 // Re-issue if directions diverge significantly
      } else {
        true // Re-issue if we can't determine direction
      }
    } else {
      true // Re-issue if no target
    }
  } else {
    true // Re-issue if not moving
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

  if handle_threat_avoidance(game, unit, None, ThreatAvoidanceMode::Aggressive) {
    return true;
  }

  false
}

fn can_attack_worker_close_to_unit(game: &Game, unit: &Unit) -> bool {
  let workers_close_to_this_unit =
    get_enemies_within(game, unit.get_position(), 64.0, unit.get_player().get_id());

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

    if unit.is_in_weapon_range(closest_worker) || distance < 80.0 {
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

fn get_threats_with_range_awareness(
  game: &Game,
  position: Position,
  player_id: usize,
) -> Vec<Unit> {
  let max_detection_radius = 300.0;
  let radius_squared = max_detection_radius * max_detection_radius;

  let mut threats: Vec<(Unit, f32)> = game
    .get_all_units()
    .into_iter()
    .filter_map(|u| {
      if u.get_player().get_id() == player_id {
        return None;
      }

      if u.get_type() == UnitType::Zerg_Drone {
        return None;
      }

      // Ignore buildings that are not complete yet
      let unit_type = u.get_type();
      if unit_type.is_building() && !u.is_completed() {
        return None;
      }

      let ground_weapon = unit_type.ground_weapon();
      let air_weapon = unit_type.air_weapon();

      if ground_weapon == WeaponType::None && air_weapon == WeaponType::None {
        return None;
      }

      let enemy_pos = u.get_position();
      let dx = (position.x - enemy_pos.x) as f32;
      let dy = (position.y - enemy_pos.y) as f32;
      let distance_squared = dx * dx + dy * dy;

      if distance_squared > radius_squared {
        return None;
      }

      // Determine threat range based on weapon range
      let weapon_range = if ground_weapon != WeaponType::None {
        ground_weapon.max_range() as f32
      } else {
        air_weapon.max_range() as f32
      };

      // Melee units (range <= 32) are only threats when very close
      // Ranged units are threats from further away
      let threat_radius = if weapon_range <= 32.0 {
        60.0 // Melee threat radius
      } else {
        weapon_range + 100.0 // Ranged threat radius (weapon range + buffer)
      };

      let threat_radius_squared = threat_radius * threat_radius;

      if distance_squared <= threat_radius_squared {
        Some((u, distance_squared))
      } else {
        None
      }
    })
    .collect();

  threats.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
  threats.into_iter().map(|(u, _)| u).collect()
}

pub fn get_enemies_within(
  game: &Game,
  position: Position,
  radius: f32,
  player_id: usize,
) -> Vec<Unit> {
  let radius_squared = radius * radius;
  let mut enemies: Vec<(Unit, f32)> = game
    .get_all_units()
    .into_iter()
    .filter_map(|u| {
      if u.get_player().get_id() == player_id {
        return None;
      }

      if u.get_type() == UnitType::Zerg_Drone {
        return None;
      }

      // Only count units that can attack (have ground or air weapon)
      let unit_type = u.get_type();
      let ground_weapon = unit_type.ground_weapon();
      let air_weapon = unit_type.air_weapon();
      if ground_weapon == WeaponType::None && air_weapon == WeaponType::None {
        return None;
      }

      let enemy_pos = u.get_position();
      let dx = (position.x - enemy_pos.x) as f32;
      let dy = (position.y - enemy_pos.y) as f32;
      let distance_squared = dx * dx + dy * dy;
      if distance_squared <= radius_squared {
        Some((u, distance_squared))
      } else {
        None
      }
    })
    .collect();

  enemies.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
  enemies.into_iter().map(|(u, _)| u).collect()
}

pub fn get_worker_enemies_within(
  game: &Game,
  position: Position,
  radius: f32,
  player_id: usize,
) -> Vec<Unit> {
  let radius_squared = radius * radius;
  let mut workers: Vec<(Unit, f32)> = game
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
        Some((u, distance_squared))
      } else {
        None
      }
    })
    .collect();

  workers.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
  workers.into_iter().map(|(u, _)| u).collect()
}
