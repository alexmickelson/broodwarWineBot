// Main application entry point
import * as service from "./service.js";
import * as ui from "./ui.js";
import * as pollingControls from "./pollingControls.js";
import * as expandableSection from "./expandable-section.js";
import * as workerAssignmentsPolling from "./worker-assignments/worker-assignments-polling.js";
import * as unitOrdersPolling from "./unit-orders/unit-orders-polling.js";

let poller = null;
let isConnected = false;

function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering main status polling...");

  poller = pollingControls.registerPoller("main-status", async () => {
    const data = await service.fetchStatus();

    if (!isConnected) {
      isConnected = true;
      console.log("Connected to server");
    }

    ui.update(data);
  });

  poller.start();
}

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

  // Section toggle listeners are now set up by expandable-section.js
  // when sections are registered

  // Reconnect on visibility change
  document.addEventListener("visibilitychange", () => {
    if (!document.hidden && !isConnected) {
      startPolling();
    }
  });
}

function init() {
  // Generate entire UI
  ui.init();

  // Initialize expandable sections
  workerAssignmentsPolling.init();
  unitOrdersPolling.init();

  // Set up event listeners
  setupEventListeners();

  // Start polling
  startPolling();
}

// Initialize app when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
