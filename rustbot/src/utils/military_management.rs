use crate::utils::game_state::{GameState, MilitaryAssignment, SharedGameState};
use rsbwapi::{region::Region, *};
use std::collections::HashSet;
use std::sync::OnceLock;

static CACHED_REGION_IDS: OnceLock<Vec<i32>> = OnceLock::new();

fn get_all_region_ids(game: &Game) -> &'static Vec<i32> {
  CACHED_REGION_IDS.get_or_init(|| {
    let mut region_ids = HashSet::new();

    let map_width = game.map_width();
    let map_height = game.map_height();

    for x in 0..map_width {
      for y in 0..map_height {
        let pos = Position::new(x * 32, y * 32);
        if let Some(region) = game.get_region_at(pos) {
          region_ids.insert(region.get_id());
        }
      }
    }

    region_ids.into_iter().collect()
  })
}

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

  update_military_assignments(&my_military_units, &mut game_state);
  enforce_military_assignments(game, &my_military_units, &mut game_state);
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
      enforce_path_following_assignment(unit, assignment);
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

  for unit in unassigned_units {
    game_state.military_assignments.insert(
      unit.get_id() as usize,
      MilitaryAssignment {
        target_position: None,
        target_unit: None,
        target_path: Some(path_to_enemy.clone()),
        target_path_current_index: Some(0),
        target_path_goal_index: Some(path_to_enemy.len() / 4),
      },
    );
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

fn chokepoint_to_guard_base(game: &Game, base_location: &Position) -> Option<Position> {
  let region_ids = get_all_region_ids(game);

  let mut closest_chokepoint_region: Option<(Region, f32)> = None;

  for &region_id in region_ids {
    let Some(region) = game.get_region(region_id as u16) else {
      continue;
    };

    if region.get_defense_priority() != 2 {
      continue;
    }

    // Calculate center of region
    let center_x = (region.get_bounds_left() + region.get_bounds_right()) / 2;
    let center_y = (region.get_bounds_top() + region.get_bounds_bottom()) / 2;

    // Calculate distance to base location
    let dx = (center_x - base_location.x) as f32;
    let dy = (center_y - base_location.y) as f32;
    let distance = (dx * dx + dy * dy).sqrt();

    match closest_chokepoint_region {
      None => closest_chokepoint_region = Some((region, distance)),
      Some((_, closest_distance)) if distance < closest_distance => {
        closest_chokepoint_region = Some((region, distance));
      }
      _ => {}
    }
  }

  closest_chokepoint_region.map(|(region, _)| {
    let center_x = (region.get_bounds_left() + region.get_bounds_right()) / 2;
    let center_y = (region.get_bounds_top() + region.get_bounds_bottom()) / 2;
    Position::new(center_x, center_y)
  })
}

fn draw_region_with_defense(game: &Game, region: Region) {
  let left = region.get_bounds_left();
  let top = region.get_bounds_top();
  let right = region.get_bounds_right();
  let bottom = region.get_bounds_bottom();

  let top_left = Position::new(left, top);
  let bottom_right = Position::new(right, bottom);
  game.draw_box_map(top_left, bottom_right, Color::Blue, false);

  let center_x = (left + right) / 2;
  let center_y = (top + bottom) / 2;
  let center = Position::new(center_x, center_y);
  game.draw_text_map(
    center,
    &format!("Defense: {}", region.get_defense_priority()),
  );
}

fn draw_region_boxes(game: &Game) {
  let Some(self_player) = game.self_() else {
    return;
  };

  let start_locations = game.get_start_locations();
  let Some(_self_start) = start_locations.get(self_player.get_id() as usize) else {
    return;
  };

  let region_ids = get_all_region_ids(game);

  for &region_id in region_ids {
    if let Some(region) = game.get_region(region_id as u16) {
      draw_region_with_defense(game, region);
    }
  }
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

  // Draw offensive target position
  if let Some(target) = game_state.offensive_target {
    game.draw_circle_map(target, 20, Color::Yellow, false);
    game.draw_text_map(target, "Attack Target");
  }

  draw_region_boxes(game);
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

fn enforce_path_following_assignment(unit: &Unit, assignment: &mut MilitaryAssignment) {
  let Some(target_path_current_index) = assignment.target_path_current_index else {
    return;
  };
  let Some(target_path_goal_index) = assignment.target_path_goal_index else {
    return;
  };
  let Some(target_path) = &assignment.target_path else {
    return;
  };

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
