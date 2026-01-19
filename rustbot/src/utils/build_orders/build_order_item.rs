use rsbwapi::*;
use serde::{Deserialize, Serialize};

use crate::utils::military::squad_models;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildOrderItem {
  Unit {
    unit_type: UnitType,
    base_index: Option<usize>,
  },
  Upgrade(UpgradeType),
  Squad {
    name: String,
    role: squad_models::SquadRole,
    status: squad_models::SquadStatus,
  },
}

impl BuildOrderItem {
  /// Create a Unit build order item without a specific base location
  pub fn unit(unit_type: UnitType) -> Self {
    BuildOrderItem::Unit {
      unit_type,
      base_index: None,
    }
  }

  /// Create a Unit build order item at a specific base location
  /// base_index 0 = starting location, 1 = natural expansion, etc.
  pub fn unit_at_base(unit_type: UnitType, base_index: usize) -> Self {
    BuildOrderItem::Unit {
      unit_type,
      base_index: Some(base_index),
    }
  }

  /// Create a Squad build order item
  pub fn squad(name: String, role: squad_models::SquadRole, status: squad_models::SquadStatus) -> Self {
    BuildOrderItem::Squad { name, role, status }
  }
}

impl Serialize for BuildOrderItem {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      BuildOrderItem::Unit {
        unit_type,
        base_index,
      } => {
        let base_str = match base_index {
          Some(idx) => format!(" @base{}", idx),
          None => String::new(),
        };
        serializer.serialize_str(&format!("{:?}{}", unit_type, base_str))
      }
      BuildOrderItem::Upgrade(upgrade_type) => {
        serializer.serialize_str(&format!("{:?}", upgrade_type))
      }
      BuildOrderItem::Squad { name, role, status } => {
        serializer.serialize_str(&format!("Squad({}, {:?}, {:?})", name, role, status))
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
    Ok(BuildOrderItem::Unit {
      unit_type: UnitType::None,
      base_index: None,
    })
  }
}
