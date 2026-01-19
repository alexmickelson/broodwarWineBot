use rsbwapi::*;

pub fn get_buildable_location(
  game: &Game,
  builder: &Unit,
  unit_type: UnitType,
  base_locations: &[TilePosition],
  base_index: Option<usize>,
) -> Option<TilePosition> {
  if is_extractor_type(unit_type) {
    return find_extractor_location(game, builder, unit_type, base_locations, base_index);
  }

  let player = game.self_()?;
  let base_index = base_index.unwrap_or(0);
  let base_tile = base_locations.get(base_index)?;
  let search_center_pos = Position::new(base_tile.x * 32, base_tile.y * 32);
  let search_radius = 15;

  let resource_depots = collect_resource_depots(&player);
  let all_minerals = game.get_static_minerals();
  let all_geysers = game.get_static_geysers();

  let scored_locations = find_all_buildable_tiles(
    game,
    builder,
    unit_type,
    search_center_pos,
    search_radius,
    &resource_depots,
    &all_minerals,
    &all_geysers,
  );

  scored_locations
    .into_iter()
    .max_by_key(|(_, score)| *score)
    .map(|(pos, _)| pos)
}

fn is_extractor_type(unit_type: UnitType) -> bool {
  unit_type == UnitType::Zerg_Extractor
    || unit_type == UnitType::Terran_Refinery
    || unit_type == UnitType::Protoss_Assimilator
}

fn collect_resource_depots(player: &Player) -> Vec<Unit> {
  player
    .get_units()
    .into_iter()
    .filter(|u| u.get_type().is_resource_depot())
    .collect()
}

fn find_all_buildable_tiles(
  game: &Game,
  builder: &Unit,
  unit_type: UnitType,
  search_center_pos: Position,
  search_radius: i32,
  resource_depots: &[Unit],
  all_minerals: &[Unit],
  all_geysers: &[Unit],
) -> Vec<(TilePosition, i32)> {
  let mut buildable_locations = Vec::new();

  for dy in -search_radius..=search_radius {
    for dx in -search_radius..=search_radius {
      let tile_pos = TilePosition {
        x: search_center_pos.x / 32 + dx,
        y: search_center_pos.y / 32 + dy,
      };

      if !is_tile_buildable(game, builder, tile_pos, unit_type) {
        continue;
      }

      let score = score_build_location(tile_pos, resource_depots, all_minerals, all_geysers);

      buildable_locations.push((tile_pos, score));
    }
  }

  buildable_locations
}

fn is_tile_buildable(
  game: &Game,
  builder: &Unit,
  tile_pos: TilePosition,
  unit_type: UnitType,
) -> bool {
  game
    .can_build_here(builder, tile_pos, unit_type, true)
    .unwrap_or(false)
}

fn score_build_location(
  tile_pos: TilePosition,
  resource_depots: &[Unit],
  all_minerals: &[Unit],
  all_geysers: &[Unit],
) -> i32 {
  let tile_center_pos = tile_to_pixel_position(tile_pos);
  let mut score = 0;

  score += score_depot_relationships(tile_center_pos, resource_depots, all_minerals, all_geysers);
  score +=
    penalize_depot_resource_paths(tile_center_pos, resource_depots, all_minerals, all_geysers);

  score
}

fn tile_to_pixel_position(tile_pos: TilePosition) -> Position {
  Position::new(tile_pos.x * 32 + 16, tile_pos.y * 32 + 16)
}

fn score_depot_relationships(
  location: Position,
  resource_depots: &[Unit],
  all_minerals: &[Unit],
  all_geysers: &[Unit],
) -> i32 {
  let mut score = 0;

  for depot in resource_depots {
    let depot_pos = depot.get_position();

    score += penalize_mineral_line_blocking(location, depot_pos, all_minerals, all_geysers);
    score += score_depot_distance(location, depot_pos);
  }

  score
}

fn penalize_mineral_line_blocking(
  location: Position,
  depot_pos: Position,
  all_minerals: &[Unit],
  all_geysers: &[Unit],
) -> i32 {
  let mut penalty = 0;
  let tolerance = 3.0 * 32.0;

  for mineral in all_minerals {
    if is_between(location, depot_pos, mineral.get_position(), tolerance) {
      penalty -= 10000;
    }
  }

  for geyser in all_geysers {
    if is_between(location, depot_pos, geyser.get_position(), tolerance) {
      penalty -= 10000;
    }
  }

  penalty
}

