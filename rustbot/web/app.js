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
const larvaeAssignmentsContainer = document.getElementById(
  "larvae-assignments-container"
);
const unitOrdersContainer = document.getElementById(
  "unit-orders-container"
);
const buildOrderContainer = document.getElementById("build-order-container");

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

// Save and restore collapsed state across updates
function saveCollapsedState() {
  const state = {};
  
  // Save worker assignments section state
  const workerContainer = document.getElementById("worker-assignments-container");
  if (workerContainer) {
    state.workerAssignments = workerContainer.classList.contains("collapsed");
  }
  
  // Save larvae assignments section state
  const larvaeContainer = document.getElementById("larvae-assignments-container");
  if (larvaeContainer) {
    state.larvaeAssignments = larvaeContainer.classList.contains("collapsed");
  }
  
  // Save unit orders section state
  const unitOrdersContainer = document.getElementById("unit-orders-container");
  if (unitOrdersContainer) {
    state.unitOrders = unitOrdersContainer.classList.contains("collapsed");
  }
  
  // Save completed build order items state
  const completedItems = document.getElementById("completed-items");
  if (completedItems) {
    state.completedItems = completedItems.style.display === "none";
  }
  
  return state;
}

function restoreCollapsedState(state) {
  // Restore worker assignments section state
  if (state.workerAssignments !== undefined) {
    const workerContainer = document.getElementById("worker-assignments-container");
    const toggle = document.querySelector('.section-toggle[data-section="worker-assignments"]');
    
    if (workerContainer && toggle) {
      if (state.workerAssignments) {
        workerContainer.classList.add("collapsed");
        workerContainer.style.display = "none";
        toggle.textContent = "▶";
      } else {
        workerContainer.classList.remove("collapsed");
        workerContainer.style.display = "block";
        toggle.textContent = "▼";
      }
    }
  }
  
  // Restore larvae assignments section state
  if (state.larvaeAssignments !== undefined) {
    const larvaeContainer = document.getElementById("larvae-assignments-container");
    const toggle = document.querySelector('.section-toggle[data-section="larvae-assignments"]');
    
    if (larvaeContainer && toggle) {
      if (state.larvaeAssignments) {
        larvaeContainer.classList.add("collapsed");
        larvaeContainer.style.display = "none";
        toggle.textContent = "▶";
      } else {
        larvaeContainer.classList.remove("collapsed");
        larvaeContainer.style.display = "block";
        toggle.textContent = "▼";
      }
    }
  }
  
  // Restore unit orders section state
  if (state.unitOrders !== undefined) {
    const unitOrdersContainer = document.getElementById("unit-orders-container");
    const toggle = document.querySelector('.section-toggle[data-section="unit-orders"]');
    
    if (unitOrdersContainer && toggle) {
      if (state.unitOrders) {
        unitOrdersContainer.classList.add("collapsed");
        unitOrdersContainer.style.display = "none";
        toggle.textContent = "▶";
      } else {
        unitOrdersContainer.classList.remove("collapsed");
        unitOrdersContainer.style.display = "block";
        toggle.textContent = "▼";
      }
    }
  }
  
  // Restore completed build order items state
  if (state.completedItems !== undefined) {
    const completedItems = document.getElementById("completed-items");
    const header = document.querySelector('.collapsible[data-target="completed-items"]');
    const icon = header ? header.querySelector(".toggle-icon") : null;
    
    if (completedItems && header) {
      if (state.completedItems) {
        completedItems.style.display = "none";
        header.classList.add("collapsed");
        if (icon) icon.textContent = "▶";
      } else {
        completedItems.style.display = "flex";
        header.classList.remove("collapsed");
        if (icon) icon.textContent = "▼";
      }
    }
  }
}

