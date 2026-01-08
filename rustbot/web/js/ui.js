// UI update and rendering logic
import * as state from "./state.js";
import * as buildOrder from "./build-order/build-order-ui.js";
import * as workerAssignments from "./worker-assignments/worker-assignments-ui.js";
import * as larvae from "./larvae/larvae-ui.js";
import * as unitOrders from "./unit-orders/unit-orders-ui.js";
import * as map from "./map/map-ui.js";
import * as gameSpeed from "./game-speed/game-speed-ui.js";

export function init() {
  const app = document.getElementById("app");

  app.innerHTML = `
    <div class="container">
      ${gameSpeed.createSection()}
      <div class="scrollable-content">
        ${workerAssignments.createSection()}
        ${larvae.createSection()}
        ${unitOrders.createSection()}
        ${buildOrder.createSection()}
        ${map.createSection()}
        <div class="refresh-note">Live updates via polling</div>
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

export function updateGameSpeed(speed) {
  gameSpeed.update(speed);
}

export function updateWorkerAssignments(assignments) {
  // This is no longer used since worker-assignments polls independently
  // Keeping for backwards compatibility
  workerAssignments.update(assignments);
}

export function updateBuildOrder(order, currentIndex) {
  buildOrder.update(order, currentIndex);
}

export function updateMap(mapSvg) {
  map.update(mapSvg);
}

export function updateLarvaeAssignments(responsibilities) {
  larvae.update(responsibilities);
}

export function updateUnitOrders(orders) {
  unitOrders.update(orders);
}

export function update(data) {
  const collapsedState = state.saveCollapsedState();

  if (data.game_speed !== undefined) {
    updateGameSpeed(data.game_speed);
  }

  // Worker assignments now polls independently via /worker-status
  // No longer updating from /status endpoint

  if (data.larva_responsibilities) {
    updateLarvaeAssignments(data.larva_responsibilities);
  }

  if (data.unit_orders) {
    updateUnitOrders(data.unit_orders);
  }

  if (data.build_order) {
    updateBuildOrder(data.build_order, data.build_order_index);
  }

  if (data.map_svg) {
    updateMap(data.map_svg);
  }

  state.restoreCollapsedState(collapsedState);
}
