use rsbwapi::*;

use crate::utils::build_orders::build_order_item::BuildOrderItem;

pub fn build_order() -> Vec<BuildOrderItem> {
  vec![
    // Opening - 12 Pool
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Overlord),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Spawning_Pool),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    // Create units to attack
    BuildOrderItem::unit_at_base(UnitType::Zerg_Hatchery, 1), // Natural expansion
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Hatchery),
    BuildOrderItem::unit_at_base(UnitType::Zerg_Extractor, 0),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    // Economy and upgrades
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Evolution_Chamber),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::unit(UnitType::Zerg_Drone),
    BuildOrderItem::Upgrade(UpgradeType::Metabolic_Boost),
    BuildOrderItem::unit(UnitType::Zerg_Overlord),
    BuildOrderItem::unit(UnitType::Zerg_Creep_Colony),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    BuildOrderItem::unit(UnitType::Zerg_Zergling),
    // Lair tech
    BuildOrderItem::unit(UnitType::Zerg_Lair),
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
