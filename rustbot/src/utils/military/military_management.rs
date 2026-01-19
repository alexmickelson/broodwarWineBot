use crate::utils::{
  game_state::GameState,
  map_utils::pathing,
  military::{
    squad_attack_workers::{
      self, attack_nearby_worker, create_initial_attack_workers_squad, get_enemies_within,
      get_worker_enemies_within, move_to_target, ThreatAvoidanceMode,
    },
    squad_models::{MilitarySquad, SquadRole, SquadStatus},
  },
};
use rsbwapi::*;

pub fn military_onframe(game: &Game, game_state: &mut GameState) {
  update_squads(game, game_state);
  enforce_military_assignments(game, game_state);
}

pub fn assign_unit_to_squad(game: &Game, unit: &Unit, game_state: &mut GameState) {
  let last_squad = game_state.military_squads.last_mut();
  if let Some(squad) = last_squad {
    squad.assigned_unit_ids.insert(unit.get_id());
    return;
  }

  game.draw_text_screen((0, 50), "no squads available to assign unit");
}

pub fn is_military_unit(unit: &Unit) -> bool {
  if unit.get_type().is_building()
    || unit.get_type() == UnitType::Zerg_Larva
    || unit.get_type() == UnitType::Zerg_Egg
    || unit.get_type() == UnitType::Zerg_Drone
    || unit.get_type() == UnitType::Zerg_Overlord
  {
    return false;
  }
  true
}

pub fn remove_unit_from_squads(unit: &Unit, game_state: &mut GameState) {
  let unit_id = unit.get_id();
  for squad in game_state.military_squads.iter_mut() {
    let _ = squad.assigned_unit_ids.remove(&unit_id);
  }
}

pub fn create_squad(
  game: &Game,
  name: &str,
  role: SquadRole,
  status: SquadStatus,
  self_player: &Player,
) -> MilitarySquad {
  return match role {
    SquadRole::Attack | SquadRole::Defend => MilitarySquad {
      name: name.to_string(),
      role,
      status,
      assigned_unit_ids: std::collections::HashSet::new(),
      target_position: None,
      target_path: None,
      target_path_index: None,
    },
    SquadRole::AttackWorkers => {
      squad_attack_workers::create_initial_attack_workers_squad(game, &self_player)
    }
  };
}

fn update_squads(game: &Game, game_state: &mut GameState) {
  for squad in game_state.military_squads.iter_mut() {
    if let (Some(ref path), Some(index)) = (&squad.target_path, squad.target_path_index) {
      if index < path.len() {
        squad.target_position = Some(path[index]);
      }
    }

    if squad.target_position.is_none() {
      println!(
        "Squad {} has no target position, skipping update",
        squad.name
      );
      continue;
    }

    let squad_units: Vec<Unit> = squad
      .assigned_unit_ids
      .iter()
      .filter_map(|&unit_id| game.get_unit(unit_id))
      .collect();
    let squad_count_close_to_target =
      get_units_close_to_position(&squad_units, squad.target_position.unwrap(), 80.0);

    match squad.role {
      SquadRole::Attack => {}
      SquadRole::Defend => {}
      SquadRole::AttackWorkers => {
        if squad_count_close_to_target < 4 {
          // println!(
          //   "Squad {} not ready to attack: {} units close to target (need 6)",
          //   squad.name, squad_count_close_to_target
          // );
          continue;
        }

        let Some(ref path) = squad.target_path else {
          println!(
            "Squad {} cannot switch to attacking: no target path",
            squad.name
          );
          continue;
        };

        if path.is_empty() {
          println!(
            "Squad {} cannot switch to attacking: path is empty",
            squad.name
          );
          continue;
        }

        squad.status = SquadStatus::Attacking;
        squad.target_path_index = Some(path.len() - 1);
        squad.target_position = Some(path[path.len() - 1]);
      }
    }
  }
}

