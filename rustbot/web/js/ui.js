// UI update and rendering logic
import * as state from "./state.js";
import * as buildOrder from "./features/build-order.js";
import * as workerAssignments from "./features/worker-assignments.js";
import * as larvae from "./features/larvae.js";
import * as unitOrders from "./features/unit-orders.js";
import * as map from "./features/map.js";
import * as gameSpeed from "./features/game-speed.js";

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
}

export function updateGameSpeed(speed) {
  gameSpeed.update(speed);
}

export function updateWorkerAssignments(assignments) {
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

  if (data.worker_assignments) {
    updateWorkerAssignments(data.worker_assignments);
  }

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
