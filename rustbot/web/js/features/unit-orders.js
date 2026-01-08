// Unit orders feature

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
