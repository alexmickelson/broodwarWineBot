// Larvae management feature

export function render(responsibilities) {
  const entries = Object.entries(responsibilities);

  if (entries.length === 0) {
    return '<div class="empty-data">No larvae assignments</div>';
  }

  let html = '<div class="assignments-grid">';

  for (const [larvaId, buildOrderIndex] of entries) {
    html += `
      <div class="assignment-card">
        <div class="worker-header">
          <span class="worker-label">Larva</span>
          <span class="worker-id-value">#${larvaId}</span>
        </div>
        <div class="assignment-data">
          <div class="data-field">
            <span class="field-name">build_order_index:</span>
            <span class="field-value number">${buildOrderIndex}</span>
          </div>
        </div>
      </div>
    `;
  }

  html += "</div>";
  return html;
}

export function update(responsibilities) {
  const container = document.getElementById("larvae-assignments-container");
  if (container) {
    container.innerHTML = render(responsibilities);
  }
}

export function createSection() {
  return `
    <h2>
      <span class="section-toggle" data-section="larvae-assignments">▼</span>
      Larvae Assignments
    </h2>
    <div id="larvae-assignments-container" class="data-structure-container collapsible-content">
      <div class="loading">Waiting for larvae data...</div>
    </div>
  `;
}
