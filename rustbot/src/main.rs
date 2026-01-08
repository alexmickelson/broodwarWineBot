mod bot;
mod map;
mod status_webserver;
mod utils;

use bot::RustBot;
use status_webserver::start_server;
use std::sync::{Arc, Mutex};
use utils::game_state::GameState;

fn main() {
    println!("Starting RustBot...");

    let game_state = Arc::new(Mutex::new(GameState::default()));

    std::thread::spawn({
        let game_state = game_state.clone();
        move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(start_server(game_state));
        }
    });

    rsbwapi::start(move |_game| RustBot::new(game_state.clone()));
}
