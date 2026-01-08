// Larvae management feature

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