fn score_depot_distance(location: Position, depot_pos: Position) -> i32 {
  let distance = calculate_distance(location, depot_pos);

  // Prefer locations 4-12 tiles away from depot
  if distance < 4.0 * 32.0 {
    -500 // Too close
  } else if distance > 12.0 * 32.0 {
    -300 // Too far
  } else {
    100 // Good distance
  }
}

fn penalize_depot_resource_paths(
  location: Position,
  resource_depots: &[Unit],
  all_minerals: &[Unit],
  all_geysers: &[Unit],
) -> i32 {
  let mut penalty = 0;
  let tolerance = 4.0 * 32.0;

  // Only penalize if location is between a depot and a resource
  for depot in resource_depots {
    let depot_pos = depot.get_position();

    for mineral in all_minerals {
      if is_between(location, depot_pos, mineral.get_position(), tolerance) {
        penalty -= 1000;
      }
    }

    for geyser in all_geysers {
      if is_between(location, depot_pos, geyser.get_position(), tolerance) {
        penalty -= 1000;
      }
    }
  }

  penalty
}

fn calculate_distance(pos1: Position, pos2: Position) -> f32 {
  let dx = (pos1.x - pos2.x) as f32;
  let dy = (pos1.y - pos2.y) as f32;
  (dx * dx + dy * dy).sqrt()
}

/// Check if point C is roughly between points A and B within a tolerance
fn is_between(c: Position, a: Position, b: Position, tolerance: f32) -> bool {
  // Calculate distances
  let ac_dx = (c.x - a.x) as f32;
  let ac_dy = (c.y - a.y) as f32;
  let dist_ac = (ac_dx * ac_dx + ac_dy * ac_dy).sqrt();

  let bc_dx = (c.x - b.x) as f32;
  let bc_dy = (c.y - b.y) as f32;
  let dist_bc = (bc_dx * bc_dx + bc_dy * bc_dy).sqrt();

  let ab_dx = (b.x - a.x) as f32;
  let ab_dy = (b.y - a.y) as f32;
  let dist_ab = (ab_dx * ab_dx + ab_dy * ab_dy).sqrt();

  // C is between A and B if AC + BC ≈ AB (within tolerance)
  let sum_dist = dist_ac + dist_bc;
  (sum_dist - dist_ab).abs() < tolerance
}

pub fn find_extractor_location(
  game: &Game,
  builder: &Unit,
  building_type: UnitType,
  base_locations: &[TilePosition],
  base_index: Option<usize>,
) -> Option<TilePosition> {
  let Some(player) = game.self_() else {
    println!("No player found for extractor location");
    return None;
  };

  let all_geysers = game.get_static_geysers();

  println!(
    "Looking for extractor location at base {:?}, total geysers: {}",
    base_index,
    all_geysers.len()
  );

  // Find the closest geyser that:
  // 1. Is reasonably close to the specified base (or any base if None)
  // 2. Doesn't already have an extractor on it
  // 3. Can be built on
  
  let target_base_pos = if let Some(idx) = base_index {
    base_locations.get(idx).map(|tile| Position::new(tile.x * 32 + 16, tile.y * 32 + 16))
  } else {
    None
  };

  let player_units = player.get_units();
  let resource_depots: Vec<_> = player_units
    .iter()
    .filter(|u| u.get_type().is_resource_depot())
    .collect();

  if resource_depots.is_empty() {
    println!("No resource depots found");
    return None;
  }

  println!("Found {} resource depots", resource_depots.len());

  let nearby_geysers: Vec<_> = all_geysers
    .iter()
    .filter(|geyser| {
      let geyser_pos = geyser.get_position();
      
      // If a specific base is requested, only check that base
      if let Some(base_pos) = target_base_pos {
        let dx = (base_pos.x - geyser_pos.x) as f32;
        let dy = (base_pos.y - geyser_pos.y) as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        return distance <= 12.0 * 32.0;
      }
      
      // Otherwise check if geyser is near any of our bases
      resource_depots.iter().any(|depot| {
        let depot_pos = depot.get_position();
        let dx = (depot_pos.x - geyser_pos.x) as f32;
        let dy = (depot_pos.y - geyser_pos.y) as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= 12.0 * 32.0
      })
    })
    .collect();

  println!("Found {} geysers near target base", nearby_geysers.len());

  nearby_geysers.iter().find_map(|geyser| {
    // Check if we can build on this geyser
    let geyser_tile = geyser.get_tile_position();
    let can_build = game.can_build_here(builder, geyser_tile, building_type, false);

    println!(
      "Checking geyser at ({}, {}): can_build = {:?}",
      geyser_tile.x, geyser_tile.y, can_build
    );

    if can_build.unwrap_or(false) {
      Some(geyser_tile)
    } else {
      None
    }
  })
}
