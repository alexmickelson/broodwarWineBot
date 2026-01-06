use crate::status::{SharedStatus, WorkerStatus};
use rsbwapi::*;

/// Updates worker status information (total, gathering, idle, building counts)
pub fn update_worker_status(game: &Game, status: &SharedStatus) {
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

    if let Ok(mut status) = status.lock() {
        status.worker_status = WorkerStatus {
            total,
            gathering,
            idle,
            building,
        };
    }
}


pub fn manage_workers(game: &Game) {
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

/// Draws lines between workers and their assigned resources for debugging
pub fn draw_worker_resource_lines(game: &Game) {
    if let Some(player) = game.self_() {
        let my_units = player.get_units();
        let workers: Vec<_> = my_units
            .iter()
            .filter(|u| u.get_type().is_worker())
            .collect();

        for worker in workers {
            // Check if worker has a target (resource being gathered)
            if let Some(target) = worker.get_target() {
                // Check if the target is a resource (mineral or geyser)
                let target_type = target.get_type();
                if target_type.is_mineral_field()
                    || target_type == UnitType::Resource_Vespene_Geyser
                {
                    // Draw a line from worker to resource
                    let worker_pos = worker.get_position();
                    let target_pos = target.get_position();

                    // Use cyan color for the line
                    let color = Color::Cyan;

                    game.draw_line_map(worker_pos, target_pos, color);
                }
            }

            // Also check order target for workers that may be moving to gather
            if let Some(order_target) = worker.get_order_target() {
                let target_type = order_target.get_type();
                if target_type.is_mineral_field()
                    || target_type == UnitType::Resource_Vespene_Geyser
                {
                    let worker_pos = worker.get_position();
                    let target_pos = order_target.get_position();

                    // Use yellow color for order target lines
                    let color = Color::Yellow;

                    game.draw_line_map(worker_pos, target_pos, color);
                }
            }
        }
    }
}
