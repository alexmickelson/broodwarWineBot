use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitInfo {
  pub x: i32,
  pub y: i32,
  pub unit_type: String,
  pub is_ally: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceInfo {
  pub x: i32,
  pub y: i32,
  pub resource_type: String,
  pub amount: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapData {
  pub width: usize,
  pub height: usize,
  pub walkability: Vec<Vec<bool>>, // true if walkable
  pub explored: Vec<Vec<bool>>,    // true if explored
  pub units: Vec<UnitInfo>,
  pub resources: Vec<ResourceInfo>,
}

impl Default for MapData {
  fn default() -> Self {
    Self {
      width: 0,
      height: 0,
      walkability: Vec::new(),
      explored: Vec::new(),
      units: Vec::new(),
      resources: Vec::new(),
    }
  }
}

pub fn collect_map_data(game: &rsbwapi::Game) -> MapData {
  // Map dimensions in build tiles (32 pixels per tile)
  let width_tiles = game.map_width() as usize;
  let height_tiles = game.map_height() as usize;
  
  // For visualization, we'll use walk tile resolution (4 walk tiles per build tile)
  let width = width_tiles * 4;
  let height = height_tiles * 4;

  let mut walkability = vec![vec![false; width]; height];
  let mut explored = vec![vec![false; width]; height];

  for y in 0..height {
    for x in 0..width {
      let walk_pos = rsbwapi::WalkPosition {
        x: x as i32,
        y: y as i32,
      };
      let tile_pos = rsbwapi::TilePosition {
        x: (x / 4) as i32,
        y: (y / 4) as i32,
      };
      walkability[y][x] = game.is_walkable(walk_pos);
      explored[y][x] = game.is_explored(tile_pos);
    }
  }

  let mut units = Vec::new();
  let mut resources = Vec::new();

  if let Some(player) = game.self_() {
    for unit in game.get_all_units() {
      let pos = unit.get_position();
      let walk_pos = pos.to_walk_position();

      if unit.get_type().is_resource_container() {
        resources.push(ResourceInfo {
          x: walk_pos.x,
          y: walk_pos.y,
          resource_type: format!("{:?}", unit.get_type()),
          amount: unit.get_resources(),
        });
      } else {
        let unit_player = unit.get_player();
        let is_ally = unit_player == player || player.is_ally(&unit_player);
        units.push(UnitInfo {
          x: walk_pos.x,
          y: walk_pos.y,
          unit_type: format!("{:?}", unit.get_type()),
          is_ally,
        });
      }
    }
  }

  MapData {
    width,
    height,
    walkability,
    explored,
    units,
    resources,
  }
}
