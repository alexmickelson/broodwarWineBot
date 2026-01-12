use rsbwapi::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerAssignmentType {
  Gathering,
  Building,
  Scouting,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkerAssignment {
  pub assignment_type: WorkerAssignmentType,
  pub target_unit: Option<usize>,
  pub target_position: Option<(i32, i32)>,
  pub build_order_index: Option<usize>,
}

impl WorkerAssignment {
  pub fn gathering(target_unit: usize) -> Self {
    Self {
      assignment_type: WorkerAssignmentType::Gathering,
      target_unit: Some(target_unit),
      target_position: None,
      build_order_index: None,
    }
  }

  pub fn building(
    target_unit: Option<usize>,
    target_position: (i32, i32),
    build_order_index: usize,
  ) -> Self {
    Self {
      assignment_type: WorkerAssignmentType::Building,
      target_unit,
      target_position: Some(target_position),
      build_order_index: Some(build_order_index),
    }
  }

  pub fn scouting(target_position: (i32, i32)) -> Self {
    Self {
      assignment_type: WorkerAssignmentType::Scouting,
      target_unit: None,
      target_position: Some(target_position),
      build_order_index: None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitOrder {
  pub unit_id: usize,
  pub unit_type: String,
  pub order_name: String,
  pub target_id: Option<usize>,
  pub target_type: Option<String>,
  pub current_position: (i32, i32),
  pub target_position: Option<(i32, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildOrderItem {
  Unit(UnitType),
  Upgrade(UpgradeType),
}

impl Serialize for BuildOrderItem {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      BuildOrderItem::Unit(unit_type) => serializer.serialize_str(&format!("{:?}", unit_type)),
      BuildOrderItem::Upgrade(upgrade_type) => serializer.serialize_str(&format!("{:?}", upgrade_type)),
    }
  }
}

impl<'de> Deserialize<'de> for BuildOrderItem {
  fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    // For now, just return a default value as we don't need to deserialize
    Ok(BuildOrderItem::Unit(UnitType::None))
  }
}

pub struct GameState {
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub game_speed: i32,
  pub build_order: Vec<BuildOrderItem>,
  pub build_order_index: usize,
  // Maps larva unit ID to the build order index it was assigned to morph into
  pub larva_responsibilities: HashMap<usize, usize>,
}

impl Default for GameState {
  fn default() -> Self {
    Self {
      worker_assignments: HashMap::new(),
      game_speed: -1,
      build_order: vec![
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Overlord),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Spawning_Pool),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Hatchery),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Extractor),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Creep_Colony),
        BuildOrderItem::Unit(UnitType::Zerg_Overlord),
        BuildOrderItem::Unit(UnitType::Zerg_Evolution_Chamber),
        BuildOrderItem::Upgrade(UpgradeType::Metabolic_Boost), // zergling speed
      ],
      build_order_index: 0,
      larva_responsibilities: HashMap::new(),
    }
  }
}

pub type SharedGameState = Arc<Mutex<GameState>>;
