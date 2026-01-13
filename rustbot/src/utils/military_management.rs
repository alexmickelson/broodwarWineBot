use crate::utils::game_state::{GameState, MilitaryAssignment, SharedGameState};
use crate::utils::region_stuff::{chokepoint_to_guard_base, draw_region_boxes};
use rsbwapi::*;
use std::collections::HashSet;

pub fn military_onframe(game: &Game, game_state: &mut SharedGameState) {
  let Ok(mut game_state) = game_state.lock() else {
    println!("Failed to lock game_state in military_onframe");
    return;
  };

  let Some(self_player) = game.self_() else {
    println!("Could not get self player");
    return;
  };

  if !game_state.offensive_target.is_some() {
    game_state.offensive_target = get_offensive_target(game, &self_player);
  }

  let all_my_units: Vec<Unit> = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_player().get_id() == self_player.get_id())
    .collect();

  let my_military_units: Vec<Unit> = all_my_units
    .into_iter()
    .filter(|u| {
      !u.get_type().is_building()
        && u.get_type() != UnitType::Zerg_Larva
        && u.get_type() != UnitType::Zerg_Egg
        && u.get_type() != UnitType::Zerg_Drone
        && u.get_type() != UnitType::Zerg_Overlord
    })
    .collect();

  // println!("Military units after filter: {}", my_military_units.len());

  remove_dead_unit_assignments(game, &mut game_state);
  update_military_assignments(&my_military_units, &mut game_state);
  enforce_military_assignments(game, &my_military_units, &mut game_state);
}

fn remove_dead_unit_assignments(game: &Game, game_state: &mut GameState) {
  let unit_ids_to_remove: Vec<usize> = game_state
    .military_assignments
    .keys()
    .filter(|&&unit_id| game.get_unit(unit_id).map_or(true, |unit| !unit.exists()))
    .copied()
    .collect();

  for unit_id in unit_ids_to_remove {
    game_state.military_assignments.remove(&unit_id);
  }
}

fn enforce_military_assignments(
  game: &Game,
  my_military_units: &[Unit],
  game_state: &mut GameState,
) {
  for unit in my_military_units {
    let unit_id = unit.get_id() as usize;

    let Some(assignment) = game_state.military_assignments.get_mut(&unit_id) else {
      continue;
    };

    if assignment.target_position.is_some() {
      enforce_attack_position_assignment(unit, assignment);
      continue;
    }

    if assignment.target_path.is_some() {
      enforce_path_following_assignment(game, unit, assignment);
      continue;
    }
  }
}

fn update_military_assignments(my_military_units: &[Unit], game_state: &mut GameState) {
  let Some(path_to_enemy) = &game_state.path_to_enemy_base else {
    println!("No path to enemy base available for military assignment");
    return;
  };

  let assigned_unit_ids: HashSet<usize> = game_state.military_assignments.keys().copied().collect();

  let unassigned_units: Vec<&Unit> = my_military_units
    .iter()
    .filter(|u| !assigned_unit_ids.contains(&(u.get_id() as usize)))
    .collect();

  let military_unit_count = my_military_units.len();

  let unit_count_close_to_greatest_path_index =
    count_units_near_furthest_unit(my_military_units, game_state);

  let target_path_goal_index = if military_unit_count < 5 {
    path_to_enemy.len() / 4
  } else if military_unit_count < 15 || unit_count_close_to_greatest_path_index < 12 {
    path_to_enemy.len() / 2
  } else {
    path_to_enemy.len() - 1
  };

  for unit in unassigned_units {
    game_state.military_assignments.insert(
      unit.get_id() as usize,
      MilitaryAssignment {
        target_position: None,
        target_unit: None,
        target_path: Some(path_to_enemy.clone()),
        target_path_current_index: Some(0),
        target_path_goal_index: Some(target_path_goal_index),
      },
    );
  }

  for unit_id in assigned_unit_ids.iter() {
    if let Some(assignment) = game_state.military_assignments.get_mut(unit_id) {
      if assignment.target_path.is_some() {
        assignment.target_path_goal_index = Some(target_path_goal_index);
        // If the new goal index is less than current index, reduce current index
        if let Some(current_index) = assignment.target_path_current_index {
          if target_path_goal_index < current_index {
            assignment.target_path_current_index = Some(target_path_goal_index);
          }
        }
      }
    }
  }

  for unit_id in assigned_unit_ids {
    if game_state.military_assignments[&unit_id]
      .target_path
      .is_some()
    {
      let Some(unit) = my_military_units
        .iter()
        .find(|u| u.get_id() as usize == unit_id)
      else {
        println!(
          "Could not find unit with id {} for military assignment update",
          unit_id
        );
        continue;
      };
      update_path_assignment_if_close_to_goal(
        &unit,
        game_state.military_assignments.get_mut(&unit_id).unwrap(),
      );
    }
  }
}

