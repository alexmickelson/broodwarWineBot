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

  let is_extractor = unit_type == UnitType::Zerg_Extractor
    || unit_type == UnitType::Terran_Refinery
    || unit_type == UnitType::Protoss_Assimilator;
  if is_extractor {
    return find_extractor_location(game, builder, unit_type);
  }

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

pub fn find_extractor_location(
  game: &Game,
  builder: &Unit,
  building_type: UnitType,
) -> Option<TilePosition> {
  let Some(player) = game.self_() else {
    println!("No player found for extractor location");
    return None;
  };

  let all_geysers = game.get_static_geysers();

  println!(
    "Looking for extractor location, total geysers: {}",
    all_geysers.len()
  );

  // Find the closest geyser that:
  // 1. Is reasonably close to our base
  // 2. Doesn't already have an extractor on it
  // 3. Can be built on
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
      // Check if geyser is near any of our bases
      resource_depots.iter().any(|depot| {
        let depot_pos = depot.get_position();
        let geyser_pos = geyser.get_position();
        let dx = (depot_pos.x - geyser_pos.x) as f32;
        let dy = (depot_pos.y - geyser_pos.y) as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= 12.0 * 32.0
      })
    })
    .collect();

  println!("Found {} geysers near bases", nearby_geysers.len());

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
