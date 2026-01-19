use rsbwapi::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::utils::build_orders::build_order_item::BuildOrderItem;
use crate::utils::military::squad_models::MilitarySquad;

pub struct GameState {
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub building_assignments: HashMap<usize, BuildingAssignment>,
  pub base_locations: Vec<(i32, i32)>,
  pub debug_lines: Vec<(Position, Position, Color)>,
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
      base_locations: vec![],
      debug_lines: vec![],
      game_speed: -1,
      build_order: vec![],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DebugFlag {
  ShowWorkerAssignments,
  ShowMilitaryAssignments,
  ShowPathToEnemyBase,
  ShowRegions,
}
