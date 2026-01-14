use crate::utils::{
  game_state::{GameState, SharedGameState},
  map_utils::pathing,
  military::squad_models::{MilitarySquad, SquadRole, SquadStatus},
};
use rsbwapi::*;

pub fn military_onframe(game: &Game, game_state: &mut SharedGameState) {
  let Ok(mut game_state) = game_state.lock() else {
    println!("Failed to lock game_state in military_onframe");
    return;
  };
  update_squads(game, &mut game_state);
  enforce_military_assignments(game, &mut game_state);
}

pub fn assign_unit_to_squad(game: &Game, unit: &Unit, game_state: &mut GameState) {
  let first_squad = game_state.military_squads.first_mut();
  if let Some(squad) = first_squad {
    squad.assigned_unit_ids.insert(unit.get_id() as usize);
    return;
  }

  game.draw_text_screen((0, 50), "no squads available to assign unit");
}

pub fn is_military_unit(unit: &Unit) -> bool {
  if unit.get_type().is_building()
    || unit.get_type() == UnitType::Zerg_Larva
    || unit.get_type() == UnitType::Zerg_Egg
    || unit.get_type() == UnitType::Zerg_Drone
    || unit.get_type() == UnitType::Zerg_Overlord
  {
    return false;
  }
  true
}

pub fn remove_unit_from_squads(unit: &Unit, game_state: &mut GameState) {
  let unit_id = unit.get_id() as usize;
  for squad in game_state.military_squads.iter_mut() {
    let _ = squad.assigned_unit_ids.remove(&unit_id);
  }
}

pub fn create_initial_squad(game: &Game) -> Option<MilitarySquad> {
  let Some(self_player) = game.self_() else {
    return None;
  };

  let start_locations = game.get_start_locations();
  let Some(my_starting_position) = start_locations.get(self_player.get_id() as usize) else {
    return None;
  };

  let Some(enemy_location) = start_locations
    .iter()
    .find(|&loc| loc != my_starting_position)
  else {
    return None;
  };

  let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
  let enemy_pos = (enemy_location.x * 32, enemy_location.y * 32);

  let path_to_enemy = pathing::get_path_between_points(game, my_pos, enemy_pos);

  let goal = if let Some(ref path) = path_to_enemy {
    path.len() / 5
  } else {
    println!("No path to enemy found when creating initial squad");
    0
  };

  Some(MilitarySquad {
    name: "Main Squad".to_string(),
    role: SquadRole::AttackWorkers,
    status: SquadStatus::Gathering,
    assigned_unit_ids: std::collections::HashSet::new(),
    target_position: None,
    target_path: path_to_enemy,
    target_path_index: Some(goal),
  })
}

pub fn update_squads(game: &Game, game_state: &mut GameState) {
  for squad in game_state.military_squads.iter_mut() {
    if let (Some(ref path), Some(index)) = (&squad.target_path, squad.target_path_index) {
      if index < path.len() {
        squad.target_position = Some(path[index]);
      }
    }

    if squad.target_position.is_none() {
      println!(
        "Squad {} has no target position, skipping update",
        squad.name
      );
      continue;
    }

    let squad_count = squad.assigned_unit_ids.len();
    let squad_units: Vec<Unit> = squad
      .assigned_unit_ids
      .iter()
      .filter_map(|&unit_id| game.get_unit(unit_id))
      .collect();
    let squad_count_close_to_target =
      get_units_close_to_position(&squad_units, squad.target_position.unwrap(), 80.0);

    match squad.role {
      SquadRole::Attack => {}
      SquadRole::Defend => {}
      SquadRole::AttackWorkers => {
        if squad_count_close_to_target < 6 {
          // println!(
          //   "Squad {} not ready to attack: {} units close to target (need 6)",
          //   squad.name, squad_count_close_to_target
          // );
          continue;
        }

        let Some(ref path) = squad.target_path else {
          println!(
            "Squad {} cannot switch to attacking: no target path",
            squad.name
          );
          continue;
        };

        if path.is_empty() {
          println!(
            "Squad {} cannot switch to attacking: path is empty",
            squad.name
          );
          continue;
        }

        squad.status = SquadStatus::Attacking;
        squad.target_path_index = Some(path.len() - 1);
        squad.target_position = Some(path[path.len() - 1]);
      }
    }
  }
}

fn calculate_threat_avoidance_move(
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
  let perp_x = -norm_avoid_y;
  let perp_y = norm_avoid_x;

  let mut move_x = unit_pos.x as f32 + norm_avoid_x * backward_weight + perp_x * lateral_weight;
  let mut move_y = unit_pos.y as f32 + norm_avoid_y * backward_weight + perp_y * lateral_weight;

  // Add target component if provided
  if let (Some(tx), Some(ty)) = (target_x, target_y) {
    let target_dx = tx as f32 - unit_pos.x as f32;
    let target_dy = ty as f32 - unit_pos.y as f32;
    move_x += target_dx * target_weight;
    move_y += target_dy * target_weight;
  }

  Some(Position::new(move_x as i32, move_y as i32))
}

