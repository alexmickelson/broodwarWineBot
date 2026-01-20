use rsbwapi::*;

pub fn get_base_locations_ordered(
  game: &Game,
  debug_lines: &mut Vec<(Position, Position, Color)>,
) -> Vec<TilePosition> {
  let Some(player) = game.self_() else {
    println!("No player found in get_base_locations_ordered");
    return Vec::new();
  };

  let start_locations = game.get_start_locations();
  let Some(&start_tile) = start_locations.get(player.get_id() as usize) else {
    println!("No start location found for player");
    return Vec::new();
  };

  let all_minerals = game.get_static_minerals();
  let all_geysers = game.get_static_geysers();

  let resource_clusters = cluster_resources(&all_minerals, &all_geysers);

  let mut base_locations: Vec<(TilePosition, f32)> = resource_clusters
    .into_iter()
    .filter_map(|cluster| {
      let hatchery_location = find_best_hatchery_location(game, &cluster, debug_lines)?;

      let start_pos = Position::new(start_tile.x * 32, start_tile.y * 32);
      let distance = distance_to_hatchery_edge(hatchery_location, start_pos) as f32;

      Some((hatchery_location, distance))
    })
    .collect();

  base_locations.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
  base_locations.into_iter().map(|(tile, _)| tile).collect()
}

fn cluster_resources(minerals: &[Unit], geysers: &[Unit]) -> Vec<Vec<Unit>> {
  let mut all: Vec<Unit> = minerals.iter().chain(geysers).cloned().collect();
  let mut clusters: Vec<Vec<Unit>> = Vec::new();

  const CLUSTER_DIST: i32 = 9 * 32;

  while let Some(seed) = all.pop() {
    let mut cluster = vec![seed];
    let mut i = 0;

    while i < cluster.len() {
      let p = cluster[i].get_position();
      let mut j = 0;

      while j < all.len() {
        let dx = (p.x - all[j].get_position().x) as f32;
        let dy = (p.y - all[j].get_position().y) as f32;
        let distance = (dx * dx + dy * dy).sqrt() as i32;

        if distance < CLUSTER_DIST {
          cluster.push(all.swap_remove(j));
        } else {
          j += 1;
        }
      }
      i += 1;
    }
    clusters.push(cluster);
  }
  clusters
}

fn closest_point_on_hatchery(tile: TilePosition, point: Position) -> Position {
  const TILE_SIZE: i32 = 32;
  const HATCHERY_WIDTH: i32 = 4 * TILE_SIZE;
  const HATCHERY_HEIGHT: i32 = 3 * TILE_SIZE;

  let left = tile.x * TILE_SIZE;
  let right = tile.x * TILE_SIZE + HATCHERY_WIDTH;
  let top = tile.y * TILE_SIZE;
  let bottom = tile.y * TILE_SIZE + HATCHERY_HEIGHT;

  let closest_x = point.x.clamp(left, right);
  let closest_y = point.y.clamp(top, bottom);

  Position::new(closest_x, closest_y)
}

fn distance_to_hatchery_edge(tile: TilePosition, point: Position) -> i32 {
  let closest_point = closest_point_on_hatchery(tile, point);

  let dx = (point.x - closest_point.x) as f32;
  let dy = (point.y - closest_point.y) as f32;

  (dx * dx + dy * dy).sqrt() as i32
}

fn bounding_box(
  resources: &[Unit],
  padding_tiles: i32,
  game: &Game,
) -> (TilePosition, TilePosition) {
  let mut min_x = i32::MAX;
  let mut min_y = i32::MAX;
  let mut max_x = i32::MIN;
  let mut max_y = i32::MIN;

  for r in resources {
    let t = r.get_tile_position();
    min_x = min_x.min(t.x);
    min_y = min_y.min(t.y);
    max_x = max_x.max(t.x);
    max_y = max_y.max(t.y);
  }

  let map_width = game.map_width();
  let map_height = game.map_height();

  (
    TilePosition {
      x: (min_x - padding_tiles).max(0),
      y: (min_y - padding_tiles).max(0),
    },
    TilePosition {
      x: (max_x + padding_tiles).min(map_width - 1),
      y: (max_y + padding_tiles).min(map_height - 1),
    },
  )
}

