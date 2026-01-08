use crate::map::MapData;
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

#[derive(Clone, Debug)]
pub struct GameState {
  pub map_data: MapData,
  pub worker_assignments: HashMap<usize, WorkerAssignment>,
  pub game_speed: i32,
  pub build_order: Vec<UnitType>,
  pub build_order_index: usize,
  // Maps larva unit ID to the build order index it was assigned to morph into
  pub larva_responsibilities: HashMap<usize, usize>,
  pub unit_orders: HashMap<usize, UnitOrder>,
}

impl Default for GameState {
  fn default() -> Self {
    Self {
      map_data: MapData::default(),
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
        UnitType::Zerg_Drone,
      ],
      build_order_index: 0,
      larva_responsibilities: HashMap::new(),
      unit_orders: HashMap::new(),
    }
  }
}

pub type SharedGameState = Arc<Mutex<GameState>>;

pub fn update_unit_orders(game: &rsbwapi::Game, game_state: &SharedGameState) {
  let Ok(mut game_state_lock) = game_state.lock() else {
    return;
  };

  let self_player = match game.self_() {
    Some(p) => p,
    None => return,
  };

  let my_units: Vec<_> = self_player.get_units().into_iter().collect();

  let mut unit_orders = HashMap::new();

  for unit in my_units {
    let unit_id = unit.get_id();
    let current_pos = unit.get_position();
    let order = unit.get_order();
    
    let target_id = unit.get_order_target().map(|t| t.get_id());
    let target_type = unit.get_order_target().map(|t| format!("{:?}", t.get_type()));
    let target_position = unit.get_target_position().map(|p| (p.x, p.y));

    unit_orders.insert(
      unit_id,
      UnitOrder {
        unit_id,
        unit_type: format!("{:?}", unit.get_type()),
        order_name: format!("{:?}", order),
        target_id,
        target_type,
        current_position: (current_pos.x, current_pos.y),
        target_position,
      },
    );
  }

  game_state_lock.unit_orders = unit_orders;
}
