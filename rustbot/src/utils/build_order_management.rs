use crate::utils::building_stuff::{creature_stuff, structure_stuff};
use crate::utils::game_state::{BuildOrderItem, GameState, SharedGameState};
use rsbwapi::*;

fn make_assignment_for_current_build_order_item(game: &Game, game_state: &SharedGameState) {
  let Ok(mut game_state) = game_state.lock() else {
    println!("Failed to lock game_state in make_assignment_for_current_build_order_item");
    return;
  };

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
      structure_stuff::build_building_onframe(game, &mut game_state, &player, unit_to_build);
    } else {
      creature_stuff::assign_larva_to_build_unit(game, &mut game_state, &player, unit_to_build);
    }
  }
}

pub fn build_order_on_unit_create(
  game: &Game,
  completed_unit: &Unit,
  game_state: &SharedGameState,
) {
  let Ok(mut gs) = game_state.lock() else {
    println!("Failed to lock game_state in build_order_on_unit_create");
    return;
  };

  let Some(player) = game.self_() else {
    println!("Failed to get self player in build_order_on_unit_create");
    return;
  };

  if completed_unit.get_player().get_id() != player.get_id() {
    return;
  }

  let Some(current_build_order_item) = gs.build_order.get(gs.build_order_index).cloned() else {
    println!("Build order empty in build_order_on_unit_create");
    return;
  };

  match current_build_order_item {
    BuildOrderItem::Unit(unit_type) => {
      if completed_unit.get_type() == unit_type {
        println!(
          "Completed build order item: {:?} (unit created)",
          current_build_order_item
        );
        gs.build_order_index += 1;
        drop(gs);
        make_assignment_for_current_build_order_item(game, game_state);
      }
    }
    BuildOrderItem::Upgrade(upgrade_type) => {}
  }
}

fn figure_out_what_to_build(game: &Game, game_state: &mut GameState) {
  let Some(player) = game.self_() else {
    println!("Failed to get self player in figure_out_what_to_build");
    return;
  };
  let supply_total = player.supply_total() / 2;
  let supply_used = player.supply_used() / 2;
  let supply_remaining = supply_total - supply_used;

  let overlords_in_production = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_type() == UnitType::Zerg_Egg
        && u.get_player().get_id() == player.get_id()
        && u.get_build_type() == UnitType::Zerg_Overlord
    })
    .count();

  if supply_remaining < 4 && overlords_in_production == 0 {
    println!(
      "queuing overlord because supply is {} and overlords in production is {}",
      supply_remaining, overlords_in_production
    );
    game_state
      .build_order
      .push(BuildOrderItem::Unit(UnitType::Zerg_Overlord));
    return;
  }

  let total_drones = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Zerg_Drone && u.get_player().get_id() == player.get_id())
    .count();
  if total_drones < 20 {
    println!("queuing drone because total drones is {}", total_drones);
    game_state
      .build_order
      .push(BuildOrderItem::Unit(UnitType::Zerg_Drone));
  } else {
    println!("queuing zergling");
    game_state
      .build_order
      .push(BuildOrderItem::Unit(UnitType::Zerg_Zergling));
  }
}
