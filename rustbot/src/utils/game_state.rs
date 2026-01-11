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
pub struct GameState {
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub game_speed: i32,
  pub build_order: Vec<UnitType>,
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
        UnitType::Zerg_Drone,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Overlord,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Overlord,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Spawning_Pool,
        UnitType::Zerg_Drone,
        UnitType::Zerg_Hatchery,
        UnitType::Zerg_Zergling,
        UnitType::Zerg_Zergling,
        UnitType::Zerg_Zergling,
        UnitType::Zerg_Zergling,
      ],
      build_order_index: 0,
      larva_responsibilities: HashMap::new(),
    }
  }
}

pub type SharedGameState = Arc<Mutex<GameState>>;
