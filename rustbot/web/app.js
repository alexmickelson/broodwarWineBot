// WebSocket connection
let ws = null;
let reconnectTimer = null;

// DOM elements
const connectionIndicator = document.getElementById("connection-indicator");
const connectionText = document.getElementById("connection-text");
const mapContainer = document.getElementById("map-container");
const totalWorkers = document.getElementById("total-workers");
const gatheringWorkers = document.getElementById("gathering-workers");
const idleWorkers = document.getElementById("idle-workers");
const buildingWorkers = document.getElementById("building-workers");

function connect() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const wsUrl = `${protocol}//${window.location.host}/ws`;

  console.log("Connecting to WebSocket:", wsUrl);
  ws = new WebSocket(wsUrl);

  ws.onopen = () => {
    console.log("WebSocket connected");
    connectionIndicator.className = "status-dot connected";
    connectionText.textContent = "Connected";

    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      updateUI(data);
    } catch (err) {
      console.error("Error parsing message:", err);
    }
  };

  ws.onerror = (error) => {
    console.error("WebSocket error:", error);
  };

  ws.onclose = () => {
    console.log("WebSocket disconnected");
    connectionIndicator.className = "status-dot disconnected";
    connectionText.textContent = "Disconnected - Reconnecting...";

    // Attempt to reconnect after 2 seconds
    reconnectTimer = setTimeout(() => {
      connect();
    }, 2000);
  };
}

function updateUI(data) {
  // Update worker stats
  if (data.worker_status) {
    totalWorkers.textContent = data.worker_status.total;
    gatheringWorkers.textContent = data.worker_status.gathering;
    idleWorkers.textContent = data.worker_status.idle;
    buildingWorkers.textContent = data.worker_status.building;
  }

  // Update map
  if (data.map_svg) {
    mapContainer.innerHTML = data.map_svg;
  }
}

// Connect when page loads
connect();

// Handle page visibility changes - reconnect when page becomes visible
document.addEventListener("visibilitychange", () => {
  if (!document.hidden && (!ws || ws.readyState !== WebSocket.OPEN)) {
    connect();
  }
});
