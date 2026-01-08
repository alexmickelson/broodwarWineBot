// Worker assignments UI rendering

function renderWorkerCard(item, typeClass) {
  return `
    <div class="assignment-card" data-worker-id="${item.workerId}">
      <div class="worker-header">
        <span class="worker-label">Worker</span>
        <span class="worker-id-value">#${item.workerId}</span>
      </div>
      <div class="assignment-data">
        <div class="data-field">
          <span class="field-name">assignment_type:</span>
          <span class="field-value enum ${typeClass}">${
    item.assignment_type
  }</span>
        </div>
        ${renderTargetDisplay(item)}
      </div>
    </div>
  `;
}

function renderTargetDisplay(item) {
  let targetDisplay = "";
  if (item.target_unit !== null && item.target_unit !== undefined) {
    targetDisplay += `
      <div class="data-field">
        <span class="field-name">target_unit:</span>
        <span class="field-value number">${item.target_unit}</span>
      </div>
    `;
  }

  if (item.target_position) {
    const [x, y] = item.target_position;
    targetDisplay += `
      <div class="data-field">
        <span class="field-name">target_position:</span>
        <span class="field-value tuple">(${x}, ${y})</span>
      </div>
    `;
  }
  return targetDisplay;
}

export function update(assignments) {
  const container = document.getElementById("worker-assignments-container");
  if (!container) return;

  const entries = Object.entries(assignments);

  if (entries.length === 0) {
    container.innerHTML = '<div class="empty-data">No worker assignments</div>';
    return;
  }

  const grouped = { Gathering: [], Scouting: [], Building: [] };

  for (const [workerId, assignment] of entries) {
    grouped[assignment.assignment_type].push({ workerId, ...assignment });
  }

  // Check if grid exists, if not create it
  let grid = container.querySelector(".assignments-grid");
  if (!grid) {
    container.innerHTML = '<div class="assignments-grid"></div>';
    grid = container.querySelector(".assignments-grid");
  }

  // Update each assignment group
  for (const [type, items] of Object.entries(grouped)) {
    const typeClass = type.toLowerCase();
    let group = grid.querySelector(`.assignment-group.${typeClass}`);

    if (items.length === 0) {
      // Remove group if it exists and has no items
      if (group) group.remove();
      continue;
    }

    // Create group if it doesn't exist
    if (!group) {
      const groupHtml = `
        <div class="assignment-group ${typeClass}">
          <div class="group-header">
            <h3>${type}</h3>
            <span class="count-badge">${items.length}</span>
          </div>
          <div class="assignment-list"></div>
        </div>
      `;
      grid.insertAdjacentHTML("beforeend", groupHtml);
      group = grid.querySelector(`.assignment-group.${typeClass}`);
    }

    // Update count badge
    const countBadge = group.querySelector(".count-badge");
    if (countBadge) {
      countBadge.textContent = items.length;
    }

    const list = group.querySelector(".assignment-list");
    if (!list) continue;

    // Get existing worker IDs in the DOM
    const existingCards = new Map();
    list.querySelectorAll(".assignment-card").forEach((card) => {
      const workerId = card.dataset.workerId;
      if (workerId) {
        existingCards.set(workerId, card);
      }
    });

    // Get new worker IDs
    const newWorkerIds = new Set(items.map((item) => item.workerId));

    // Remove cards that no longer exist
    existingCards.forEach((card, workerId) => {
      if (!newWorkerIds.has(workerId)) {
        card.remove();
      }
    });

    // Update or add cards
    items.forEach((item) => {
      const existingCard = existingCards.get(item.workerId);
      const newCardHtml = renderWorkerCard(item, typeClass);

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
        list.insertAdjacentHTML("beforeend", newCardHtml);
      }
    });
  }

  // Remove empty groups
  grid.querySelectorAll(".assignment-group").forEach((group) => {
    const type = group.classList.contains("gathering")
      ? "Gathering"
      : group.classList.contains("scouting")
      ? "Scouting"
      : "Building";
    if (grouped[type].length === 0) {
      group.remove();
    }
  });
}

export function createSection() {
  return `
    <h2>
      <span class="section-toggle" data-section="worker-assignments">▼</span>
      Worker Assignments
    </h2>
    <div id="worker-assignments-container" class="data-structure-container collapsible-content">
      <div class="loading">Waiting for assignment data...</div>
    </div>
  `;
}