fn attack_nearby_worker(game: &Game, unit: &Unit) -> bool {
  // Tunable parameters for worker harassment
  let backward_movement_weight = 90.0;
  let lateral_movement_weight = 60.0;

  // Get threats with weapon-range-aware detection
  let nearby_threats =
    get_threats_with_range_awareness(game, unit.get_position(), unit.get_player().get_id());

  if !nearby_threats.is_empty() {
    if let Some(move_pos) = calculate_threat_avoidance_move(
      unit.get_position(),
      &nearby_threats,
      None,
      None,
      backward_movement_weight,
      lateral_movement_weight,
      0.0,
    ) {
      let _ = unit.move_(move_pos);
      return true;
    }
  }

  // Only attack workers if no threats nearby
  let nearby_workers =
    get_worker_enemies_within(game, unit.get_position(), 200.0, unit.get_player().get_id());

  if let Some(closest_worker) = nearby_workers.first() {
    let unit_order = unit.get_order();
    let order_target = unit.get_target();

    if unit_order != Order::AttackUnit
      || order_target.map(|u: Unit| u.get_id()) != Some(closest_worker.get_id())
    {
      let _ = unit.attack(closest_worker);
    }
    return true;
  }
  false
}

fn move_avoiding_threats(game: &Game, unit: &Unit, target_x: i32, target_y: i32) -> bool {
  // Tunable parameters
  let backward_movement_weight = 30.0;
  let lateral_movement_weight = 65.0;
  let target_movement_weight = 0.05;

  // Get threats with weapon-range-aware detection
  let nearby_threats =
    get_threats_with_range_awareness(game, unit.get_position(), unit.get_player().get_id());

  if nearby_threats.is_empty() {
    return false;
  }

  if let Some(move_pos) = calculate_threat_avoidance_move(
    unit.get_position(),
    &nearby_threats,
    Some(target_x),
    Some(target_y),
    backward_movement_weight,
    lateral_movement_weight,
    target_movement_weight,
  ) {
    let unit_order = unit.get_order();
    let order_target = unit.get_order_target_position();
    if unit_order != Order::Move || order_target != Some(move_pos) {
      let _ = unit.move_(move_pos);
    }
    return true;
  }

  false
}

fn move_to_target(unit: &Unit, target_x: i32, target_y: i32) {
  let target_position = Position::new(target_x, target_y);
  let unit_order = unit.get_order();
  let order_target = unit.get_order_target_position();
  if unit_order != Order::Move || order_target != Some(target_position) {
    let _ = unit.move_(target_position);
  }
}

fn enforce_military_assignments(game: &Game, game_state: &mut GameState) {
  for squad in game_state.military_squads.iter() {
    for &unit_id in &squad.assigned_unit_ids {
      let Some(unit) = game.get_unit(unit_id) else {
        continue;
      };
      unit_in_squad_control(game, &unit, squad);
    }
  }
}

fn unit_in_squad_control(game: &Game, unit: &Unit, squad: &MilitarySquad) {
  match squad.role {
    SquadRole::Attack => {}
    SquadRole::Defend => {}
    SquadRole::AttackWorkers => match squad.status {
      SquadStatus::Gathering => {
        let nearby_enemies =
          get_enemies_within(game, unit.get_position(), 80.0, unit.get_player().get_id());
        if let Some(closest_enemy) = nearby_enemies.first() {
          let unit_order = unit.get_order();
          let order_target = unit.get_target();

          if unit_order == Order::AttackUnit {
            return;
          }

          if unit_order != Order::AttackUnit
            || order_target.map(|u| u.get_id()) != Some(closest_enemy.get_id())
          {
            let _ = unit.attack(closest_enemy);
          }
          return;
        } else {
          let Some((target_x, target_y)) = squad.target_position else {
            return;
          };
          let target_position = Position::new(target_x, target_y);
          let unit_order = unit.get_order();
          let order_target = unit.get_order_target_position();
          if unit_order != Order::AttackMove || order_target != Some(target_position) {
            let _ = unit.attack(target_position);
          }
        }
      }
      SquadStatus::Attacking => {
        if attack_nearby_worker(game, unit) {
          return;
        }

        let Some((target_x, target_y)) = squad.target_position else {
          return;
        };

        if move_avoiding_threats(game, unit, target_x, target_y) {
          return;
        }

        move_to_target(unit, target_x, target_y);
      }
    },
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
pub fn draw_military_assignments(game: &Game, game_state: &GameState) {
  for squad in &game_state.military_squads {
    if let Some(target_path) = squad.target_path.as_ref() {
      pathing::draw_path(game, target_path);

      if let Some(index) = squad.target_path_index {
        if index < target_path.len() {
          let target_pos = Position::new(target_path[index].0, target_path[index].1);
          game.draw_circle_map(target_pos, 10, Color::Red, false);
        }
      }
    }

    for &unit_id in &squad.assigned_unit_ids {
      let Some(unit) = game.get_unit(unit_id) else {
        continue;
      };

      if let Some((target_x, target_y)) = squad.target_position {
        let unit_pos = unit.get_position();
        let target_pos = Position::new(target_x, target_y);
        game.draw_line_map(unit_pos, target_pos, Color::Red);
      }
    }
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

fn get_enemies_within(game: &Game, position: Position, radius: f32, player_id: usize) -> Vec<Unit> {
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

fn get_worker_enemies_within(
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
