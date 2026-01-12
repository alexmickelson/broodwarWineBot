// Military assignments UI rendering

function renderMilitaryCard(item) {
  return `
    <div class="assignment-card" data-unit-id="${item.unitId}">
      <div class="military-header">
        <span class="unit-label">${item.unitType || "Military Unit"}</span>
        <span class="unit-id-value">#${item.unitId}</span>
      </div>
      <div class="assignment-data">
        ${renderAssignmentDetails(item)}
        ${renderOrderDetails(item)}
      </div>
    </div>
  `;
}

function renderAssignmentDetails(item) {
  let assignmentHtml = '<div class="data-section"><div class="section-title">Assignment</div>';
  
  if (item.target_position) {
    const [x, y] = item.target_position;
    assignmentHtml += `
      <div class="data-field">
        <span class="field-name">target_position:</span>
        <span class="field-value tuple">(${x}, ${y})</span>
      </div>
    `;
  } else {
    assignmentHtml += `
      <div class="data-field">
        <span class="field-name">target_position:</span>
        <span class="field-value null">None</span>
      </div>
    `;
  }

  if (item.target_unit !== null && item.target_unit !== undefined) {
    assignmentHtml += `
      <div class="data-field">
        <span class="field-name">target_unit:</span>
        <span class="field-value number">${item.target_unit}</span>
      </div>
    `;
  }

  assignmentHtml += '</div>';
  return assignmentHtml;
}

function renderOrderDetails(item) {
  let orderHtml = '<div class="data-section"><div class="section-title">Current Order</div>';
  
  if (item.order) {
    orderHtml += `
      <div class="data-field">
        <span class="field-name">order:</span>
        <span class="field-value enum">${item.order}</span>
      </div>
    `;
  }

  if (item.order_target_position) {
    const [x, y] = item.order_target_position;
    orderHtml += `
      <div class="data-field">
        <span class="field-name">order_target:</span>
        <span class="field-value tuple">(${x}, ${y})</span>
      </div>
    `;
  }

  orderHtml += '</div>';
  return orderHtml;
}

export function update(militaryData) {
  const container = document.getElementById("military-assignments-container");
  if (!container) return;

  const entries = Object.entries(militaryData);

  if (entries.length === 0) {
    container.innerHTML = '<div class="empty-state">No military units</div>';
    return;
  }

  const grouped = entries.reduce((acc, [unitId, data]) => {
    const type = data.unitType || "Unknown";
    if (!acc[type]) acc[type] = [];
    acc[type].push({ unitId, ...data });
    return acc;
  }, {});

  let html = "";
  for (const [type, units] of Object.entries(grouped).sort()) {
    html += `
      <div class="unit-type-group">
        <div class="unit-type-header">${type} (${units.length})</div>
        <div class="cards-grid">
          ${units.map((unit) => renderMilitaryCard(unit)).join("")}
        </div>
      </div>
    `;
  }

  container.innerHTML = html;
}

export function createSection() {
  return `
    <h2>
      <span class="section-toggle" data-section="military-assignments">▼</span>
      Military Assignments
    </h2>
    <div id="military-assignments-container" class="data-structure-container collapsible-content">
      <div class="loading">Waiting for military data...</div>
    </div>
  `;
}
