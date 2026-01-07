use crate::utils::game_status::SharedStatus;
use crate::utils::map_status;
use crate::utils::worker_management;
use rsbwapi::*;

pub struct RustBot {
    status: SharedStatus,
}

impl RustBot {
    pub fn new(status: SharedStatus) -> Self {
        Self { status }
    }
}

impl AiModule for RustBot {
    fn on_start(&mut self, game: &Game) {
        game.send_text("RustBot initialized!");
        println!("Game started on map: {}", game.map_file_name());
    }

    fn on_frame(&mut self, game: &Game) {
        worker_management::draw_worker_resource_lines(game);
        worker_management::draw_worker_ids(game);

        worker_management::update_assignments(game, &self.status);
        worker_management::enforce_assignments(game, &self.status);

        if game.get_frame_count() % 24 == 0 {
            map_status::update_map_data(game, &self.status);
        }
    }

    fn on_unit_create(&mut self, game: &Game, _unit: Unit) {}

    fn on_unit_destroy(&mut self, _game: &Game, _unit: Unit) {}

    fn on_unit_complete(&mut self, _game: &Game, _unit: Unit) {}

    fn on_end(&mut self, _game: &Game, is_winner: bool) {
        if is_winner {
            println!("Victory!");
        } else {
            println!("Defeat!");
        }
    }
}
