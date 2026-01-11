// Map visualization feature

// Color constants
const COLORS = {
  WALKABLE_TERRAIN: '#2a4a2a',
  UNWALKABLE_TERRAIN: '#4a4a4a',
  UNEXPLORED: '#000000',
  ALLIED_UNITS: '#0000FF',
  ENEMY_UNITS: '#FF0000',
  MINERALS: '#00FFFF',
  GAS: '#00FF00',
  UNIT_STROKE: '#FFFFFF',
};

function generateMapSvg(mapData) {
  if (!mapData || mapData.width === 0 || mapData.height === 0) {
    return '<svg></svg>';
  }

  // Scale down the map for display (each walk tile is scaled)
  const scale = 3; // pixels per walk tile
  const svgWidth = mapData.width * scale;
  const svgHeight = mapData.height * scale;

  let svg = `<svg width="${svgWidth}" height="${svgHeight}" viewBox="0 0 ${svgWidth} ${svgHeight}" xmlns="http://www.w3.org/2000/svg">`;

  // Draw background (unexplored areas)
  svg += `<rect width="${svgWidth}" height="${svgHeight}" fill="${COLORS.UNEXPLORED}"/>`;

  // Draw explored and walkability
  for (let y = 0; y < mapData.height; y++) {
    for (let x = 0; x < mapData.width; x++) {
      const isExplored = mapData.explored[y]?.[x] || false;
      const isWalkable = mapData.walkability[y]?.[x] || false;

      if (isExplored) {
        const color = isWalkable ? COLORS.WALKABLE_TERRAIN : COLORS.UNWALKABLE_TERRAIN;
        svg += `<rect x="${x * scale}" y="${y * scale}" width="${scale}" height="${scale}" fill="${color}"/>`;
      }
    }
  }

  // Draw resources
  for (const resource of mapData.resources || []) {
    let color = COLORS.MINERALS;
    if (resource.resource_type.includes('Geyser')) {
      color = COLORS.GAS;
    }

    const cx = resource.x * scale + scale / 2;
    const cy = resource.y * scale + scale / 2;
    svg += `<circle cx="${cx}" cy="${cy}" r="${scale * 2}" fill="${color}" opacity="0.8"/>`;
  }

  // Draw units
  for (const unit of mapData.units || []) {
    const color = unit.is_ally ? COLORS.ALLIED_UNITS : COLORS.ENEMY_UNITS;
    const cx = unit.x * scale + scale / 2;
    const cy = unit.y * scale + scale / 2;
    svg += `<circle cx="${cx}" cy="${cy}" r="${scale}" fill="${color}" stroke="${COLORS.UNIT_STROKE}" stroke-width="0.5"/>`;
  }

  svg += '</svg>';
  return svg;
}

export function update(mapData) {
  const container = document.getElementById("map-container");
  if (container) {
    const svg = generateMapSvg(mapData);
    container.innerHTML = svg;
  }
}

export function createLegend() {
  const items = Object.entries(COLORS)
    .filter(([key]) => key !== 'UNIT_STROKE') // Exclude non-legend colors
    .map(([key, color]) => ({
      color,
      label: key
        .split('_')
        .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
        .join(' ')
    }));

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
