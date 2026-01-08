// Game speed polling logic
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";
import { update } from "./game-speed-ui.js";

let poller = null;

export function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering game speed polling...");

  poller = pollingControls.registerPoller("game-speed", async () => {
    const data = await service.fetchGameSpeed();
    update(data.game_speed);
  });

  poller.start();
}

export function stopPolling() {
  if (poller) {
    poller.stop();
    console.log("Stopped game speed polling");
  }
}

export function init() {
  // Game speed always polls (controls are always visible)
  startPolling();
}
