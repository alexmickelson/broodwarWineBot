use crate::utils::{
  game_state::GameState,
  map_utils::pathing,
  military::{
    attack_workers_squad,
    squad_models::{MilitarySquad, SquadRole, SquadStatus},
  },
};
use rsbwapi::*;

pub fn military_onframe(game: &Game, game_state: &mut GameState) {
  update_squads(game, game_state);
  enforce_military_assignments(game, game_state);
}

pub fn create_initial_squad(game: &Game) -> Option<MilitarySquad> {
  let Some(self_player) = game.self_() else {
    return None;
  };

  let start_locations = game.get_start_locations();
  let Some(my_starting_position) = start_locations.get(self_player.get_id() as usize) else {
    return None;
  };

  let Some(enemy_location) = start_locations
    .iter()
    .find(|&loc| loc != my_starting_position)
  else {
    return None;
  };

  let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
  let enemy_pos = (enemy_location.x * 32, enemy_location.y * 32);

  let path_to_enemy = pathing::get_path_between_points(game, my_pos, enemy_pos);

  let goal = if let Some(ref path) = path_to_enemy {
    path.len() / 5
  } else {
    println!("No path to enemy found when creating initial squad");
    0
  };

  Some(MilitarySquad {
    name: "Main Squad".to_string(),
    role: SquadRole::AttackWorkers,
    status: SquadStatus::Gathering,
    assigned_unit_ids: std::collections::HashSet::new(),
    target_position: None,
    target_path: path_to_enemy,
    target_path_index: Some(goal),
  })
}

pub fn create_attack_squad(game: &Game) -> Option<MilitarySquad> {
  let Some(self_player) = game.self_() else {
    return None;
  };

  let start_locations = game.get_start_locations();
  let Some(my_starting_position) = start_locations.get(self_player.get_id() as usize) else {
    return None;
  };

  let Some(enemy_location) = start_locations
    .iter()
    .find(|&loc| loc != my_starting_position)
  else {
    return None;
  };

  println!(
    "Creating attack squad with target enemy base at ({}, {})",
    enemy_location.x, enemy_location.y
  );

  let my_pos = (my_starting_position.x * 32, my_starting_position.y * 32);
  let enemy_pos = (enemy_location.x * 32, enemy_location.y * 32);

  let path_to_enemy = pathing::get_path_between_points(game, my_pos, enemy_pos);

  let target_index = if let Some(ref path) = path_to_enemy {
    Some(path.len() - 1)
  } else {
    println!("No path to enemy found when creating attack squad");
    None
  };

  Some(MilitarySquad {
    name: "Attack Squad".to_string(),
    role: SquadRole::Attack,
    status: SquadStatus::Attacking,
    assigned_unit_ids: std::collections::HashSet::new(),
    target_position: path_to_enemy
      .as_ref()
      .and_then(|p| target_index.map(|i| p[i])),
    target_path: path_to_enemy,
    target_path_index: target_index,
  })
}

pub fn update_squads(game: &Game, game_state: &mut GameState) {
  let mut new_squads_to_add: Vec<MilitarySquad> = Vec::new();

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
        if squad_count_close_to_target < 6 {
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

        if squad.status == SquadStatus::Attacking {
          continue;
        }

        squad.status = SquadStatus::Attacking;
        squad.target_path_index = Some(path.len() - 1);
        squad.target_position = Some(path[path.len() - 1]);

        // Create new attack squad
        if let Some(new_squad) = create_attack_squad(game) {
          new_squads_to_add.push(new_squad);
        }
      }
    }
  }

  // Add new squads after iteration
  game_state.military_squads.extend(new_squads_to_add);
}

fn enforce_military_assignments(game: &Game, game_state: &mut GameState) {
  for squad in game_state.military_squads.iter() {
    for &unit_id in &squad.assigned_unit_ids {
      let Some(unit) = game.get_unit(unit_id) else {
        continue;
      };
      unit_in_squad_control(game, &unit, squad);
    }
  }
}

fn unit_in_squad_control(game: &Game, unit: &Unit, squad: &MilitarySquad) {
  match squad.role {
    SquadRole::Attack => {}
    SquadRole::Defend => {}
    SquadRole::AttackWorkers => attack_workers_squad::control_unit_in_squad(game, unit, squad),
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

      if let Some((target_x, target_y)) = squad.target_position {
        let unit_pos = unit.get_position();
        let target_pos = Position::new(target_x, target_y);
        game.draw_line_map(unit_pos, target_pos, Color::Red);
      }
    }
  }
}

pub fn assign_unit_to_squad(game: &Game, unit: &Unit, player: &Player, game_state: &mut GameState) {
  if unit.get_player().get_id() != player.get_id() {
    return;
  }

  let last_squad: Option<&mut MilitarySquad> = game_state.military_squads.last_mut();
  if let Some(squad) = last_squad {
    squad.assigned_unit_ids.insert(unit.get_id() as usize);
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
  let unit_id = unit.get_id() as usize;
  for squad in game_state.military_squads.iter_mut() {
    let _ = squad.assigned_unit_ids.remove(&unit_id);
  }
}
