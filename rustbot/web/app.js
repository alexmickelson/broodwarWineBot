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
const assignedGathering = document.getElementById("assigned-gathering");
const assignedScouting = document.getElementById("assigned-scouting");
const assignedBuilding = document.getElementById("assigned-building");
const totalAssignments = document.getElementById("total-assignments");
const assignmentDetails = document.getElementById("assignment-details");

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

  // Update worker assignments
  if (data.worker_assignments) {
    const assignments = Object.values(data.worker_assignments);
    const gatheringCount = assignments.filter(
      (a) => a.assignment_type === "Gathering"
    ).length;
    const scoutingCount = assignments.filter(
      (a) => a.assignment_type === "Scouting"
    ).length;
    const buildingCount = assignments.filter(
      (a) => a.assignment_type === "Building"
    ).length;

    assignedGathering.textContent = gatheringCount;
    assignedScouting.textContent = scoutingCount;
    assignedBuilding.textContent = buildingCount;
    totalAssignments.textContent = assignments.length;

    // Show assignment details
    let detailsHTML = '<div class="assignment-list">';
    const assignmentEntries = Object.entries(data.worker_assignments);

    if (assignmentEntries.length > 0) {
      detailsHTML += "<h3>Assignment Details</h3>";
      detailsHTML += '<div class="assignment-table">';

      for (const [workerId, assignment] of assignmentEntries.slice(0, 20)) {
        // Show first 20
        const typeClass = assignment.assignment_type.toLowerCase();
        let targetInfo = "";

        if (assignment.target_unit) {
          targetInfo = `→ Resource #${assignment.target_unit}`;
        } else if (assignment.target_position) {
          const [x, y] = assignment.target_position;
          targetInfo = `→ Position (${x}, ${y})`;
        }

        detailsHTML += `
          <div class="assignment-row ${typeClass}">
            <span class="worker-id">Worker #${workerId}</span>
            <span class="assignment-type">${assignment.assignment_type}</span>
            <span class="assignment-target">${targetInfo}</span>
          </div>
        `;
      }

      if (assignmentEntries.length > 20) {
        detailsHTML += `<div class="more-assignments">... and ${
          assignmentEntries.length - 20
        } more</div>`;
      }

      detailsHTML += "</div>";
    } else {
      detailsHTML += '<p class="no-assignments">No active assignments</p>';
    }

    detailsHTML += "</div>";
    assignmentDetails.innerHTML = detailsHTML;
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
