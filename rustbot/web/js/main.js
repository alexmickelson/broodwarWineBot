// Main application entry point
import * as service from "./service.js";
import * as ui from "./ui.js";
import * as state from "./state.js";

let pollInterval = null;
let isConnected = false;

function startPolling() {
  console.log("Starting polling...");

  // Poll every 500ms
  pollInterval = setInterval(async () => {
    try {
      const data = await service.fetchStatus();

      if (!isConnected) {
        isConnected = true;
        console.log("Connected to server");
      }

      ui.update(data);
    } catch (err) {
      if (isConnected) {
        isConnected = false;
        console.error("Polling error:", err);
      }
    }
  }, 500);

  // Do initial poll immediately
  service
    .fetchStatus()
    .then((data) => {
      isConnected = true;
      ui.update(data);
    })
    .catch((err) => {
      console.error("Initial poll error:", err);
    });
}

function setupEventListeners() {
  // Speed button listeners
  document.querySelectorAll(".speed-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const speed = parseInt(btn.dataset.speed);
      service.setGameSpeed(speed);
    });
  });

  // Section toggle listeners
  document.addEventListener("click", (e) => {
    if (e.target.classList.contains("section-toggle")) {
      const section = e.target.dataset.section;
      state.toggleSection(section);
    }
  });

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
