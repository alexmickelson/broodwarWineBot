pub mod build_order_management;
pub mod game_state;
pub mod http_status_callbacks;
pub mod worker_management;

pub mod building_stuff {
  pub mod build_location_utils;
  pub mod creature_stuff;
  pub mod expansion_location_stuff;
  pub mod researching_stuff;
  pub mod structure_stuff;
}

pub mod military {
  pub mod military_management;
  pub mod squad_attack_workers;
  pub mod squad_defend;
  pub mod squad_models;
  pub mod squad_mutas;
}

pub mod map_utils {
  pub mod pathing;
  pub mod region_stuff;
}

pub mod build_orders {
  pub mod build_order_item;
  pub mod pool_speed_expand;
}
