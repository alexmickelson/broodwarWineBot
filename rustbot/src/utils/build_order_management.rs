use crate::utils::building_stuff::{creature_stuff, structure_stuff};
use crate::utils::game_state::{BuildOrderItem, GameState};
use rsbwapi::*;

pub fn build_order_on_unit_started(game: &Game, completed_unit: &Unit, game_state: &mut GameState) {
  let Some(player) = game.self_() else {
    println!("Failed to get self player in build_order_on_unit_started");
    return;
  };

  if completed_unit.get_player().get_id() != player.get_id() {
    println!(
      "Unit started that does not belong to us: unit id {}, type {:?}",
      completed_unit.get_id(),
      completed_unit.get_type()
    );
    return;
  }

  let Some(current_build_order_item) = game_state
    .build_order
    .get(game_state.build_order_index)
    .cloned()
  else {
    println!("Build order empty in build_order_on_unit_create");
    return;
  };

  match current_build_order_item {
    BuildOrderItem::Unit(unit_type) => {
      if completed_unit.get_type() == unit_type {
        println!(
          "Completed build order item: {:?} (unit created), moving to next item",
          current_build_order_item
        );
        game_state.build_order_index += 1;
        make_assignment_for_current_build_order_item(game, game_state);
      }
    }
    BuildOrderItem::Upgrade(_) => {}
  }
}

pub fn make_assignment_for_current_build_order_item(game: &Game, game_state: &mut GameState) {
  let Some(player) = game.self_() else {
    println!("Failed to get self player in make_assignment_for_current_build_order_item");
    return;
  };

  if game_state.build_order_index >= game_state.build_order.len() {
    println!("build order empty in make_assignment_for_current_build_order_item");
    return;
  }
  let thing_to_build = game_state.build_order[game_state.build_order_index].clone();

  if let BuildOrderItem::Unit(unit_to_build) = thing_to_build {
    if unit_to_build.is_building() {
      structure_stuff::make_building_assignment(game, game_state, unit_to_build);
    } else {
      creature_stuff::assign_larva_to_build_current_index(game, game_state, &player);
    }
  }
}

pub fn build_order_enforce_assignments(game: &Game, game_state: &mut GameState) {
  if game_state.build_order_index >= game_state.build_order.len() {
    println!("nothing to build");
    return;
  }

  let thing_to_build = game_state.build_order[game_state.build_order_index].clone();

  match thing_to_build {
    BuildOrderItem::Unit(unit_type) => {
      if unit_type.is_building() {
        // verify drone or building
      } else {
        enforce_larvae_assignment(game, game_state);
      }
    }
    BuildOrderItem::Upgrade(_) => {
      // Upgrades are handled elsewhere
    }
  }
}

fn enforce_larvae_assignment(game: &Game, game_state: &mut GameState) {
  let larvae_assigned_for_current_index =
    game_state
      .larva_responsibilities
      .iter()
      .find_map(|(&larvae_id, build_idx)| {
        if *build_idx == game_state.build_order_index {
          let maybe_larvae = game.get_unit(larvae_id);

          let Some(l) = maybe_larvae else {
            return None;
          };
          Some(l)
        } else {
          None
        }
      });

  let Some(larvae) = larvae_assigned_for_current_index else {
    println!(
      "No larvae for current build order index {}, trying to assign",
      game_state.build_order_index
    );
    make_assignment_for_current_build_order_item(game, game_state);
    return;
  };

  let type_to_morph = match game_state.build_order[game_state.build_order_index] {
    BuildOrderItem::Unit(unit_type) => unit_type,
    _ => {
      println!("Expected unit to build in build_order_on_frame, found upgrade instead");
      return;
    }
  };

  if let Err(e) = larvae.morph(type_to_morph) {
    game.draw_text_screen((0, 10), format!("Failed to morph larvae: {:?}", e).as_str());
  }
}

// fn figure_out_what_to_build(game: &Game, game_state: &mut GameState) {
//   let Some(player) = game.self_() else {
//     println!("Failed to get self player in figure_out_what_to_build");
//     return;
//   };
//   let supply_total = player.supply_total() / 2;
//   let supply_used = player.supply_used() / 2;
//   let supply_remaining = supply_total - supply_used;

//   let overlords_in_production = game
//     .get_all_units()
//     .into_iter()
//     .filter(|u| {
//       u.get_type() == UnitType::Zerg_Egg
//         && u.get_player().get_id() == player.get_id()
//         && u.get_build_type() == UnitType::Zerg_Overlord
//     })
//     .count();

//   if supply_remaining < 4 && overlords_in_production == 0 {
//     println!(
//       "queuing overlord because supply is {} and overlords in production is {}",
//       supply_remaining, overlords_in_production
//     );
//     game_state
//       .build_order
//       .push(BuildOrderItem::Unit(UnitType::Zerg_Overlord));
//     return;
//   }

//   let total_drones = game
//     .get_all_units()
//     .into_iter()
//     .filter(|u| u.get_type() == UnitType::Zerg_Drone && u.get_player().get_id() == player.get_id())
//     .count();
//   if total_drones < 20 {
//     println!("queuing drone because total drones is {}", total_drones);
//     game_state
//       .build_order
//       .push(BuildOrderItem::Unit(UnitType::Zerg_Drone));
//   } else {
//     println!("queuing zergling");
//     game_state
//       .build_order
//       .push(BuildOrderItem::Unit(UnitType::Zerg_Zergling));
//   }
// }
