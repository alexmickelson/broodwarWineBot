use crate::utils::game_state::{GameState, SharedGameState};
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

  advance_build_order_for_morphed_larvae(game, &mut game_state, &player);

  if game_state.build_order_index >= game_state.build_order.len() {
    println!("build order empty");
    figure_out_what_to_build(game, &mut game_state);
    return;
  }

  let unit_type = game_state.build_order[game_state.build_order_index];

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

  match unit_type {
    ut if ut.is_building() => {
      println!("Need to build: {:?}", ut);
    }
    _ => {
      // Find a larva that doesn't have a responsibility yet
      let available_larva = larva_units.iter().find(|larva| {
        !game_state.larva_responsibilities.contains_key(&(larva.get_id() as usize))
      });

      let Some(larva) = available_larva else {
        game.draw_text_screen((10, 10), "No available larva to train unit");
        return;
      };

      let larva_id = larva.get_id() as usize;
      let current_build_idx = game_state.build_order_index;

      if larva.train(unit_type).is_ok() {
        // Assign this larva the responsibility of morphing this build order item
        game_state.larva_responsibilities.insert(larva_id, current_build_idx);
        println!("Assigned larva {} to build order index {}", larva_id, current_build_idx);
      }
    }
  }
}

fn figure_out_what_to_build(game: &Game, game_state: &mut GameState) {
  let Some(player) = game.self_() else {
    println!("Failed to get self player in figure_out_what_to_build");
    return;
  };
  // BWAPI stores supply as double the actual value
  let supply_total = player.supply_total() / 2;
  let supply_used = player.supply_used() / 2;
  let supply_remaining = supply_total - supply_used;

  // Check for eggs morphing into overlords, not overlords themselves
  let overlords_in_production = game
    .get_all_units()
    .into_iter()
    .filter(|u| {
      u.get_type() == UnitType::Zerg_Egg
        && u.get_player().get_id() == player.get_id()
        && u.get_build_type() == UnitType::Zerg_Overlord
    })
    .count();
  // println!("supply remaining: {}, player total supply: {}, player used supply: {}, overlords in production: {}", supply_remaining, supply_total, supply_used, overlords_in_production);

  if supply_remaining < 4 && overlords_in_production == 0 {
    println!("queuing overlord because supply is {} and overlords in production is {}", supply_remaining, overlords_in_production);
    game_state.build_order.push(UnitType::Zerg_Overlord);
    return;
  }

  let total_drones = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Zerg_Drone && u.get_player().get_id() == player.get_id())
    .count();
  if total_drones < 20 {
    println!("queuing drone because total drones is {}", total_drones);
    game_state.build_order.push(UnitType::Zerg_Drone);
    return;
  }
}



fn advance_build_order_for_morphed_larvae(game: &Game, game_state: &mut GameState, player: &Player) {
  // Clean up responsibilities for larvae that have morphed (no longer exist as larvae)
  let current_larva_ids: std::collections::HashSet<usize> = game
    .get_all_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Zerg_Larva && u.get_player().get_id() == player.get_id())
    .map(|u| u.get_id() as usize)
    .collect();

  let mut morphed_indices = Vec::new();
  game_state.larva_responsibilities.retain(|larva_id, build_idx| {
    if !current_larva_ids.contains(larva_id) {
      // Larva has morphed, track which index to advance
      morphed_indices.push(*build_idx);
      false // Remove this responsibility
    } else {
      true // Keep this responsibility
    }
  });

  // Advance the build order index if the current item was morphed
  for morphed_idx in morphed_indices {
    if morphed_idx == game_state.build_order_index {
      game_state.build_order_index += 1;
      println!("Larva morphed, advancing build order index to {}", game_state.build_order_index);
    }
  }
}