fn enforce_military_assignments(game: &Game, game_state: &mut GameState) {
  for squad in game_state.military_squads.iter_mut() {
    let enemy_workers_close_to_squad = if let Some((target_x, target_y)) = squad.target_position {
      get_worker_enemies_within(
        game,
        Position::new(target_x, target_y),
        200.0,
        game.self_().map_or(0, |p| p.get_id()),
      )
    } else {
      vec![]
    };

    let unit_ids: Vec<usize> = squad.assigned_unit_ids.iter().copied().collect();
    for unit_id in unit_ids {
      let Some(unit) = game.get_unit(unit_id) else {
        continue;
      };
      unit_in_squad_control(game, &unit, squad, &enemy_workers_close_to_squad);
    }
  }
}

fn unit_in_squad_control(
  game: &Game,
  unit: &Unit,
  squad: &mut MilitarySquad,
  enemy_workers_close_to_squad: &[Unit],
) {
  match squad.role {
    SquadRole::Attack => {}
    SquadRole::Defend => {}
    SquadRole::AttackWorkers => match squad.status {
      SquadStatus::Gathering => {
        let nearby_enemies =
          get_enemies_within(game, unit.get_position(), 80.0, unit.get_player().get_id());
        if let Some(closest_enemy) = nearby_enemies.first() {
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

          if unit_order != Order::AttackUnit || order_target_id != Some(closest_enemy.get_id()) {
            let _ = unit.attack(closest_enemy);
          }
          return;
        } else {
          let Some((target_x, target_y)) = squad.target_position else {
            return;
          };
          let target_position = Position::new(target_x, target_y);
          let unit_order = unit.get_order();
          let order_target = unit.get_order_target_position();
          if unit_order != Order::AttackMove || order_target != Some(target_position) {
            let _ = unit.attack(target_position);
          }
        }
      }
      SquadStatus::Attacking => {
        if attack_nearby_worker(game, unit, enemy_workers_close_to_squad) {
          return;
        }

        let Some((target_x, target_y)) = squad.target_position else {
          return;
        };

        squad_attack_workers::handle_threat_avoidance(
          game,
          unit,
          Some((target_x, target_y)),
          ThreatAvoidanceMode::Aggressive,
        );

        move_to_target(unit, target_x, target_y);
      }
    },
  }
}

fn get_units_close_to_position(units: &[Unit], position: (i32, i32), radius: f32) -> usize {
  let pos = Position::new(position.0, position.1);
  let radius_squared = radius * radius;

  units
    .iter()
    .filter(|u| {
      let unit_pos = u.get_position();
      let dx = (unit_pos.x - pos.x) as f32;
      let dy = (unit_pos.y - pos.y) as f32;
      let distance_squared = dx * dx + dy * dy;
      distance_squared <= radius_squared
    })
    .count()
}

pub fn draw_military_assignments(game: &Game, game_state: &GameState) {
  for squad in &game_state.military_squads {
    if let Some(target_path) = squad.target_path.as_ref() {
      pathing::draw_path(game, target_path);

      if let Some(index) = squad.target_path_index {
        if index < target_path.len() {
          let target_pos = Position::new(target_path[index].0, target_path[index].1);
          game.draw_circle_map(target_pos, 10, Color::Red, false);
        }
      }
    }

    for &unit_id in &squad.assigned_unit_ids {
      let Some(unit) = game.get_unit(unit_id) else {
        continue;
      };

      // Draw current order
      let unit_pos = unit.get_position();
      let unit_order = unit.get_order();

      match unit_order {
        Order::Move | Order::AttackMove => {
          if let Some(order_target_pos) = unit.get_order_target_position() {
            game.draw_line_map(unit_pos, order_target_pos, Color::Cyan);
          }
        }
        Order::AttackUnit => {
          if let Some(target_unit) = unit.get_target() {
            // Validate the target unit before using it
            let id = target_unit.get_id();
            if id < 10000 {
              game.draw_line_map(unit_pos, target_unit.get_position(), Color::Yellow);
            }
          }
        }
        _ => {}
      }

      if let Some((target_x, target_y)) = squad.target_position {
        let target_pos = Position::new(target_x, target_y);
        game.draw_line_map(unit_pos, target_pos, Color::Red);
      }
    }
  }
}
