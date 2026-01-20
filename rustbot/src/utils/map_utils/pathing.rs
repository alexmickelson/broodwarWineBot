use rsbwapi::*;
use std::collections::{HashSet, VecDeque};

pub fn get_path_between_points(
  game: &Game,
  start: (i32, i32),
  end: (i32, i32),
  is_flier: Option<bool>,
) -> Option<Vec<(i32, i32)>> {
  println!(
    "Calculating path from ({}, {}) to ({}, {})",
    start.0, start.1, end.0, end.1
  );
  let mut queue: VecDeque<((i32, i32), Vec<(i32, i32)>)> = VecDeque::new();
  let mut visited: HashSet<(i32, i32)> = HashSet::new();
  let mut locations_checked = 0;
  queue.push_back((start, vec![start]));
  visited.insert(start);

  let dimension_interval = 15;
  let directions = [
    (0, -dimension_interval), // North
    (dimension_interval, 0),  // East
    (0, dimension_interval),  // South
    (-dimension_interval, 0), // West
    (dimension_interval, -dimension_interval), // NE (Northeast)
    (dimension_interval, dimension_interval),  // SE (Southeast)
    (-dimension_interval, dimension_interval), // SW (Southwest)
    (-dimension_interval, -dimension_interval), // NW (Northwest)
  ];

  let close_threshold = 3 * dimension_interval;
  let map_width_pixels = game.map_width() * 32;
  let map_height_pixels = game.map_height() * 32;
  let is_flier = is_flier.unwrap_or(false);

  while let Some((current, path)) = queue.pop_front() {
    locations_checked += 1;

    // if locations_checked % 1000 == 0 {
    //   println!("Pathfinding: checked {} locations", locations_checked);
    // }

    let dx = (current.0 - end.0).abs();
    let dy = (current.1 - end.1).abs();
    let distance_squared = dx * dx + dy * dy;

    if distance_squared <= (close_threshold * close_threshold) {
      println!(
        "Path found with {} steps after checking {} locations",
        path.len(),
        locations_checked
      );
      return Some(path);
    }

    for (dx, dy) in &directions {
      let neighbor = (current.0 + dx, current.1 + dy);

      if visited.contains(&neighbor) {
        continue;
      }

      if neighbor.0 < 0
        || neighbor.0 >= map_width_pixels
        || neighbor.1 < 0
        || neighbor.1 >= map_height_pixels
      {
        continue;
      }

      let position = Position::new(neighbor.0, neighbor.1);
      
      // Only check walkability for non-fliers
      if !is_flier && !game.is_walkable(position.to_walk_position()) {
        continue;
      }

      visited.insert(neighbor);
      let mut new_path = path.clone();
      new_path.push(neighbor);
      queue.push_back((neighbor, new_path));
    }
  }

  println!(
    "No path found after checking {} locations",
    locations_checked
  );
  None
}

pub fn get_path_avoiding_enemies(
  game: &Game,
  start: (i32, i32),
  end: (i32, i32),
  is_flier: bool,
  player_id: usize,
) -> Option<Vec<(i32, i32)>> {
  println!(
    "Calculating path avoiding enemies from ({}, {}) to ({}, {})",
    start.0, start.1, end.0, end.1
  );
  
  // Get all enemy units
  let enemy_units: Vec<Unit> = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_player().get_id() != player_id && u.exists())
    .collect();

  let mut queue: VecDeque<((i32, i32), Vec<(i32, i32)>)> = VecDeque::new();
  let mut visited: HashSet<(i32, i32)> = HashSet::new();
  let mut locations_checked = 0;
  queue.push_back((start, vec![start]));
  visited.insert(start);

  let dimension_interval = 15;
  let directions = [
    (0, -dimension_interval), // North
    (dimension_interval, 0),  // East
    (0, dimension_interval),  // South
    (-dimension_interval, 0), // West
    (dimension_interval, -dimension_interval), // NE (Northeast)
    (dimension_interval, dimension_interval),  // SE (Southeast)
    (-dimension_interval, dimension_interval), // SW (Southwest)
    (-dimension_interval, -dimension_interval), // NW (Northwest)
  ];

  let close_threshold = 3 * dimension_interval;
  let map_width_pixels = game.map_width() * 32;
  let map_height_pixels = game.map_height() * 32;
  let enemy_avoidance_range = 200.0; // Stay this far from enemies

  while let Some((current, path)) = queue.pop_front() {
    locations_checked += 1;

    let dx = (current.0 - end.0).abs();
    let dy = (current.1 - end.1).abs();
    let distance_squared = dx * dx + dy * dy;

    if distance_squared <= (close_threshold * close_threshold) {
      println!(
        "Path avoiding enemies found with {} steps after checking {} locations",
        path.len(),
        locations_checked
      );
      return Some(path);
    }

    for (dx, dy) in &directions {
      let neighbor = (current.0 + dx, current.1 + dy);

      if visited.contains(&neighbor) {
        continue;
      }

      if neighbor.0 < 0
        || neighbor.0 >= map_width_pixels
        || neighbor.1 < 0
        || neighbor.1 >= map_height_pixels
      {
        continue;
      }

      let position = Position::new(neighbor.0, neighbor.1);
      
      // Only check walkability for non-fliers
      if !is_flier && !game.is_walkable(position.to_walk_position()) {
        continue;
      }

      // Check if this position is too close to any enemy
      let mut too_close_to_enemy = false;
      for enemy in &enemy_units {
        let enemy_pos = enemy.get_position();
        let enemy_dx = (neighbor.0 - enemy_pos.x) as f32;
        let enemy_dy = (neighbor.1 - enemy_pos.y) as f32;
        let distance_to_enemy = (enemy_dx * enemy_dx + enemy_dy * enemy_dy).sqrt();
        
        if distance_to_enemy < enemy_avoidance_range {
          too_close_to_enemy = true;
          break;
        }
      }

      if too_close_to_enemy {
        continue;
      }

      visited.insert(neighbor);
      let mut new_path = path.clone();
      new_path.push(neighbor);
      queue.push_back((neighbor, new_path));
    }
  }

  println!(
    "No path avoiding enemies found after checking {} locations",
    locations_checked
  );
  None
}

pub fn draw_path(game: &Game, path: &Vec<(i32, i32)>) {
  if path.len() < 2 {
    return;
  }

  for i in 0..path.len() - 1 {
    let start = Position::new(path[i].0, path[i].1);
    let end = Position::new(path[i + 1].0, path[i + 1].1);
    game.draw_line_map(start, end, Color::Purple);
  }

  for (x, y) in path {
    let pos = Position::new(*x, *y);
    game.draw_circle_screen(pos, 3, Color::Purple, true)
  }
}
