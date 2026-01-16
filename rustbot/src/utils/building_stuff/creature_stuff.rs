use crate::utils::game_state::*;
use rsbwapi::*;

pub fn assign_larva_to_build_unit(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
  unit_type: UnitType,
) {
  let needed_minerals = unit_type.mineral_price();

  let larva_units: Vec<Unit> = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Zerg_Larva && u.get_player().get_id() == player.get_id())
    .collect();

  println!(
    "assigning larva to build unit {:?}, {}/{} minerals, {} larva",
    unit_type,
    player.minerals(),
    needed_minerals,
    larva_units.len()
  );

  if player.minerals() < needed_minerals {
    return;
  }

  // Find a larva that doesn't have a responsibility yet
  let available_larva = larva_units.iter().find(|larva| {
    !game_state
      .larva_responsibilities
      .contains_key(&(larva.get_id() as usize))
  });

  let Some(larva) = available_larva else {
    game.draw_text_screen((10, 10), "No available larva to train unit");
    return;
  };

  let larva_id = larva.get_id() as usize;
  let current_build_idx = game_state.build_order_index;

  if larva.train(unit_type).is_ok() {
    // Assign this larva the responsibility of morphing this build order item
    game_state
      .larva_responsibilities
      .insert(larva_id, current_build_idx);
    println!(
      "Assigned larva {} to build order index {}",
      larva_id, current_build_idx
    );
  }
}

pub fn remove_larva_responsibility(game_state: &mut GameState, unit: &Unit) {
  let unit_id = unit.get_id() as usize;
  if game_state.larva_responsibilities.remove(&unit_id).is_some() {
    println!(
      "Removed larva responsibility for unit {} (finished morphing into {:?})",
      unit_id,
      unit.get_type()
    );
  } else {
    println!(
      "No larva responsibility found for unit {} (finished morphing into {:?})",
      unit_id,
      unit.get_type()
    );
  }
}
