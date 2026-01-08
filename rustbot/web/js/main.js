// Main application entry point
import * as service from "./service.js";
import * as ui from "./ui.js";
import * as state from "./state.js";
import * as pollingControls from "./pollingControls.js";

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
