use crate::utils::building_stuff::researching_stuff::research_upgrade_onframe;
use crate::utils::building_stuff::{creature_stuff, structure_stuff};
use crate::utils::game_state::{BuildOrderItem, GameState, SharedGameState};
use rsbwapi::*;

pub fn build_order_onframe(game: &Game, game_state: &SharedGameState) {
  let Ok(mut game_state) = game_state.lock() else {
    println!("Failed to lock game_state in build_order_onframe");
    return;
  };

  let Some(player) = game.self_() else {
    println!("Failed to get self player in build_order_onframe");
    return;
  };

  creature_stuff::advance_build_order_for_morphed_larvae(game, &mut game_state, &player);
  structure_stuff::advance_build_order_if_building_building(game, &mut game_state, &player);

  if game_state.build_order_index >= game_state.build_order.len() {
    println!("build order empty");
    figure_out_what_to_build(game, &mut game_state);
    return;
  }
  let thing_to_build = game_state.build_order[game_state.build_order_index].clone();

  if let BuildOrderItem::Unit(unit_to_build) = thing_to_build {
    if unit_to_build.is_building() {
      structure_stuff::build_building_onframe(game, &mut game_state, &player, unit_to_build);
    } else {
      creature_stuff::build_unit_from_larva_onframe(game, &mut game_state, &player, unit_to_build);
    }
  } else if let BuildOrderItem::Upgrade(upgrade_to_build) = thing_to_build {
    research_upgrade_onframe(game, &mut game_state, &player, upgrade_to_build);
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
