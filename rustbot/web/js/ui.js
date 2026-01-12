// UI update and rendering logic
import * as buildOrder from "./build-order/build-order-ui.js";
import * as workerAssignments from "./worker-assignments/worker-assignments-ui.js";
import * as larvae from "./larvae/larvae-ui.js";
import * as unitOrders from "./unit-orders/unit-orders-ui.js";
import * as militaryAssignments from "./military-assignments/military-assignments-ui.js";
import * as map from "./map/map-ui.js";
import * as gameSpeed from "./game-speed/game-speed-ui.js";

export function init() {
  const app = document.getElementById("app");

  app.innerHTML = `
    <div class="container">
      ${gameSpeed.createSection()}
      <div class="scrollable-content">
        ${workerAssignments.createSection()}
        ${militaryAssignments.createSection()}
        ${larvae.createSection()}
        ${unitOrders.createSection()}
        ${buildOrder.createSection()}
        ${map.createSection()}
      </div>
    </div>
  `;

  // Render speed buttons
  document.getElementById("speed-controls").innerHTML =
    gameSpeed.createButtons();

  // Render poll speed buttons
  document.getElementById("poll-speed-controls").innerHTML =
    gameSpeed.createPollSpeedButtons();
}
