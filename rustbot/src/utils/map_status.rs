use crate::status_webserver::{MapData, ResourceInfo, UnitInfo};
use crate::utils::game_status::SharedStatus;
use rsbwapi::*;

pub fn update_map_data(game: &Game, status: &SharedStatus) {
    let map_width = game.map_width() as usize * 4; // Convert from build tiles to walk tiles
    let map_height = game.map_height() as usize * 4;

    let mut walkability = vec![vec![false; map_width]; map_height];
    let mut explored = vec![vec![false; map_width]; map_height];

    for y in 0..map_height {
        for x in 0..map_width {
            let walk_pos = WalkPosition {
                x: x as i32,
                y: y as i32,
            };

            walkability[y][x] = game.is_walkable(walk_pos);
            explored[y][x] = game.is_explored(walk_pos.to_tile_position());
        }
    }

    let mut units = Vec::new();

    if let Some(self_player) = game.self_() {
        for unit in self_player.get_units() {
            units.push(UnitInfo {
                x: unit.get_position().x,
                y: unit.get_position().y,
                unit_type: format!("{:?}", unit.get_type()),
                is_ally: true,
            });
        }
    }

    for player in game.enemies() {
        for unit in player.get_units() {
            if unit.exists() {
                units.push(UnitInfo {
                    x: unit.get_position().x,
                    y: unit.get_position().y,
                    unit_type: format!("{:?}", unit.get_type()),
                    is_ally: false,
                });
            }
        }
    }

    let mut resources = Vec::new();
    for mineral in game.get_static_minerals() {
        if mineral.exists() {
            resources.push(ResourceInfo {
                x: mineral.get_position().x,
                y: mineral.get_position().y,
                resource_type: "minerals".to_string(),
                amount: mineral.get_resources(),
            });
        }
    }

    for geyser in game.get_static_geysers() {
        if geyser.exists() {
            resources.push(ResourceInfo {
                x: geyser.get_position().x,
                y: geyser.get_position().y,
                resource_type: "gas".to_string(),
                amount: geyser.get_resources(),
            });
        }
    }

    if let Ok(mut status) = status.lock() {
        status.map_data = MapData {
            width: map_width,
            height: map_height,
            walkability,
            explored,
            units,
            resources,
        };
    }
}
