use rsbwapi::*;

use crate::utils::build_orders::build_order_item::BuildOrderItem;
use crate::utils::military::squad_models;

pub fn build_order() -> Vec<BuildOrderItem> {
  vec![
    // Opening
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::squad(
      "Rush Workers".to_string(),
      squad_models::SquadRole::AttackWorkers,
      squad_models::SquadStatus::Gathering
    ),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Spawning_Pool),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Overlord),
    BuildOrderItem::unit(UnitType::Zerg_Drone), // this one delays a zergling slightly
    // rush attack and fast expand
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::squad(
      "Defend Base".to_string(),
      squad_models::SquadRole::Defend,
      squad_models::SquadStatus::Gathering
    ),
    BuildOrderItem::unit_at_base(UnitType::Zerg_Hatchery, 1), // Natural expansion
    BuildOrderItem::unit(UnitType::Zerg_Hatchery),
    BuildOrderItem::unit_at_base(UnitType::Zerg_Extractor, 0),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    // Economy and upgrades
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Evolution_Chamber),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Overlord),
    BuildOrderItem::unit_at_base(UnitType::Zerg_Creep_Colony, 1),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Sunken_Colony),
    // Lair tech
    BuildOrderItem::unit(UnitType::Zerg_Lair),
    BuildOrderItem::Upgrade(UpgradeType::Metabolic_Boost),
    BuildOrderItem::unit(UnitType::Zerg_Hatchery),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Overlord),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Hatchery),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
  ]
}
