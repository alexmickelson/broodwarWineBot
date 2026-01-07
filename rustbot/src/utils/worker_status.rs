use crate::status::{SharedStatus, WorkerStatus};
use rsbwapi::*;

/// Updates worker status information (total, gathering, idle, building counts)
pub fn update_worker_stats(game: &Game, status: &SharedStatus) {
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
