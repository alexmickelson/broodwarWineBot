use rsbwapi::*;
use crate::utils::game_state::*;

pub fn advance_build_order_for_morphed_larvae(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
) {
  let current_larva_ids: std::collections::HashSet<usize> = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Zerg_Larva && u.get_player().get_id() == player.get_id())
    .map(|u| u.get_id() as usize)
    .collect();

  let mut morphed_indices = Vec::new();
  game_state
    .larva_responsibilities
    .retain(|larva_id, build_idx| {
      if !current_larva_ids.contains(larva_id) {
        morphed_indices.push(*build_idx);
        false
      } else {
        true
      }
    });

  for morphed_idx in morphed_indices {
    if morphed_idx == game_state.build_order_index {
      game_state.build_order_index += 1;
      println!(
        "Larva morphed, advancing build order index to {}",
        game_state.build_order_index
      );
    }
  }
}

pub fn build_unit_from_larva_onframe(
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

  game.draw_text_screen(
    (0, 0),
    &format!(
      "next {:?}, {}/{} minerals, {} larva",
      unit_type,
      player.minerals(),
      needed_minerals,
      larva_units.len()
    ),
  );

  if player.minerals() < needed_minerals {
    return;
  }

  build_unit_from_larva(game, game_state, &larva_units, unit_type);
}

fn build_unit_from_larva(
  game: &Game,
  game_state: &mut GameState,
  larva_units: &[Unit],
  unit_type: UnitType,
) {
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
