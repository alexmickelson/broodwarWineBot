use rsbwapi::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::utils::military::squad_models::MilitarySquad;

pub struct GameState {
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub building_assignments: HashMap<usize, BuildingAssignment>,
  pub game_speed: i32,
  pub build_order: Vec<BuildOrderItem>,
  pub build_order_index: usize,
  pub larva_responsibilities: HashMap<usize, usize>,
  pub military_squads: Vec<MilitarySquad>,
  pub path_to_enemy_base: Option<Vec<(i32, i32)>>,
  pub debug_flags: HashSet<DebugFlag>,
}

impl Default for GameState {
  fn default() -> Self {
    Self {
      worker_assignments: HashMap::new(),
      building_assignments: HashMap::new(),
      game_speed: -1,
      build_order: vec![
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
        BuildOrderItem::Unit(UnitType::Zerg_Hatchery),
        BuildOrderItem::Unit(UnitType::Zerg_Extractor),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Upgrade(UpgradeType::Metabolic_Boost), // zergling speed
        BuildOrderItem::Unit(UnitType::Zerg_Evolution_Chamber),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Drone),
        BuildOrderItem::Unit(UnitType::Zerg_Overlord),
        BuildOrderItem::Unit(UnitType::Zerg_Creep_Colony),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Lair),
        BuildOrderItem::Unit(UnitType::Zerg_Hatchery),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Zergling),
        BuildOrderItem::Unit(UnitType::Zerg_Hatchery),
        BuildOrderItem::Upgrade(UpgradeType::Zerg_Melee_Attacks),
        BuildOrderItem::Unit(UnitType::Zerg_Lair),
      ],
      build_order_index: 0,
      larva_responsibilities: HashMap::new(),
      military_squads: vec![],
      path_to_enemy_base: None,
      debug_flags: [
        DebugFlag::ShowWorkerAssignments,
        DebugFlag::ShowMilitaryAssignments,
        DebugFlag::ShowPathToEnemyBase,
      ]
      .into_iter()
      .collect(),
    }
  }
}

pub type SharedGameState = Arc<Mutex<GameState>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerAssignmentType {
  Gathering,
  Building,
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildingAssignmentType {
  TrainUnit(UnitType),
  ResearchUpgrade(UpgradeType),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingAssignment {
  pub assignment_type: BuildingAssignmentType,
  pub build_order_index: usize,
}

impl BuildingAssignment {
  pub fn new(unit_to_train: UnitType, build_order_index: usize) -> Self {
    Self {
      assignment_type: BuildingAssignmentType::TrainUnit(unit_to_train),
      build_order_index,
    }
  }

  pub fn new_upgrade(upgrade_type: UpgradeType, build_order_index: usize) -> Self {
    Self {
      assignment_type: BuildingAssignmentType::ResearchUpgrade(upgrade_type),
      build_order_index,
    }
  }
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
      BuildOrderItem::Upgrade(upgrade_type) => {
        serializer.serialize_str(&format!("{:?}", upgrade_type))
      }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DebugFlag {
  ShowWorkerAssignments,
  ShowMilitaryAssignments,
  ShowPathToEnemyBase,
  ShowRegions,
}
