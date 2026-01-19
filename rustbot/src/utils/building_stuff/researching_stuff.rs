use crate::utils::build_order_management;
use crate::utils::build_orders::build_order_item::BuildOrderItem;
use crate::utils::game_state::{BuildingAssignment, GameState};
use rsbwapi::*;

pub fn check_and_advance_upgrade_if_started(game: &Game, game_state: &mut GameState) {
  if has_started_current_upgrade(game, game_state) {
    let upgrade_name = if let Some(BuildOrderItem::Upgrade(upgrade_type)) =
      game_state.build_order.get(game_state.build_order_index)
    {
      format!("{:?}", upgrade_type)
    } else {
      "Unknown".to_string()
    };
    build_order_management::advance_build_order(
      game,
      game_state,
      &format!("Upgrade {:} started", upgrade_name),
    );
  }
}

fn has_started_current_upgrade(game: &Game, game_state: &GameState) -> bool {
  // Check if the current build order item is an upgrade
  let Some(current_item) = game_state.build_order.get(game_state.build_order_index) else {
    return false;
  };

  let upgrade_type = match current_item {
    BuildOrderItem::Upgrade(upgrade) => upgrade,
    _ => return false,
  };
  // Find the building assigned to this build order index
  let Some(building_id) =
    game_state
      .building_assignments
      .iter()
      .find_map(|(&building_id, assignment)| {
        if assignment.build_order_index == game_state.build_order_index {
          Some(building_id)
        } else {
          None
        }
      })
  else {
    return false;
  };

  // Check if the building is actually researching the upgrade
  let Some(building_unit) = game.get_unit(building_id) else {
    return false;
  };

  // Check if the building is upgrading and if it's the right upgrade
  building_unit.is_upgrading() && building_unit.get_upgrade() == *upgrade_type
}

pub fn assign_building_to_research_upgrade(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
  upgrade: UpgradeType,
) {
  let building_type = upgrade.what_upgrades();

  // Find a building of this type that can research the upgrade
  let Some(building) = game.get_all_units().into_iter().find(|u| {
    u.get_player().get_id() == player.get_id()
      && u.get_type() == building_type
      && u.is_completed()
      && !u.is_upgrading()
  }) else {
    println!(
      "No available {:?} found to research upgrade {:?}",
      building_type, upgrade
    );
    return;
  };

  let building_id = building.get_id();
  game_state.building_assignments.insert(
    building_id,
    BuildingAssignment::new_upgrade(upgrade, game_state.build_order_index),
  );
  println!(
    "Assigned building {} ({:?}) to research {:?} for build order index {}",
    building_id, building_type, upgrade, game_state.build_order_index
  );
}

pub fn enforce_research_assignment(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
  upgrade_to_build: UpgradeType,
) {
  let current_level = player.get_upgrade_level(upgrade_to_build);
  let needed_minerals = upgrade_to_build.mineral_price(current_level);
  let needed_gas = upgrade_to_build.gas_price(current_level);
  let current_gas = player.gas();

  game.draw_text_screen(
    (0, 60),
    &format!(
      "next {:?}, {}/{} minerals, {}/{} gas",
      upgrade_to_build,
      player.minerals(),
      needed_minerals,
      current_gas,
      needed_gas
    ),
  );

  if player.minerals() < needed_minerals || current_gas < needed_gas {
    return;
  }

  let Some(building_id) =
    game_state
      .building_assignments
      .iter()
      .find_map(|(&building_id, assignment)| {
        if assignment.build_order_index == game_state.build_order_index {
          Some(building_id)
        } else {
          None
        }
      })
  else {
    game.draw_text_screen((0, 80), "No building assigned for this upgrade");
    return;
  };

  let Some(building_unit) = game.get_unit(building_id) else {
    game.draw_text_screen((0, 80), "Assigned building unit not found");
    return;
  };

  if building_unit.upgrade(upgrade_to_build).is_ok() {
    // println!("Started researching upgrade {:?}", upgrade_to_build);
    // game_state.build_order_index += 1;
  } else {
    game.draw_text_screen(
      (0, 80),
      &format!("Failed to start researching upgrade {:?}", upgrade_to_build),
    );
  }
}
