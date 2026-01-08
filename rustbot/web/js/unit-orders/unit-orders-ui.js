// Unit orders feature

export function render(orders) {
  const entries = Object.entries(orders);

  if (entries.length === 0) {
    return '<div class="empty-data">No unit orders</div>';
  }

  let html = '<div class="assignments-grid">';

  for (const [unitId, order] of entries) {
    let targetDisplay = "";
    if (order.target_id !== null && order.target_id !== undefined) {
      targetDisplay += `
        <div class="data-field">
          <span class="field-name">target_id:</span>
          <span class="field-value number">${order.target_id}</span>
        </div>
      `;
    }

    if (order.target_type) {
      targetDisplay += `
        <div class="data-field">
          <span class="field-name">target_type:</span>
          <span class="field-value enum">${order.target_type}</span>
        </div>
      `;
    }

    if (order.target_position) {
      const [x, y] = order.target_position;
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
          <span class="worker-label">${order.unit_type}</span>
          <span class="worker-id-value">#${unitId}</span>
        </div>
        <div class="assignment-data">
          <div class="data-field">
            <span class="field-name">order:</span>
            <span class="field-value enum">${order.order_name}</span>
          </div>
          <div class="data-field">
            <span class="field-name">position:</span>
            <span class="field-value tuple">(${order.current_position[0]}, ${order.current_position[1]})</span>
          </div>
          ${targetDisplay}
        </div>
      </div>
    `;
  }

  html += "</div>";
  return html;
}

export function update(orders) {
  const container = document.getElementById("unit-orders-container");
  if (container) {
    container.innerHTML = render(orders);
  }
}

export function createSection() {
  return `
    <h2>
      <span class="section-toggle" data-section="unit-orders">▼</span>
      Unit Orders
    </h2>
    <div id="unit-orders-container" class="data-structure-container collapsible-content">
      <div class="loading">Waiting for unit order data...</div>
    </div>
  `;
}
