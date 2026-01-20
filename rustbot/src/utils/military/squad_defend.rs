use rsbwapi::*;

use crate::utils::{
  build_orders::build_order_item::BuildOrderItem,
  game_state::GameState,
  map_utils::{pathing, region_stuff},
};

pub fn defend_unit_control(game: &Game, unit: &Unit, defense_point: (i32, i32)) {
  let defense_position = Position::new(defense_point.0, defense_point.1);

  let enemies_near_defense =
    get_enemies_near_position(game, defense_position, 100.0, unit.get_player().get_id());

  if let Some(closest_enemy) = enemies_near_defense.first() {
    let unit_order = unit.get_order();
    let order_target = unit.get_target();

    // Validate and check if we're already attacking a valid target
    let mut already_attacking_valid_target = false;
    let order_target_id = order_target.and_then(|u: Unit| {
      let id = u.get_id();
      if id < 10000 {
        if unit_order == Order::AttackUnit {
          already_attacking_valid_target = true;
        }
        Some(id)
      } else {
        None
      }
    });

    // If we're already attacking a valid target, keep doing so
    if already_attacking_valid_target {
      return;
    }

    // Attack the closest enemy
    if unit_order != Order::AttackUnit || order_target_id != Some(closest_enemy.get_id()) {
      let _ = unit.attack(closest_enemy);
    }
  } else {
    // No enemies nearby, attack move to defense point
    let unit_order = unit.get_order();
    let order_target = unit.get_order_target_position();

    if unit_order != Order::AttackMove || order_target != Some(defense_position) {
      let _ = unit.attack(defense_position);
    }
  }
}

pub fn calculate_defense_point(
  game: &Game,
  game_state: &GameState,
  _self_player: &Player,
) -> Option<(i32, i32)> {
  let largest_hatchery_base_index = game_state
    .build_order
    .iter()
    .take(game_state.build_order_index)
    .filter_map(|item| {
      if let BuildOrderItem::Unit {
        unit_type,
        base_index,
      } = item
      {
        if *unit_type == UnitType::Zerg_Hatchery {
          return *base_index;
        }
      }
      None
    })
    .max()?;

  println!("defending from base index: {}", largest_hatchery_base_index);
  let furthest_base_with_hatchery = game_state.base_locations.get(largest_hatchery_base_index)?;

  let start_locations = game.get_start_locations();
  let Some(self_player) = game.self_() else {
    return None;
  };
  let my_start = start_locations.get(self_player.get_id() as usize)?;
  let enemy_base = start_locations
    .iter()
    .find(|&loc| loc != my_start)?;

  let path_to_enemy = pathing::get_path_between_points(
    game,
    (
      furthest_base_with_hatchery.x * 32,
      furthest_base_with_hatchery.y * 32,
    ),
    (enemy_base.x * 32, enemy_base.y * 32),
    None,
  )?;


  let nearest_chokepoint = region_stuff::chokepoint_along_path(game, &path_to_enemy)?;

  Some((nearest_chokepoint.x, nearest_chokepoint.y))
}

fn get_enemies_near_position(
  game: &Game,
  position: Position,
  radius: f32,
  player_id: usize,
) -> Vec<Unit> {
  let radius_squared = radius * radius;
  let mut enemies: Vec<(Unit, f32)> = game
    .get_all_units()
    .into_iter()
    .filter_map(|u| {
      if u.get_player().get_id() == player_id {
        return None;
      }

      // Only count units that can attack (have ground or air weapon)
      let unit_type = u.get_type();
      let ground_weapon = unit_type.ground_weapon();
      let air_weapon = unit_type.air_weapon();
      if ground_weapon == WeaponType::None && air_weapon == WeaponType::None {
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
