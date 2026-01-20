use rsbwapi::*;

/// Identifies threats based on weapon range and proximity
pub fn get_threats_with_range_awareness(
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

/// Simple directional avoidance without pathfinding
/// Returns a position to move to that avoids threats while progressing toward target
pub fn calculate_simple_avoidance_move(
  game: &Game,
  unit_pos: Position,
  threats: &[Unit],
  target_x: Option<i32>,
  target_y: Option<i32>,
  backward_weight: f32,
  lateral_weight: f32,
  target_weight: f32,
) -> Option<Position> {
  if threats.is_empty() {
    return None;
  }

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

  // Calculate average threat position
  let mut avg_threat_x = 0.0;
  let mut avg_threat_y = 0.0;
  for threat in &relevant_threats {
    let threat_pos = threat.get_position();
    avg_threat_x += threat_pos.x as f32;
    avg_threat_y += threat_pos.y as f32;
  }
  avg_threat_x /= relevant_threats.len() as f32;
  avg_threat_y /= relevant_threats.len() as f32;

  // Direction away from threats
  let avoid_x = unit_pos.x as f32 - avg_threat_x;
  let avoid_y = unit_pos.y as f32 - avg_threat_y;

  let avoid_length = (avoid_x * avoid_x + avoid_y * avoid_y).sqrt();
  let (norm_avoid_x, norm_avoid_y) = if avoid_length > 0.0 {
    (avoid_x / avoid_length, avoid_y / avoid_length)
  } else {
    (1.0, 0.0)
  };

  // Add perpendicular component for arc movement (circle around threats)
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

  // Combine backward, lateral, and target components
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

  // Validate walkability
  let is_valid_and_walkable = |pos: Position| -> bool {
    let walk_pos = pos.to_walk_position();
    if walk_pos.x < 0 || walk_pos.y < 0 || walk_pos.x > 1024 || walk_pos.y > 1024 {
      return false;
    }
    game.is_walkable(walk_pos)
  };

  if is_valid_and_walkable(move_pos) {
    Some(move_pos)
  } else {
    // Try to find a walkable position nearby in a spiral pattern
    let mut found_walkable = None;
    for radius in 1..=5 {
      let check_distance = radius * 8;
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

    found_walkable
  }
}

/// Check if we should reissue a move command to keep unit moving smoothly
pub fn should_reissue_move_command(
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

#[derive(Debug, Clone, Copy)]
pub enum ThreatAvoidanceMode {
  Evasive,
  Kiting,
  Aggressive,
}

#[derive(Debug, Clone, Copy)]
struct ThreatAvoidanceWeights {
  backward: f32,
  lateral: f32,
  target: f32,
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

/// High-level function that handles threat avoidance for a unit
/// Returns true if avoidance movement was issued
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

  if let Some(final_move_pos) = calculate_simple_avoidance_move(
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

/// Get all enemy units within a radius
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

/// Get all enemy units within a radius (no filtering by weapon type)
pub fn get_all_enemies_within(
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