function updateUI(data) {
  // Save collapsed state before updating
  const collapsedState = saveCollapsedState();
  
  // Update game speed display
  if (data.game_speed !== undefined) {
    // Remove active class from all buttons
    document.querySelectorAll(".speed-btn").forEach((btn) => {
      btn.classList.remove("active");
    });
    // Add active class to the button matching current speed
    const activeBtn = document.querySelector(
      `.speed-btn[data-speed="${data.game_speed}"]`
    );
    if (activeBtn) {
      activeBtn.classList.add("active");
    }
  }

  // Display worker assignments as formatted data structure
  if (data.worker_assignments) {
    workerAssignmentsContainer.innerHTML = formatWorkerAssignments(
      data.worker_assignments
    );
  }

  // Display larvae assignments
  if (data.larva_responsibilities !== undefined && data.build_order !== undefined) {
    larvaeAssignmentsContainer.innerHTML = formatLarvaeAssignments(
      data.larva_responsibilities,
      data.build_order
    );
  }

  // Display unit orders
  if (data.unit_orders) {
    unitOrdersContainer.innerHTML = formatUnitOrders(data.unit_orders);
  }

  // Display build order
  if (data.build_order !== undefined && data.build_order_index !== undefined) {
    buildOrderContainer.innerHTML = formatBuildOrder(
      data.build_order,
      data.build_order_index,
      data.larva_responsibilities || {}
    );
  }

  // Update map
  if (data.map_svg) {
    mapContainer.innerHTML = data.map_svg;
  }
  
  // Restore collapsed state after updating
  restoreCollapsedState(collapsedState);
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

function formatLarvaeAssignments(larvaResponsibilities, buildOrder) {
  const entries = Object.entries(larvaResponsibilities);

  if (entries.length === 0) {
    return '<div class="empty-data">No larvae assigned</div>';
  }

  let html = '<div class="larvae-grid">';

  for (const [larvaId, buildOrderIndex] of entries) {
    const unitType = buildOrder[buildOrderIndex] || "Unknown";
    const displayName = unitType.replace(/^(Terran|Protoss|Zerg)_/, "");

    html += `
      <div class="larva-card">
        <div class="larva-header">
          <span class="larva-label">Larva</span>
          <span class="larva-id-value">#${larvaId}</span>
        </div>
        <div class="larva-data">
          <div class="data-field">
            <span class="field-name">morphing into:</span>
            <span class="field-value unit-type">${displayName}</span>
          </div>
          <div class="data-field">
            <span class="field-name">build order index:</span>
            <span class="field-value number">${buildOrderIndex + 1}</span>
          </div>
        </div>
      </div>
    `;
  }

  html += "</div>";
  return html;
}

function formatUnitOrders(unitOrders) {
  const entries = Object.entries(unitOrders);

  if (entries.length === 0) {
    return '<div class="empty-data">No unit order data</div>';
  }

  let html = '<div class="unit-orders-grid">';

  for (const [unitId, order] of entries) {
    const displayUnitType = order.unit_type.replace(/^(Terran|Protoss|Zerg)_/, "");
    const displayTargetType = order.target_type ? order.target_type.replace(/^(Terran|Protoss|Zerg)_/, "") : "None";

    html += `
      <div class="unit-order-card">
        <div class="unit-order-header">
          <span class="unit-type-badge">${displayUnitType}</span>
          <span class="unit-id-value">#${unitId}</span>
        </div>
        <div class="unit-order-data">
          <div class="data-field">
            <span class="field-name">order:</span>
            <span class="field-value order-name">${order.order_name}</span>
          </div>
          <div class="data-field">
            <span class="field-name">position:</span>
            <span class="field-value tuple">(${order.current_position[0]}, ${order.current_position[1]})</span>
          </div>
          ${order.target_id ? `
          <div class="data-field">
            <span class="field-name">target_id:</span>
            <span class="field-value number">${order.target_id}</span>
          </div>
          ` : ''}
          ${order.target_type ? `
          <div class="data-field">
            <span class="field-name">target_type:</span>
            <span class="field-value target-type">${displayTargetType}</span>
          </div>
          ` : ''}
          ${order.target_position ? `
          <div class="data-field">
            <span class="field-name">target_pos:</span>
            <span class="field-value tuple">(${order.target_position[0]}, ${order.target_position[1]})</span>
          </div>
          ` : ''}
        </div>
      </div>
    `;
  }

  html += "</div>";
  return html;
}

function formatBuildOrder(buildOrder, currentIndex, larvaResponsibilities) {
  if (buildOrder.length === 0) {
    return '<div class="empty-data">No build order set</div>';
  }

  const completedItems = buildOrder.slice(0, currentIndex);
  const currentItem = currentIndex < buildOrder.length ? buildOrder[currentIndex] : null;
  const upcomingItems = buildOrder.slice(currentIndex + 1);

  // Get larvae assigned to current item
  const assignedLarvae = Object.entries(larvaResponsibilities)
    .filter(([_, idx]) => idx === currentIndex)
    .map(([larvaId, _]) => larvaId);

  let html = '<div class="build-order-sections">';

  // Current build order item
  if (currentItem) {
    const displayName = currentItem.replace(/^(Terran|Protoss|Zerg)_/, "");
    html += `
      <div class="build-order-section current-section">
        <h3 class="section-title">Current</h3>
        <div class="build-order-item current-item">
          <span class="build-order-index">${currentIndex + 1}</span>
          <span class="build-order-unit">${displayName}</span>
        </div>
    `;

    // Show assigned larvae
    if (assignedLarvae.length > 0) {
      html += `
        <div class="larva-assignments">
          <div class="larva-header">Assigned Larvae:</div>
          <div class="larva-list">
            ${assignedLarvae.map(id => `<span class="larva-id">#${id}</span>`).join('')}
          </div>
        </div>
      `;
    }

    html += '</div>';
  }

  // Upcoming items
  if (upcomingItems.length > 0) {
    html += `
      <div class="build-order-section upcoming-section">
        <h3 class="section-title">Upcoming (${upcomingItems.length})</h3>
        <div class="build-order-list">
    `;

    upcomingItems.forEach((unit, index) => {
      const displayName = unit.replace(/^(Terran|Protoss|Zerg)_/, "");
      const itemIndex = currentIndex + 1 + index;
      html += `
        <div class="build-order-item upcoming-item">
          <span class="build-order-index">${itemIndex + 1}</span>
          <span class="build-order-unit">${displayName}</span>
        </div>
      `;
    });

    html += '</div></div>';
  }

  // Completed items (collapsible)
  if (completedItems.length > 0) {
    html += `
      <div class="build-order-section completed-section">
        <h3 class="section-title collapsible collapsed" data-target="completed-items">
          <span class="toggle-icon">▶</span>
          Completed (${completedItems.length})
        </h3>
        <div id="completed-items" class="build-order-list collapsed-content" style="display: none;">
    `;

    completedItems.forEach((unit, index) => {
      const displayName = unit.replace(/^(Terran|Protoss|Zerg)_/, "");
      html += `
        <div class="build-order-item completed-item">
          <span class="build-order-index completed">${index + 1}</span>
          <span class="build-order-unit">${displayName}</span>
        </div>
      `;
    });

    html += '</div></div>';
  }

  html += '</div>';
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

// Handle collapsible sections
document.addEventListener("click", (e) => {
  // Handle build order completed section toggle
  if (e.target.closest(".collapsible")) {
    const header = e.target.closest(".collapsible");
    const target = header.dataset.target;
    const content = document.getElementById(target);
    const icon = header.querySelector(".toggle-icon");
    
    if (content) {
      if (content.style.display === "none") {
        content.style.display = "flex";
        header.classList.remove("collapsed");
        if (icon) icon.textContent = "▼";
      } else {
        content.style.display = "none";
        header.classList.add("collapsed");
        if (icon) icon.textContent = "▶";
      }
    }
  }

  // Handle worker assignments section toggle
  if (e.target.classList.contains("section-toggle")) {
    const section = e.target.dataset.section;
    const container = document.getElementById(`${section}-container`);
    
    if (container) {
      if (container.classList.contains("collapsed")) {
        container.classList.remove("collapsed");
        container.style.display = "block";
        e.target.textContent = "▼";
      } else {
        container.classList.add("collapsed");
        container.style.display = "none";
        e.target.textContent = "▶";
      }
    }
  }
});

// Handle page visibility changes - reconnect when page becomes visible
document.addEventListener("visibilitychange", () => {
  if (!document.hidden && (!ws || ws.readyState !== WebSocket.OPEN)) {
    connect();
  }
});
