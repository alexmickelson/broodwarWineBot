use rsbwapi::{region::Region, *};
use std::collections::HashSet;
use std::sync::OnceLock;

static CACHED_REGION_IDS: OnceLock<Vec<i32>> = OnceLock::new();

pub fn get_all_region_ids(game: &Game) -> &'static Vec<i32> {
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

pub fn chokepoint_to_guard_base(game: &Game, base_location: &Position) -> Option<Position> {
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

pub fn draw_region_with_defense(game: &Game, region: Region) {
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

pub fn draw_region_boxes(game: &Game) {
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
