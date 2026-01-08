use crate::utils::game_state::SharedGameState;
use rsbwapi::*;

pub fn build_order_onframe(game: &Game, game_state: &SharedGameState) {
  let Ok(mut game_state) = game_state.lock() else {
    println!("Failed to lock game_state in build_order_onframe");
    return;
  };

  let Some(unit_type) = game_state.build_order.first().cloned() else {
    println!("build order empty");
    return;
  };

  let Some(player) = game.self_() else {
    println!("Failed to get self player in build_order_onframe");
    return;
  };

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
      println!("Need to train: {:?}", unit_type);
      if larva_units.is_empty() {
        game.draw_text_screen((10, 10), "No larva available to train unit");
        return;
      }

      let larva = larva_units.first().expect("could not get first larva");

      if larva.train(unit_type).is_ok() {
        game_state.build_order.remove(0);
      }
    }
  }
}
