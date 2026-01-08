// Build order feature

export function render(buildOrder) {
  if (!buildOrder || buildOrder.length === 0) {
    return '<div class="empty-data">No build order set</div>';
  }

  let html = '<div class="build-order-list">';

  buildOrder.forEach((unit, index) => {
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

export function update(buildOrder) {
  const container = document.getElementById("build-order-container");
  if (container) {
    container.innerHTML = render(buildOrder);
  }
}

export function createSection() {
  return `
    <h2>Build Order</h2>
    <div id="build-order-container" class="data-structure-container">
      <div class="loading">Waiting for build order...</div>
    </div>
  `;
}
