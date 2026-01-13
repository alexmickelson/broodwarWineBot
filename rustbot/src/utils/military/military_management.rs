use crate::utils::{
  game_state::{GameState, SharedGameState},
  map_utils::pathing,
  military::squad_models::{MilitarySquad, SquadRole, SquadStatus},
};
use rsbwapi::*;

pub fn military_onframe(game: &Game, game_state: &mut SharedGameState) {
  let Ok(mut game_state) = game_state.lock() else {
    println!("Failed to lock game_state in military_onframe");
    return;
  };

  let Some(self_player) = game.self_() else {
    println!("Could not get self player");
    return;
  };

  update_squads(game, &mut game_state);
  enforce_military_assignments(game, &mut game_state);
}

pub fn assign_unit_to_squad(game: &Game, unit: &Unit, game_state: &mut GameState) {
  let first_squad = game_state.military_squads.first_mut();
  if let Some(squad) = first_squad {
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

  let goal = path_to_enemy.iter().len() / 8;

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

pub fn update_squads(game: &Game, game_state: &mut GameState) {
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

    let squad_count = squad.assigned_unit_ids.len();
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
        if squad_count_close_to_target >= 10 {
          squad.status = SquadStatus::Attacking;

          if let Some(ref path) = squad.target_path {
            if let Some(index) = squad.target_path_index {
              if index > 0 && index <= path.len() {
                squad.target_position = Some(path[index - 1]);
              }
            }
          }
        }
      }
    }
  }
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
    SquadRole::AttackWorkers => match squad.status {
      SquadStatus::Gathering => {
        let nearby_enemies =
          get_enemies_within(game, unit.get_position(), 80.0, unit.get_player().get_id());
        if let Some(closest_enemy) = nearby_enemies.first() {
          let unit_order = unit.get_order();
          let order_target = unit.get_target();

          if unit_order == Order::AttackUnit {
            return;
          }

          if unit_order != Order::AttackUnit
            || order_target.map(|u| u.get_id()) != Some(closest_enemy.get_id())
          {
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
      SquadStatus::Attacking => {}
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

// fn update_military_assignments(my_military_units: &[Unit], game_state: &mut GameState) {
//   let Some(path_to_enemy) = &game_state.path_to_enemy_base else {
//     println!("No path to enemy base available for military assignment");
//     return;
//   };

//   let assigned_unit_ids: HashSet<usize> = game_state.military_squads.keys().copied().collect();

//   let unassigned_units: Vec<&Unit> = my_military_units
//     .iter()
//     .filter(|u| !assigned_unit_ids.contains(&(u.get_id() as usize)))
//     .collect();

//   let military_unit_count = my_military_units.len();

//   let unit_count_close_to_greatest_path_index =
//     count_units_near_furthest_unit(my_military_units, game_state);

//   let target_path_goal_index = if military_unit_count < 5 {
//     path_to_enemy.len() / 4
//   } else if military_unit_count < 15 || unit_count_close_to_greatest_path_index < 12 {
//     path_to_enemy.len() / 2
//   } else {
//     path_to_enemy.len() - 1
//   };

//   for unit in unassigned_units {
//     game_state.military_squads.insert(
//       unit.get_id() as usize,
//       MilitarySquad {
//         target_position: None,
//         target_unit: None,
//         target_path: Some(path_to_enemy.clone()),
//         target_path_current_index: Some(0),
//         target_path_goal_index: Some(target_path_goal_index),
//       },
//     );
//   }

//   for unit_id in assigned_unit_ids.iter() {
//     if let Some(assignment) = game_state.military_squads.get_mut(unit_id) {
//       if assignment.target_path.is_some() {
//         assignment.target_path_goal_index = Some(target_path_goal_index);
//         // If the new goal index is less than current index, reduce current index
//         if let Some(current_index) = assignment.target_path_current_index {
//           if target_path_goal_index < current_index {
//             assignment.target_path_current_index = Some(target_path_goal_index);
//           }
//         }
//       }
//     }
//   }

//   for unit_id in assigned_unit_ids {
//     if game_state.military_squads[&unit_id].target_path.is_some() {
//       let Some(unit) = my_military_units
//         .iter()
//         .find(|u| u.get_id() as usize == unit_id)
//       else {
//         println!(
//           "Could not find unit with id {} for military assignment update",
//           unit_id
//         );
//         continue;
//       };
//       update_path_assignment_if_close_to_goal(
//         &unit,
//         game_state.military_squads.get_mut(&unit_id).unwrap(),
//       );
//     }
//   }
// }

// fn count_units_near_furthest_unit(my_military_units: &[Unit], game_state: &GameState) -> usize {
//   let mut max_index: Option<usize> = None;
//   let mut furthest_unit_id: Option<usize> = None;

//   // Find the unit with the greatest current_index
//   for (unit_id, assignment) in game_state.military_squads.iter() {
//     if let (Some(current_index), Some(_path)) = (
//       assignment.target_path_current_index,
//       &assignment.target_path,
//     ) {
//       if max_index.is_none() || current_index > max_index.unwrap() {
//         max_index = Some(current_index);
//         furthest_unit_id = Some(*unit_id);
//       }
//     }
//   }

//   // Count units within 50 pixels of the furthest unit
//   if let Some(furthest_id) = furthest_unit_id {
//     if let Some(furthest_unit) = my_military_units
//       .iter()
//       .find(|u| u.get_id() as usize == furthest_id)
//     {
//       let furthest_pos = furthest_unit.get_position();
//       my_military_units
//         .iter()
//         .filter(|u| {
//           let pos = u.get_position();
//           let dx = (pos.x - furthest_pos.x) as f32;
//           let dy = (pos.y - furthest_pos.y) as f32;
//           let distance = (dx * dx + dy * dy).sqrt();
//           distance <= 50.0
//         })
//         .count()
//     } else {
//       0
//     }
//   } else {
//     0
//   }
// }

// fn update_path_assignment_if_close_to_goal(unit: &Unit, assignment: &mut MilitarySquad) {
//   let Some(goal_index) = assignment.target_path_goal_index else {
//     return;
//   };
//   let Some(path) = &assignment.target_path else {
//     return;
//   };
//   let Some(current_index) = assignment.target_path_current_index else {
//     return;
//   };

//   if current_index >= path.len() || goal_index >= path.len() {
//     return;
//   }

//   let unit_position = unit.get_position();
//   let current_position = Position::new(path[current_index].0, path[current_index].1);

//   let dx = (unit_position.x - current_position.x) as f32;
//   let dy = (unit_position.y - current_position.y) as f32;
//   let distance = (dx * dx + dy * dy).sqrt();
//   let close_enough_threshold = 30.0;
//   let advance_increment = 20;

//   if distance <= close_enough_threshold && current_index < goal_index {
//     let next_index = (current_index + advance_increment).min(goal_index);
//     assignment.target_path_current_index = Some(next_index);
//   }
//   if current_index > goal_index {
//     assignment.target_path_current_index = Some(goal_index);
//   }
// }

// fn get_offensive_target(game: &Game, self_player: &Player) -> Option<Position> {
//   let start_locations = game.get_start_locations();
//   let Some(self_start) = start_locations.get(self_player.get_id() as usize) else {
//     println!("Could not get self start location");
//     return None;
//   };

//   let self_start_pos = Position::new(self_start.x * 32 + 16, self_start.y * 32 + 16);
//   chokepoint_to_guard_base(game, &self_start_pos)
// }

pub fn draw_military_assignments(game: &Game, game_state: &GameState) {
  for squad in &game_state.military_squads {
    if let Some(target_path) = squad.target_path.as_ref() {
      pathing::draw_path(game, target_path);

      if let Some(index) = squad.target_path_index {
        if index < target_path.len() {
          let target_pos = Position::new(
            target_path[index].0,
            target_path[index].1,
          );
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

// fn enforce_attack_position_assignment(unit: &Unit, assignment: &MilitarySquad) {
//   let Some((target_x, target_y)) = assignment.target_position else {
//     return;
//   };

//   let target_position = Position::new(target_x, target_y);
//   let unit_order = unit.get_order();
//   let order_target = unit.get_order_target_position();

//   if unit_order != Order::AttackMove || order_target != Some(target_position) {
//     let _ = unit.attack(target_position);
//   }
// }

fn get_enemies_within(game: &Game, position: Position, radius: f32, player_id: usize) -> Vec<Unit> {
  let radius_squared = radius * radius;
  let mut enemies: Vec<(Unit, f32)> = game
    .get_all_units()
    .into_iter()
    .filter_map(|u| {
      if u.get_player().get_id() == player_id {
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

// fn enforce_path_following_assignment(game: &Game, unit: &Unit, assignment: &mut MilitarySquad) {
//   let Some(target_path_current_index) = assignment.target_path_current_index else {
//     return;
//   };
//   let Some(target_path_goal_index) = assignment.target_path_goal_index else {
//     return;
//   };
//   let Some(target_path) = &assignment.target_path else {
//     return;
//   };

//   let nearby_enemy_units =
//     get_enemies_within(game, unit.get_position(), 50.0, unit.get_player().get_id());

//   if let Some(closest_enemy) = nearby_enemy_units.first() {
//     let unit_order = unit.get_order();
//     let order_target = unit.get_target();
//     if unit_order != Order::AttackMove
//       || order_target.map(|u| u.get_id()) != Some(closest_enemy.get_id())
//     {
//       let _ = unit.attack(closest_enemy);
//     }
//     return;
//   }

//   if target_path_current_index >= target_path.len()
//     || target_path_current_index > target_path_goal_index
//   {
//     return;
//   }

//   let target_position = Position::new(
//     target_path[target_path_current_index].0,
//     target_path[target_path_current_index].1,
//   );

//   let unit_order = unit.get_order();
//   let order_target = unit.get_order_target_position();
//   if unit_order != Order::AttackMove || order_target != Some(target_position) {
//     let _ = unit.attack(target_position);
//   }
// }