fn count_units_near_furthest_unit(my_military_units: &[Unit], game_state: &GameState) -> usize {
  let mut max_index: Option<usize> = None;
  let mut furthest_unit_id: Option<usize> = None;

  // Find the unit with the greatest current_index
  for (unit_id, assignment) in game_state.military_assignments.iter() {
    if let (Some(current_index), Some(_path)) =
      (assignment.target_path_current_index, &assignment.target_path)
    {
      if max_index.is_none() || current_index > max_index.unwrap() {
        max_index = Some(current_index);
        furthest_unit_id = Some(*unit_id);
      }
    }
  }

  // Count units within 50 pixels of the furthest unit
  if let Some(furthest_id) = furthest_unit_id {
    if let Some(furthest_unit) = my_military_units
      .iter()
      .find(|u| u.get_id() as usize == furthest_id)
    {
      let furthest_pos = furthest_unit.get_position();
      my_military_units
        .iter()
        .filter(|u| {
          let pos = u.get_position();
          let dx = (pos.x - furthest_pos.x) as f32;
          let dy = (pos.y - furthest_pos.y) as f32;
          let distance = (dx * dx + dy * dy).sqrt();
          distance <= 50.0
        })
        .count()
    } else {
      0
    }
  } else {
    0
  }
}

fn update_path_assignment_if_close_to_goal(unit: &Unit, assignment: &mut MilitaryAssignment) {
  let Some(goal_index) = assignment.target_path_goal_index else {
    return;
  };
  let Some(path) = &assignment.target_path else {
    return;
  };
  let Some(current_index) = assignment.target_path_current_index else {
    return;
  };

  if current_index >= path.len() || goal_index >= path.len() {
    return;
  }

  let unit_position = unit.get_position();
  let current_position = Position::new(path[current_index].0, path[current_index].1);

  let dx = (unit_position.x - current_position.x) as f32;
  let dy = (unit_position.y - current_position.y) as f32;
  let distance = (dx * dx + dy * dy).sqrt();
  let close_enough_threshold = 30.0;
  let advance_increment = 20;

  if distance <= close_enough_threshold && current_index < goal_index {
    let next_index = (current_index + advance_increment).min(goal_index);
    assignment.target_path_current_index = Some(next_index);
  }
  if current_index > goal_index {
    assignment.target_path_current_index = Some(goal_index);
  }
}

fn get_offensive_target(game: &Game, self_player: &Player) -> Option<Position> {
  let start_locations = game.get_start_locations();
  let Some(self_start) = start_locations.get(self_player.get_id() as usize) else {
    println!("Could not get self start location");
    return None;
  };

  let self_start_pos = Position::new(self_start.x * 32 + 16, self_start.y * 32 + 16);
  chokepoint_to_guard_base(game, &self_start_pos)
}

pub fn draw_military_assignments(game: &Game, game_state: &GameState) {
  for (unit_id, assignment) in &game_state.military_assignments {
    let Some(unit) = game.get_unit(*unit_id) else {
      continue;
    };

    if let Some((target_x, target_y)) = assignment.target_position {
      let unit_pos = unit.get_position();
      let target_pos = Position::new(target_x, target_y);
      game.draw_line_map(unit_pos, target_pos, Color::Red);
    }
  }

  if let Some(target) = game_state.offensive_target {
    game.draw_circle_map(target, 20, Color::Yellow, false);
    game.draw_text_map(target, "Attack Target");
  }
}

fn enforce_attack_position_assignment(unit: &Unit, assignment: &MilitaryAssignment) {
  let Some((target_x, target_y)) = assignment.target_position else {
    return;
  };

  let target_position = Position::new(target_x, target_y);
  let unit_order = unit.get_order();
  let order_target = unit.get_order_target_position();

  if unit_order != Order::AttackMove || order_target != Some(target_position) {
    let _ = unit.attack(target_position);
  }
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

fn enforce_path_following_assignment(
  game: &Game,
  unit: &Unit,
  assignment: &mut MilitaryAssignment,
) {
  let Some(target_path_current_index) = assignment.target_path_current_index else {
    return;
  };
  let Some(target_path_goal_index) = assignment.target_path_goal_index else {
    return;
  };
  let Some(target_path) = &assignment.target_path else {
    return;
  };

  let nearby_enemy_units =
    get_enemies_within(game, unit.get_position(), 50.0, unit.get_player().get_id());

  if let Some(closest_enemy) = nearby_enemy_units.first() {
    let unit_order = unit.get_order();
    let order_target = unit.get_target();
    if unit_order != Order::AttackMove
      || order_target.map(|u| u.get_id()) != Some(closest_enemy.get_id())
    {
      let _ = unit.attack(closest_enemy);
    }
    return;
  }

  if target_path_current_index >= target_path.len()
    || target_path_current_index > target_path_goal_index
  {
    return;
  }

  let target_position = Position::new(
    target_path[target_path_current_index].0,
    target_path[target_path_current_index].1,
  );

  let unit_order = unit.get_order();
  let order_target = unit.get_order_target_position();
  if unit_order != Order::AttackMove || order_target != Some(target_position) {
    let _ = unit.attack(target_position);
  }
}
