// WebSocket connection
let ws = null;
let reconnectTimer = null;

// DOM elements
const connectionIndicator = document.getElementById("connection-indicator");
const connectionText = document.getElementById("connection-text");
const mapContainer = document.getElementById("map-container");
const workerAssignmentsContainer = document.getElementById(
  "worker-assignments-container"
);
const buildOrderContainer = document.getElementById("build-order-container");
const currentSpeedDisplay = document.getElementById("current-speed");

function sendGameSpeed(speed) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    const command = {
      command: "set_game_speed",
      value: speed,
    };
    ws.send(JSON.stringify(command));
    console.log("Sent game speed:", speed);
  } else {
    console.error("WebSocket not connected");
  }
}

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
  // Update game speed display
  if (data.game_speed !== undefined) {
    currentSpeedDisplay.textContent = data.game_speed;
  }

  // Display worker assignments as formatted data structure
  if (data.worker_assignments) {
    workerAssignmentsContainer.innerHTML = formatWorkerAssignments(
      data.worker_assignments
    );
  }

  // Display build order
  if (data.build_order) {
    buildOrderContainer.innerHTML = formatBuildOrder(data.build_order);
  }

  // Update map
  if (data.map_svg) {
    mapContainer.innerHTML = data.map_svg;
  }
}

function formatWorkerAssignments(assignments) {
  const entries = Object.entries(assignments);

  if (entries.length === 0) {
    return '<div class="empty-data">No worker assignments</div>';
  }

  // Group by assignment type
  const grouped = {
    Gathering: [],
    Scouting: [],
    Building: [],
  };

  for (const [workerId, assignment] of entries) {
    grouped[assignment.assignment_type].push({ workerId, ...assignment });
  }

  let html = '<div class="assignments-grid">';

  // Display each group
  for (const [type, items] of Object.entries(grouped)) {
    if (items.length === 0) continue;

    const typeClass = type.toLowerCase();
    html += `
      <div class="assignment-group ${typeClass}">
        <div class="group-header">
          <h3>${type}</h3>
          <span class="count-badge">${items.length}</span>
        </div>
        <div class="assignment-list">
    `;

    for (const item of items) {
      let targetDisplay = "";
      if (item.target_unit !== null && item.target_unit !== undefined) {
        targetDisplay = `
          <div class="data-field">
            <span class="field-name">target_unit:</span>
            <span class="field-value number">${item.target_unit}</span>
          </div>
        `;
      }

      if (item.target_position) {
        const [x, y] = item.target_position;
        targetDisplay += `
          <div class="data-field">
            <span class="field-name">target_position:</span>
            <span class="field-value tuple">(${x}, ${y})</span>
          </div>
        `;
      }

      html += `
        <div class="assignment-card">
          <div class="worker-header">
            <span class="worker-label">Worker</span>
            <span class="worker-id-value">#${item.workerId}</span>
          </div>
          <div class="assignment-data">
            <div class="data-field">
              <span class="field-name">assignment_type:</span>
              <span class="field-value enum ${typeClass}">${item.assignment_type}</span>
            </div>
            ${targetDisplay}
          </div>
        </div>
      `;
    }

    html += `
        </div>
      </div>
    `;
  }

  html += "</div>";
  return html;
}

function formatBuildOrder(buildOrder) {
  if (buildOrder.length === 0) {
    return '<div class="empty-data">No build order set</div>';
  }

  let html = '<div class="build-order-list">';

  buildOrder.forEach((unit, index) => {
    // Remove the race prefix for cleaner display (e.g., "Zerg_Drone" -> "Drone")
    const displayName = unit.replace(/^(Terran|Protoss|Zerg)_/, "");

    html += `
      <div class="build-order-item">
        <span class="build-order-index">${index + 1}</span>
        <span class="build-order-unit">${displayName}</span>
      </div>
    `;
  });

  html += "</div>";
  return html;
}

// Connect when page loads
connect();

// Add event listeners for game speed buttons
document.querySelectorAll(".speed-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    const speed = parseInt(btn.dataset.speed);
    sendGameSpeed(speed);
  });
});

// Handle page visibility changes - reconnect when page becomes visible
document.addEventListener("visibilitychange", () => {
  if (!document.hidden && (!ws || ws.readyState !== WebSocket.OPEN)) {
    connect();
  }
});
