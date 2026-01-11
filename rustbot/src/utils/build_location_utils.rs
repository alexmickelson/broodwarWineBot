use rand::seq::SliceRandom;
use rand::thread_rng;
use rsbwapi::*;

pub fn get_buildable_location(
  game: &Game,
  builder: &Unit,
  unit_type: UnitType,
) -> Option<TilePosition> {
  let builder_pos = builder.get_position();
  let search_radius = 10; // in tiles

  let buildable_locations: Vec<TilePosition> = (-search_radius..=search_radius)
    .flat_map(|dy| {
      (-search_radius..=search_radius).filter_map(move |dx| {
        let tile_pos = TilePosition {
          x: builder_pos.x / 32 + dx,
          y: builder_pos.y / 32 + dy,
        };

        if game
          .can_build_here(builder, tile_pos, unit_type, true)
          .unwrap_or_else(|_| false)
          && is_not_in_resource_line(game, tile_pos, unit_type)
        {
          Some(tile_pos)
        } else {
          None
        }
      })
    })
    .collect();

  let mut rng = thread_rng();
  buildable_locations.choose(&mut rng).copied()
}

fn is_not_in_resource_line(game: &Game, pos: TilePosition, building_type: UnitType) -> bool {
  let Some(player) = game.self_() else {
    return true;
  };

  let player_units = player.get_units();
  let resource_depots: Vec<_> = player_units
    .iter()
    .filter(|u| u.get_type().is_resource_depot())
    .collect();

  if resource_depots.is_empty() {
    return true;
  }

  let build_center = get_building_center(pos, building_type);

  let blocks_mineral_path = resource_depots.iter().any(|depot| {
    let depot_pos = depot.get_position();
    
    game
      .get_static_minerals()
      .iter()
      .filter(|m| is_within_range(depot_pos, m.get_position(), 10.0 * 32.0))
      .any(|mineral| {
        is_point_between(
          depot_pos.x,
          depot_pos.y,
          mineral.get_position().x,
          mineral.get_position().y,
          build_center.x,
          build_center.y,
          64.0,
        )
      })
  });

  if blocks_mineral_path {
    return false;
  }

  let blocks_geyser_path = resource_depots.iter().any(|depot| {
    let depot_pos = depot.get_position();
    
    game
      .get_static_geysers()
      .iter()
      .filter(|g| is_within_range(depot_pos, g.get_position(), 12.0 * 32.0))
      .any(|geyser| {
        is_point_between(
          depot_pos.x,
          depot_pos.y,
          geyser.get_position().x,
          geyser.get_position().y,
          build_center.x,
          build_center.y,
          64.0,
        )
      })
  });

  !blocks_geyser_path
}

fn get_building_center(pos: TilePosition, building_type: UnitType) -> Position {
  let width = building_type.tile_width() as i32;
  let height = building_type.tile_height() as i32;
  let build_left = pos.x * 32;
  let build_top = pos.y * 32;
  let build_right = build_left + width * 32;
  let build_bottom = build_top + height * 32;

  Position {
    x: (build_left + build_right) / 2,
    y: (build_top + build_bottom) / 2,
  }
}

fn is_within_range(pos1: Position, pos2: Position, max_distance: f32) -> bool {
  let dx = (pos2.x - pos1.x) as f32;
  let dy = (pos2.y - pos1.y) as f32;
  let distance = (dx * dx + dy * dy).sqrt();
  distance <= max_distance
}

fn is_point_between(x1: i32, y1: i32, x2: i32, y2: i32, px: i32, py: i32, threshold: f32) -> bool {
  let line_dx = (x2 - x1) as f32;
  let line_dy = (y2 - y1) as f32;
  let line_length_sq = line_dx * line_dx + line_dy * line_dy;

  if line_length_sq < 1.0 {
    return false;
  }

  // Calculate projection parameter t
  let t = ((px - x1) as f32 * line_dx + (py - y1) as f32 * line_dy) / line_length_sq;

  // Check if point projects onto the line segment
  if t < 0.0 || t > 1.0 {
    return false;
  }

  // Calculate closest point on line segment
  let closest_x = x1 as f32 + t * line_dx;
  let closest_y = y1 as f32 + t * line_dy;

  // Calculate perpendicular distance
  let dist_dx = px as f32 - closest_x;
  let dist_dy = py as f32 - closest_y;
  let distance = (dist_dx * dist_dx + dist_dy * dist_dy).sqrt();

  distance < threshold
}
