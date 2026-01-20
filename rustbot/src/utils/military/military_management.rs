use crate::utils::{
  game_state::GameState,
  map_utils::pathing,
  military::{
    avoid_enemy_movement_utils::{self, ThreatAvoidanceMode},
    squad_attack_workers::{self},
    squad_defend,
    squad_models::{MilitarySquad, SquadRole, SquadStatus},
    squad_mutas,
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
  game_state: &mut GameState,
) -> MilitarySquad {
  return match role {
    SquadRole::AttackAsMutas => MilitarySquad {
      name: name.to_string(),
      role,
      status,
      assigned_unit_ids: std::collections::HashSet::new(),
      target_position: None,
      target_path: None,
      target_path_index: None,
      leader_unit_id: None,
      required_units_near_leader: 5,
      unit_path_assignments: std::collections::HashMap::new(),
    },
    SquadRole::Defend => {
      let target_position = squad_defend::calculate_defense_point(game, game_state, self_player);

      if target_position.is_none() {
        println!(
          "No defense point found for defend squad {}, creating empty squad",
          name
        );
      }

      MilitarySquad {
        name: name.to_string(),
        role,
        status,
        assigned_unit_ids: std::collections::HashSet::new(),
        target_position,
        target_path: None,
        target_path_index: None,
        leader_unit_id: None,
        required_units_near_leader: 5,
        unit_path_assignments: std::collections::HashMap::new(),
      }
    }
    SquadRole::AttackWorkers => squad_attack_workers::attack_workers_squad(game, &self_player),
  };
}

fn update_squads(game: &Game, game_state: &mut GameState) {
  for squad in game_state.military_squads.iter_mut() {
    match squad.role {
      SquadRole::AttackAsMutas => {
        squad_mutas::muta_squad_control(game, squad);
      }
      SquadRole::Defend => {}
      SquadRole::AttackWorkers => {
        squad_attack_workers::update_attack_workers_squad(game, squad);
      }
    }
  }
}

fn enforce_military_assignments(game: &Game, game_state: &mut GameState) {
  for squad in game_state.military_squads.iter_mut() {
    let enemy_workers_close_to_squad = if let Some((target_x, target_y)) = squad.target_position {
      squad_attack_workers::get_worker_enemies_within(
        game,
        Position::new(target_x, target_y),
        400.0,
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
    SquadRole::AttackAsMutas => {
      squad_mutas::muta_unit_control(game, unit, squad);
    }
    SquadRole::Defend => {
      let Some((target_x, target_y)) = squad.target_position else {
        return;
      };
      squad_defend::defend_unit_control(game, unit, (target_x, target_y));
    }
    SquadRole::AttackWorkers => match squad.status {
      SquadStatus::Gathering => {
        let nearby_enemies = avoid_enemy_movement_utils::get_enemies_within(
          game,
          unit.get_position(),
          80.0,
          unit.get_player().get_id(),
        );
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
        if squad_attack_workers::attack_nearby_worker(game, unit, enemy_workers_close_to_squad) {
          return;
        }

        let Some((target_x, target_y)) = squad.target_position else {
          return;
        };

        // Only move to target if threat avoidance doesn't handle it
        let handled_by_threat_avoidance = avoid_enemy_movement_utils::handle_threat_avoidance(
          game,
          unit,
          Some((target_x, target_y)),
          ThreatAvoidanceMode::Aggressive,
        );

        if !handled_by_threat_avoidance {
          squad_attack_workers::move_to_target(unit, target_x, target_y);
        }
      }
    },
  }
}

pub fn draw_military_assignments(game: &Game, game_state: &GameState) {
  for squad in &game_state.military_squads {
    if let Some((target_x, target_y)) = squad.target_position {
      let target_pos = Position::new(target_x, target_y);
      game.draw_circle_map(target_pos, 8, Color::Red, false);
    }

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
            game.draw_line_map(unit_pos, order_target_pos, Color::Red);
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
