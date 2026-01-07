use crate::map::MapData;
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
}

impl WorkerAssignment {
  pub fn gathering(target_unit: usize) -> Self {
    Self {
      assignment_type: WorkerAssignmentType::Gathering,
      target_unit: Some(target_unit),
      target_position: None,
    }
  }

  #[allow(dead_code)]
  pub fn building(target_unit: Option<usize>, target_position: (i32, i32)) -> Self {
    Self {
      assignment_type: WorkerAssignmentType::Building,
      target_unit,
      target_position: Some(target_position),
    }
  }

  pub fn scouting(target_position: (i32, i32)) -> Self {
    Self {
      assignment_type: WorkerAssignmentType::Scouting,
      target_unit: None,
      target_position: Some(target_position),
    }
  }
}

#[derive(Clone, Debug)]
pub struct GameStatus {
  pub map_data: MapData,
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub game_speed: i32,
}

impl Default for GameStatus {
  fn default() -> Self {
    Self {
      map_data: MapData::default(),
      worker_assignments: HashMap::new(),
      game_speed: -1,
    }
  }
}

pub type SharedStatus = Arc<Mutex<GameStatus>>;
