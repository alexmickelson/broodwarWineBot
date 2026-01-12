// Main application entry point
import * as service from "./service.js";
import * as ui from "./ui.js";
import * as pollingControls from "./pollingControls.js";
import * as workerAssignmentsPolling from "./worker-assignments/worker-assignments-polling.js";
import * as unitOrdersPolling from "./unit-orders/unit-orders-polling.js";
import * as militaryAssignmentsPolling from "./military-assignments/military-assignments-polling.js";
import * as larvaePolling from "./larvae/larvae-polling.js";
import * as buildOrderPolling from "./build-order/build-order-polling.js";
import * as mapPolling from "./map/map-polling.js";
import * as gameSpeedPolling from "./game-speed/game-speed-polling.js";

function setupEventListeners() {
  // Speed button listeners
  document.querySelectorAll(".speed-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const speed = parseInt(btn.dataset.speed);
      service.setGameSpeed(speed);
    });
  });

  // Poll speed button listeners
  document.querySelectorAll(".poll-speed-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const speed = parseInt(btn.dataset.pollSpeed);
      pollingControls.setPollInterval(speed);

      // Update active button state
      document.querySelectorAll(".poll-speed-btn").forEach((b) => {
        b.classList.remove("active");
      });
      btn.classList.add("active");
    });
  });
}

function init() {
  // Generate entire UI
  ui.init();

  // Initialize all polling modules
  workerAssignmentsPolling.init();
  militaryAssignmentsPolling.init();
  unitOrdersPolling.init();
  larvaePolling.init();
  buildOrderPolling.init();
  mapPolling.init();
  gameSpeedPolling.init();

  // Set up event listeners
  setupEventListeners();
}

// Initialize app when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
