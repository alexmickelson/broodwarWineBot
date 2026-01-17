use rsbwapi::*;
use serde::{Deserialize, Serialize};

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
