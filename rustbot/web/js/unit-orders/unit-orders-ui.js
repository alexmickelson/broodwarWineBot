// Unit orders UI rendering

function renderUnitCard(unitId, order) {
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
        <span class="field-value target-type">${order.target_type}</span>
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

  return `
    <div class="unit-order-card" data-unit-id="${unitId}">
      <div class="unit-order-header">
        <span class="unit-type-badge">${order.unit_type}</span>
        <span class="unit-id-value">#${unitId}</span>
      </div>
      <div class="order-data">
        <div class="data-field">
          <span class="field-name">order:</span>
          <span class="field-value order-name">${order.order_name}</span>
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

export function update(orders) {
  const container = document.getElementById("unit-orders-container");
  if (!container) {
    console.log("unit-orders-container not found");
    return;
  }

  const entries = Object.entries(orders);
  console.log("Unit orders update:", entries.length, "units");

  if (entries.length === 0) {
    container.innerHTML = '<div class="empty-data">No unit orders</div>';
    return;
  }

  // Check if grid exists, if not create it
  let grid = container.querySelector(".unit-orders-grid");
  if (!grid) {
    container.innerHTML = '<div class="unit-orders-grid"></div>';
    grid = container.querySelector(".unit-orders-grid");
  }

  // Get existing unit IDs in the DOM
  const existingCards = new Map();
  grid.querySelectorAll(".unit-order-card").forEach((card) => {
    const unitId = card.dataset.unitId;
    if (unitId) {
      existingCards.set(unitId, card);
    }
  });

  // Get new unit IDs
  const newUnitIds = new Set(entries.map(([unitId]) => unitId));

  // Remove cards that no longer exist
  existingCards.forEach((card, unitId) => {
    if (!newUnitIds.has(unitId)) {
      card.remove();
    }
  });

  // Update or add cards
  for (const [unitId, order] of entries) {
    const existingCard = existingCards.get(unitId);
    const newCardHtml = renderUnitCard(unitId, order);

    if (existingCard) {
      // Update existing card
      const tempDiv = document.createElement("div");
      tempDiv.innerHTML = newCardHtml;
      const newCard = tempDiv.firstElementChild;

      if (existingCard.outerHTML !== newCard.outerHTML) {
        existingCard.replaceWith(newCard);
      }
    } else {
      // Add new card
      grid.insertAdjacentHTML("beforeend", newCardHtml);
    }
  }
}

export function createSection() {
  return `
    <h2>
      <span class="section-toggle" data-section="unit-orders">▶</span>
      Unit Orders
    </h2>
    <div id="unit-orders-container" class="data-structure-container collapsible-content collapsed">
      <div class="loading">Waiting for unit order data...</div>
    </div>
  `;
}
