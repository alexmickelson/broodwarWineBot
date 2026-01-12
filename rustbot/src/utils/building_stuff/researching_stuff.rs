use crate::utils::game_state::GameState;
use rsbwapi::*;

pub fn research_upgrade_onframe(
  game: &Game,
  game_state: &mut GameState,
  player: &Player,
  upgrade_to_build: UpgradeType,
) {
  let current_level = player.get_upgrade_level(upgrade_to_build);
  let needed_minerals = upgrade_to_build.mineral_price(current_level);

  game.draw_text_screen(
    (0, 60),
    &format!(
      "next {:?}, {}/{} minerals",
      upgrade_to_build,
      player.minerals(),
      needed_minerals
    ),
  );

  if player.minerals() < needed_minerals {
    return;
  }

  // Find the building that can research this upgrade
  // Map upgrades to their research buildings for Zerg
  let research_building_type = match upgrade_to_build {
    // Unit-specific upgrades
    UpgradeType::Metabolic_Boost => UnitType::Zerg_Spawning_Pool, // Zergling speed
    UpgradeType::Adrenal_Glands => UnitType::Zerg_Spawning_Pool,  // Zergling attack speed
    UpgradeType::Pneumatized_Carapace => UnitType::Zerg_Lair,     // Overlord speed
    UpgradeType::Ventral_Sacs => UnitType::Zerg_Lair,             // Overlord transport
    UpgradeType::Antennae => UnitType::Zerg_Lair,                 // Overlord sight range
    UpgradeType::Chitinous_Plating => UnitType::Zerg_Ultralisk_Cavern, // Ultralisk armor
    UpgradeType::Anabolic_Synthesis => UnitType::Zerg_Ultralisk_Cavern, // Ultralisk speed
    UpgradeType::Muscular_Augments => UnitType::Zerg_Hydralisk_Den, // Hydralisk speed
    UpgradeType::Grooved_Spines => UnitType::Zerg_Hydralisk_Den,  // Hydralisk range
    UpgradeType::Gamete_Meiosis => UnitType::Zerg_Queens_Nest,    // Queen energy
    UpgradeType::Metasynaptic_Node => UnitType::Zerg_Defiler_Mound, // Defiler energy

    // Attack and armor upgrades (Evolution Chamber)
    UpgradeType::Zerg_Melee_Attacks => UnitType::Zerg_Evolution_Chamber, // Ground melee attack
    UpgradeType::Zerg_Missile_Attacks => UnitType::Zerg_Evolution_Chamber, // Ground ranged attack
    UpgradeType::Zerg_Carapace => UnitType::Zerg_Evolution_Chamber,      // Ground armor

    // Air attack and armor upgrades (Spire/Greater Spire)
    UpgradeType::Zerg_Flyer_Attacks => UnitType::Zerg_Spire, // Air attack
    UpgradeType::Zerg_Flyer_Carapace => UnitType::Zerg_Spire, // Air armor

    _ => {
      game.draw_text_screen(
        (0, 80),
        &format!("Unknown research building for {:?}", upgrade_to_build),
      );
      return;
    }
  };

  let research_building = game.get_all_units().into_iter().find(|u| {
    u.get_player().get_id() == player.get_id()
      && u.get_type() == research_building_type
      && u.is_completed()
      && !u.is_upgrading()
  });

  if let Some(building) = research_building {
    if building.upgrade(upgrade_to_build).is_ok() {
      println!("Started researching upgrade {:?}", upgrade_to_build);
      game_state.build_order_index += 1;
    }
  } else {
    game.draw_text_screen(
      (0, 80),
      &format!(
        "No {:?} available to research upgrade",
        research_building_type
      ),
    );
  }
}
