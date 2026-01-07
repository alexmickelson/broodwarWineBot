use crate::map::MapData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerStatus {
    pub total: usize,
    pub gathering: usize,
    pub idle: usize,
    pub building: usize,
}

impl Default for WorkerStatus {
    fn default() -> Self {
        Self {
            total: 0,
            gathering: 0,
            idle: 0,
            building: 0,
        }
    }
}

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

#[derive(Clone, Debug, Default)]
pub struct GameStatus {
    pub worker_status: WorkerStatus,
    pub map_data: MapData,
    pub worker_assignments: HashMap<usize, WorkerAssignment>,
}

pub type SharedStatus = Arc<Mutex<GameStatus>>;
