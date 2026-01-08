// Worker assignments feature
import * as service from "../service.js";
import * as pollingControls from "../pollingControls.js";

let poller = null;

export function startPolling() {
  if (poller) {
    return; // Already registered
  }

  console.log("Registering worker assignments polling...");

  poller = pollingControls.registerPoller("worker-assignments", async () => {
    const data = await service.fetchWorkerStatus();
    update(data.worker_assignments);
  });

  poller.start();
}

export function stopPolling() {
  if (poller) {
    poller.stop();
    console.log("Stopped worker assignments polling");
  }
}

export function render(assignments) {
  const entries = Object.entries(assignments);

  if (entries.length === 0) {
    return '<div class="empty-data">No worker assignments</div>';
  }

  const grouped = { Gathering: [], Scouting: [], Building: [] };

  for (const [workerId, assignment] of entries) {
    grouped[assignment.assignment_type].push({ workerId, ...assignment });
  }

  let html = '<div class="assignments-grid">';

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
        targetDisplay += `
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

    html += "</div></div>";
  }

  html += "</div>";
  return html;
}

export function update(assignments) {
  const container = document.getElementById("worker-assignments-container");
  if (container) {
    container.innerHTML = render(assignments);
  }
}

export function createSection() {
  return `
    <h2>
      <span class="section-toggle" data-section="worker-assignments">▼</span>
      Worker Assignments
    </h2>
    <div id="worker-assignments-container" class="data-structure-container collapsible-content">
      <div class="loading">Waiting for assignment data...</div>
    </div>
  `;
}
