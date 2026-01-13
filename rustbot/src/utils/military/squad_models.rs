use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilitarySquad {
  pub name: String,
  pub assigned_unit_ids: HashSet<usize>,
  pub target_position: Option<(i32, i32)>,
  pub target_unit: Option<usize>,
  pub target_path: Option<Vec<(i32, i32)>>,
  pub target_path_goal_index: Option<usize>,
  pub target_path_current_index: Option<usize>,
}
