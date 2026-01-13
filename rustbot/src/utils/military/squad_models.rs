use std::collections::HashSet;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SquadRole {
  Attack,
  Defend,
  AttackWorkers,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SquadStatus {
  Gathering,
  Attacking,
}