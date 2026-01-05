mod bot;
mod status;

use bot::RustBot;
use status::{create_shared_status, start_server};

fn main() {
    println!("Starting RustBot...");

    // Create shared status for web server
    let status = create_shared_status();
    let status_clone = status.clone();

    // Start web server in a separate thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server(status_clone));
    });

    // Start the bot
    rsbwapi::start(move |_game| RustBot::new(status.clone()));
}