fn is_hatchery_too_close_to_resources(
  tile: TilePosition,
  resources: &[Unit],
  min_mineral_distance: i32,
  min_geyser_distance: i32,
) -> bool {
  const TILE_SIZE: i32 = 32;
  const HATCHERY_WIDTH: i32 = 4 * TILE_SIZE;
  const HATCHERY_HEIGHT: i32 = 3 * TILE_SIZE;

  let left = tile.x * TILE_SIZE;
  let right = tile.x * TILE_SIZE + HATCHERY_WIDTH;
  let top = tile.y * TILE_SIZE;
  let bottom = tile.y * TILE_SIZE + HATCHERY_HEIGHT;

  for resource in resources {
    let resource_pos = resource.get_position();
    let resource_tile = resource.get_tile_position();

    let resource_overlaps = resource_tile.x < tile.x + 4
      && resource_tile.x >= tile.x
      && resource_tile.y < tile.y + 3
      && resource_tile.y >= tile.y;

    if resource_overlaps {
      return true;
    }

    let closest_x = resource_pos.x.clamp(left, right);
    let closest_y = resource_pos.y.clamp(top, bottom);

    let dx = (resource_pos.x - closest_x) as f32;
    let dy = (resource_pos.y - closest_y) as f32;
    let distance = (dx * dx + dy * dy).sqrt() as i32;

    let min_distance = if resource.get_type().is_refinery() {
      min_geyser_distance
    } else {
      min_mineral_distance
    };

    if distance < min_distance {
      return true;
    }
  }

  false
}

fn find_best_hatchery_location(
  game: &Game,
  resources: &[Unit],
  _debug_lines: &mut Vec<(Position, Position, Color)>,
) -> Option<TilePosition> {
  if resources.is_empty() {
    return None;
  }

  const MIN_MINERAL_DISTANCE: i32 = 128;
  const MIN_GEYSER_DISTANCE: i32 = 128;

  let (min_t, max_t) = bounding_box(resources, 15, game);

  let mut best_tile = None;
  let mut best_score = i32::MAX;
  let mut best_geyser_distance = i32::MAX;
  let mut candidate_positions: Vec<(TilePosition, i32)> = Vec::new();

  for y in min_t.y..=max_t.y {
    for x in min_t.x..=max_t.x {
      let tile = TilePosition { x, y };

      // Check if all tiles in the 4x3 hatchery footprint are buildable
      let mut all_buildable = true;
      for dy in 0..3 {
        for dx in 0..4 {
          let check_tile = TilePosition {
            x: tile.x + dx,
            y: tile.y + dy,
          };
          if !game.is_buildable(check_tile) {
            all_buildable = false;
            break;
          }
        }
        if !all_buildable {
          break;
        }
      }

      if !all_buildable {
        continue;
      }

      if is_hatchery_too_close_to_resources(
        tile,
        resources,
        MIN_MINERAL_DISTANCE,
        MIN_GEYSER_DISTANCE,
      ) {
        continue;
      }

      // Check if the game allows building here (this is the authoritative check)
      match game.can_build_here(None, tile, UnitType::Zerg_Hatchery, false) {
        Ok(true) => {
          // Location is buildable, continue with scoring
        }
        Ok(false) => {
          // Location is not buildable
          continue;
        }
        Err(e) => {
          println!("Error checking buildability at tile {:?}: {:?}", tile, e);
          // Error checking buildability, skip this location
          continue;
        }
      }

      let mut score = 0;
      let mut geyser_distance = 0;

      for r in resources {
        let d = distance_to_hatchery_edge(tile, r.get_position());

        if r.get_type().is_refinery() {
          score += d * 2;
          geyser_distance += d;
        } else {
          score += d;
        }
      }

      candidate_positions.push((tile, score));

      if score < best_score || (score == best_score && geyser_distance < best_geyser_distance) {
        best_score = score;
        best_geyser_distance = geyser_distance;
        best_tile = Some(tile);
      }
    }
  }

  let mut sorted_candidates = candidate_positions.clone();
  sorted_candidates.sort_by_key(|(_, score)| *score);

  best_tile
}
