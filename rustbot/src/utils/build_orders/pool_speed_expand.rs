use rsbwapi::*;

use crate::utils::build_orders::build_order_item::BuildOrderItem;

/// 12 Pool Speed Expand into Hydralisk Transition
/// Standard Zerg opening with early economy expansion, transitioning to hydra tech
pub fn build_order() -> Vec<BuildOrderItem> {
  use BuildOrderItem::*;

  vec![
    // Opening - 12 Pool
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Overlord),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Spawning_Pool),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    
    // Fast expand
    Unit(UnitType::Zerg_Hatchery),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Hatchery),
    Unit(UnitType::Zerg_Extractor),
    Unit(UnitType::Zerg_Zergling),
    
    // Economy and upgrades
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Evolution_Chamber),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Upgrade(UpgradeType::Metabolic_Boost),
    Unit(UnitType::Zerg_Overlord),
    Unit(UnitType::Zerg_Creep_Colony),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Zergling),
    
    // Lair tech
    Unit(UnitType::Zerg_Lair),
    Unit(UnitType::Zerg_Hatchery),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Overlord),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Zergling),
    Unit(UnitType::Zerg_Hatchery),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Upgrade(UpgradeType::Zerg_Melee_Attacks),
    Unit(UnitType::Zerg_Lair),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    
    // Hydralisk transition
    Unit(UnitType::Zerg_Extractor),         // Second gas
    Unit(UnitType::Zerg_Hydralisk_Den),     // Requires Lair
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Overlord),
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Hydralisk),
    Upgrade(UpgradeType::Grooved_Spines),   // +1 hydra range
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Overlord),
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Drone),
    Unit(UnitType::Zerg_Drone),
    Upgrade(UpgradeType::Zerg_Missile_Attacks),  // Hydra attack upgrade
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Hydralisk),
    Unit(UnitType::Zerg_Overlord),
  ]
}
