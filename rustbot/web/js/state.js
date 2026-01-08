// Application state management

export function saveCollapsedState() {
  const state = {};

  ["worker-assignments", "larvae-assignments", "unit-orders"].forEach(
    (section) => {
      const container = document.getElementById(`${section}-container`);
      if (container) {
        state[section] = container.style.display === "none";
      }
    }
  );

  return state;
}

export function restoreCollapsedState(state) {
  Object.entries(state).forEach(([section, isCollapsed]) => {
    const container = document.getElementById(`${section}-container`);
    const toggle = document.querySelector(
      `.section-toggle[data-section="${section}"]`
    );

    if (container && toggle) {
      if (isCollapsed) {
        container.style.display = "none";
        toggle.textContent = "▶";
      } else {
        container.style.display = "block";
        toggle.textContent = "▼";
      }
    }
  });
}

export function toggleSection(sectionId) {
  const container = document.getElementById(`${sectionId}-container`);
  const toggle = document.querySelector(
    `.section-toggle[data-section="${sectionId}"]`
  );

  if (container && toggle) {
    const isCollapsed = container.style.display === "none";
    container.style.display = isCollapsed ? "block" : "none";
    toggle.textContent = isCollapsed ? "▼" : "▶";
  }
}
