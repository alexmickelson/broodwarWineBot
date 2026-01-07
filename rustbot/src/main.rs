mod bot;
mod map;
mod status_webserver;
mod utils;

use bot::RustBot;
use status_webserver::start_server;
use std::sync::{Arc, Mutex};
use utils::game_status::GameStatus;

fn main() {
    println!("Starting RustBot...");

    let status = Arc::new(Mutex::new(GameStatus::default()));

    std::thread::spawn({
        let status = status.clone();
        move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(start_server(status));
        }
    });

    rsbwapi::start(move |_game| RustBot::new(status.clone()));
}
