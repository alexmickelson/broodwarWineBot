use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilitarySquad {
  pub name: String,
  pub role: SquadRole,
  pub status: SquadStatus,
  pub assigned_unit_ids: HashSet<usize>,
  pub target_position: Option<(i32, i32)>,
  pub target_path: Option<Vec<(i32, i32)>>,
  pub target_path_index: Option<usize>,
  pub leader_unit_id: Option<usize>,
  pub required_units_near_leader: usize,
  #[serde(skip)]
  pub unit_path_assignments: HashMap<usize, (Vec<(i32, i32)>, usize)>, // (path, current_index)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SquadRole {
  AttackAsMutas,
  Defend,
  AttackWorkers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SquadStatus {
  Gathering,
  Attacking,
}