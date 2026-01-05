use crate::status::{SharedStatus, WorkerStatus};
use rsbwapi::*;

/// A basic Broodwar bot using rsbwapi
pub struct RustBot {
    mineral_target: Option<UnitId>,
    status: SharedStatus,
}

impl RustBot {
    pub fn new(status: SharedStatus) -> Self {
        Self {
            mineral_target: None,
            status,
        }
    }

    fn update_worker_status(&self, game: &Game) {
        let self_player = match game.self_() {
            Some(p) => p,
            None => return,
        };

        let my_units = self_player.get_units();
        let workers: Vec<_> = my_units
            .iter()
            .filter(|u| u.get_type().is_worker())
            .collect();

        let total = workers.len();
        let gathering = workers
            .iter()
            .filter(|w| w.is_gathering_gas() || w.is_gathering_minerals())
            .count();
        let idle = workers.iter().filter(|w| w.is_idle()).count();
        let building = workers.iter().filter(|w| w.is_constructing()).count();

        if let Ok(mut status) = self.status.lock() {
            *status = WorkerStatus {
                total,
                gathering,
                idle,
                building,
            };
        }
    }

    /// Manages worker gathering behavior
    fn manage_workers(&mut self, game: &Game) {
        let self_player = match game.self_() {
            Some(p) => p,
            None => return,
        };

        let my_units = self_player.get_units();

        // Find all worker units
        let workers: Vec<_> = my_units
            .iter()
            .filter(|u| u.get_type().is_worker() && u.is_idle())
            .collect();

        // Get available minerals
        let static_minerals = game.get_static_minerals();
        let minerals: Vec<_> = static_minerals
            .iter()
            .filter(|m| m.exists() && m.get_resources() > 0)
            .collect();

        // Assign idle workers to gather minerals
        for worker in workers {
            if let Some(mineral) = minerals.first() {
                if let Err(e) = worker.gather(mineral) {
                    game.draw_text_screen((10, 50), &format!("Worker error: {:?}", e));
                }
            }
        }
    }

    /// Manages unit production
    fn manage_production(&self, game: &Game) {
        let self_player = match game.self_() {
            Some(p) => p,
            None => return,
        };

        let race = self_player.get_race();

        // Build supply if needed
        if self_player.supply_total() - self_player.supply_used() < 2 {
            self.build_supply(game, &self_player, race);
        }

        // Train workers from bases
        self.train_workers(game, &self_player, race);
    }

    fn build_supply(&self, game: &Game, player: &Player, race: Race) {
        let supply_type = match race {
            Race::Terran => UnitType::Terran_Supply_Depot,
            Race::Protoss => UnitType::Protoss_Pylon,
            Race::Zerg => UnitType::Zerg_Overlord,
            _ => return,
        };

        if race == Race::Zerg {
            // Zerg builds overlords from larva
            if let Some(larva) = player
                .get_units()
                .iter()
                .find(|u| u.get_type() == UnitType::Zerg_Larva)
            {
                larva.train(supply_type).ok();
            }
        } else {
            // Terran and Protoss build supply structures
            if let Some(worker) = player
                .get_units()
                .iter()
                .find(|u| u.get_type().is_worker() && !u.is_constructing())
            {
                // Find a build location near the start location
                if let Some(start_location) = game.get_start_locations().first() {
                    let _search_position = start_location.to_position();

                    for offset_x in -5..5 {
                        for offset_y in -5..5 {
                            let tile = TilePosition {
                                x: start_location.x + offset_x,
                                y: start_location.y + offset_y,
                            };

                            if game
                                .can_build_here(worker, tile, supply_type, true)
                                .unwrap_or(false)
                            {
                                worker.build(supply_type, tile).ok();
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn train_workers(&self, game: &Game, player: &Player, race: Race) {
        let worker_type = match race {
            Race::Terran => UnitType::Terran_SCV,
            Race::Protoss => UnitType::Protoss_Probe,
            Race::Zerg => UnitType::Zerg_Drone,
            _ => return,
        };

        let base_types = match race {
            Race::Terran => vec![UnitType::Terran_Command_Center],
            Race::Protoss => vec![UnitType::Protoss_Nexus],
            Race::Zerg => vec![
                UnitType::Zerg_Hatchery,
                UnitType::Zerg_Lair,
                UnitType::Zerg_Hive,
            ],
            _ => return,
        };

        if race == Race::Zerg {
            // Zerg trains from larva
            if let Some(larva) = player
                .get_units()
                .iter()
                .find(|u| u.get_type() == UnitType::Zerg_Larva && u.is_idle())
            {
                if game.can_make(None, worker_type).unwrap_or(false) {
                    larva.train(worker_type).ok();
                }
            }
        } else {
            // Terran and Protoss train from bases
            for base in player
                .get_units()
                .iter()
                .filter(|u| base_types.contains(&u.get_type()) && u.is_idle() && u.is_completed())
            {
                if game.can_make(Some(base), worker_type).unwrap_or(false) {
                    base.train(worker_type).ok();
                }
            }
        }
    }

    /// Draw debug information
    fn draw_debug_info(&self, game: &Game) {
        if let Some(player) = game.self_() {
            game.draw_text_screen(
                (10, 10),
                &format!("RustBot - Frame: {}", game.get_frame_count()),
            );
            game.draw_text_screen(
                (10, 20),
                &format!("Minerals: {} | Gas: {}", player.minerals(), player.gas()),
            );
            game.draw_text_screen(
                (10, 30),
                &format!(
                    "Supply: {}/{}",
                    player.supply_used() / 2,
                    player.supply_total() / 2
                ),
            );
        }
    }
}

impl AiModule for RustBot {
    fn on_start(&mut self, game: &Game) {
        game.send_text("RustBot initialized!");
        println!("Game started on map: {}", game.map_file_name());

        // Note: set_local_speed and enable_flag require mutable Game reference
        // These are typically called from different contexts in BWAPI
    }

    fn on_frame(&mut self, game: &Game) {
        self.draw_debug_info(game);
        self.manage_workers(game);
        self.manage_production(game);

        // Update worker status for web dashboard
        self.update_worker_status(game);
    }

    fn on_unit_create(&mut self, _game: &Game, unit: Unit) {
        println!(
            "Unit created: {:?} (ID: {})",
            unit.get_type(),
            unit.get_id()
        );
    }

    fn on_unit_destroy(&mut self, _game: &Game, unit: Unit) {
        println!(
            "Unit destroyed: {:?} (ID: {})",
            unit.get_type(),
            unit.get_id()
        );
    }

    fn on_unit_complete(&mut self, _game: &Game, unit: Unit) {
        println!("Unit completed: {:?}", unit.get_type());
    }

    fn on_end(&mut self, _game: &Game, is_winner: bool) {
        if is_winner {
            println!("Victory!");
        } else {
            println!("Defeat!");
        }
    }
}
