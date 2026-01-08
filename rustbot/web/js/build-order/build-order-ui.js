// Build order feature

export function render(buildOrder, currentIndex) {
  if (!buildOrder || buildOrder.length === 0) {
    return '<div class="empty-data">No build order set</div>';
  }

  let html = '<div class="build-order-list">';

  buildOrder.forEach((unit, index) => {
    const displayName = unit.replace(/^(Terran|Protoss|Zerg)_/, "");
    const isCurrent = index === currentIndex;
    const isComplete = index < currentIndex;
    const statusClass = isComplete ? "completed" : isCurrent ? "current" : "";

    html += `
      <div class="build-order-item ${statusClass}">
        <span class="build-order-index">${index + 1}</span>
        <span class="build-order-unit">${displayName}</span>
      </div>
    `;
  });

  html += "</div>";
  return html;
}

export function update(buildOrder, currentIndex) {
  const container = document.getElementById("build-order-container");
  if (container) {
    container.innerHTML = render(buildOrder, currentIndex);
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
