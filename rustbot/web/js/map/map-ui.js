// Map visualization feature

export function update(mapSvg) {
  const container = document.getElementById("map-container");
  if (container) {
    container.innerHTML = mapSvg;
  }
}

export function createLegend() {
  const items = [
    { color: "#2a4a2a", label: "Walkable Terrain" },
    { color: "#4a4a4a", label: "Unwalkable Terrain" },
    { color: "#000000", label: "Unexplored" },
    { color: "#0000ff", label: "Allied Units" },
    { color: "#ff0000", label: "Enemy Units" },
    { color: "#00ffff", label: "Minerals" },
    { color: "#00ff00", label: "Gas" },
  ];

  return `
    <div class="legend">
      ${items
        .map(
          (item) => `
        <div class="legend-item">
          <div class="legend-color" style="background: ${item.color}"></div>
          <span>${item.label}</span>
        </div>
      `
        )
        .join("")}
    </div>
  `;
}

export function createSection() {
  return `
    <h2>
      <span class="section-toggle" data-section="map-visualization">▶</span>
      Map Visualization
    </h2>
    <div id="map-visualization-container" class="collapsible-content collapsed">
      ${createLegend()}
      <div class="map-container" id="map-container">
        <div class="loading">Waiting for map data...</div>
      </div>
    </div>
  `;
}
