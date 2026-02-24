use crate::utils::game_state::{BuildingAssignment, GameState};
use rsbwapi::*;

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

  let building_id = building.get_id() as usize;
  game_state.building_assignments.insert(
    building_id,
    BuildingAssignment::new_upgrade(upgrade, game_state.build_order_index),
  );
  println!(
    "Assigned building {} ({:?}) to research {:?} for build order index {}",
    building_id, building_type, upgrade, game_state.build_order_index
  );
}

pub fn enforce_upgrade_assignment(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
  upgrade_to_build: UpgradeType,
) {
  let current_level = player.get_upgrade_level(upgrade_to_build);
  let needed_minerals = upgrade_to_build.mineral_price(current_level);
  let needed_gas = upgrade_to_build.gas_price(current_level);

  game.draw_text_screen(
    (0, 60),
    &format!(
      "next {:?}, {}/{} minerals and {}/{} gas",
      upgrade_to_build,
      player.minerals(),
      needed_minerals,
      player.gas(),
      needed_gas
    ),
  );

  if player.minerals() < needed_minerals {
    return;
  }
  if player.gas() < needed_gas {
    return;
  }

  let Some(assigned_building) =
    game_state
      .building_assignments
      .iter()
      .find_map(|(&building_id, assignment)| {
        if assignment.build_order_index == game_state.build_order_index {
          let maybe_building = game.get_unit(building_id);
          let Some(building) = maybe_building else {
            return None;
          };
          Some((building, assignment))
        } else {
          None
        }
      })
  else {
    println!(
      "No building assigned to research {:?} for current build order index {}",
      upgrade_to_build, game_state.build_order_index
    );
    return;
  };

  let (building, _assignment) = assigned_building;
  if building.is_upgrading() {
    let current_upgrade = building.get_upgrade();
    if current_upgrade == upgrade_to_build {
      println!(
        "{:?} is already researching the correct upgrade {:?}, moving to next build order item when done",
        building.get_type(),
        upgrade_to_build
      );
      remove_building_upgrade_assignment(game_state, &building);
      game_state.build_order_index += 1;
    } else {
      game.draw_text_screen(
        (0, 60),
        &format!(
          "{:?} is already researching an upgrade, waiting for it to finish",
          building.get_type()
        ),
      );
    }
    return;
  }

  match building.upgrade(upgrade_to_build) {
    Ok(true) => {
      println!(
        "Started researching {:?} at building {} ({:?})",
        upgrade_to_build,
        building.get_id(),
        building.get_type()
      );
    }
    Ok(false) => {
      println!(
        "Failed to start researching {:?} at building {} ({:?}) for unknown reason",
        upgrade_to_build,
        building.get_id(),
        building.get_type()
      );
    }
    Err(e) => {
      println!(
        "Failed to start researching {:?} at building {} ({:?}): {:?}",
        upgrade_to_build,
        building.get_id(),
        building.get_type(),
        e
      );
    }
  }
}

pub fn remove_building_upgrade_assignment(game_state: &mut GameState, unit: &Unit) {
  let unit_id = unit.get_id() as usize;
  if let Some(assignment) = game_state.building_assignments.get(&unit_id) {
    if let crate::utils::game_state::BuildingAssignmentType::ResearchUpgrade(upgrade) =
      assignment.assignment_type
    {
      println!(
        "Finished researching {:?} at building {} ({:?}), removing assignment",
        upgrade,
        unit_id,
        unit.get_type()
      );
    }
  }

  if game_state.building_assignments.remove(&unit_id).is_none() {
    println!(
      "No building assignment found for building {} ({:?}) when trying to remove upgrade assignment",
      unit_id,
      unit.get_type()
    );
  }
}
