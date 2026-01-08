// Generic expandable section component
// Manages expanding/collapsing UI sections with smooth animations

const sections = new Map();

export function registerSection(sectionId, options = {}) {
  const config = {
    onExpand: options.onExpand || null,
    onCollapse: options.onCollapse || null,
    defaultExpanded: options.defaultExpanded !== false,
  };

  sections.set(sectionId, {
    ...config,
    isExpanded: config.defaultExpanded,
  });

  // Setup the section header to be clickable (defer until DOM is ready)
  setTimeout(() => setupSectionHeader(sectionId), 0);

  // If section starts expanded, trigger onExpand callback (defer until after setup)
  if (config.defaultExpanded && config.onExpand) {
    setTimeout(() => config.onExpand(), 0);
  } else if (!config.defaultExpanded) {
    // Start collapsed - don't call onCollapse since we're just initializing
    setTimeout(() => {
      const container = document.getElementById(`${sectionId}-container`);
      const toggle = document.querySelector(
        `.section-toggle[data-section="${sectionId}"]`
      );

      if (container && toggle) {
        container.classList.add("collapsed");
        toggle.classList.add("collapsed");
        toggle.textContent = "▶";
      }
    }, 0);
  }

  return {
    expand: () => expand(sectionId),
    collapse: () => collapse(sectionId),
    toggle: () => toggle(sectionId),
    isExpanded: () => sections.get(sectionId)?.isExpanded || false,
  };
}

function setupSectionHeader(sectionId) {
  // Find the header element (either by data-section on toggle or by ID pattern)
  const toggleElement = document.querySelector(
    `.section-toggle[data-section="${sectionId}"]`
  );

  if (!toggleElement) return;

  // Make the entire parent (h2) clickable
  const header = toggleElement.closest("h2");
  if (header) {
    header.classList.add("section-header");
    header.style.cursor = "pointer";

    // Remove any existing click handlers and add new one
    const clickHandler = (e) => {
      e.preventDefault();
      toggle(sectionId);
    };

    header.removeEventListener("click", clickHandler);
    header.addEventListener("click", clickHandler);
  }
}

export function expand(sectionId) {
  const section = sections.get(sectionId);
  if (!section || section.isExpanded) return;

  const container = document.getElementById(`${sectionId}-container`);
  const toggle = document.querySelector(
    `.section-toggle[data-section="${sectionId}"]`
  );

  if (container && toggle) {
    // Remove collapsed class to trigger CSS transition
    container.classList.remove("collapsed");
    toggle.classList.remove("collapsed");
    toggle.textContent = "▼";
    section.isExpanded = true;

    if (section.onExpand) {
      section.onExpand();
    }
  }
}

export function collapse(sectionId) {
  const section = sections.get(sectionId);
  if (!section || !section.isExpanded) return;

  const container = document.getElementById(`${sectionId}-container`);
  const toggle = document.querySelector(
    `.section-toggle[data-section="${sectionId}"]`
  );

  if (container && toggle) {
    // Add collapsed class to trigger CSS transition
    container.classList.add("collapsed");
    toggle.classList.add("collapsed");
    toggle.textContent = "▶";
    section.isExpanded = false;

    if (section.onCollapse) {
      section.onCollapse();
    }
  }
}

export function toggle(sectionId) {
  const section = sections.get(sectionId);
  if (!section) return;

  if (section.isExpanded) {
    collapse(sectionId);
  } else {
    expand(sectionId);
  }
}

export function isExpanded(sectionId) {
  return sections.get(sectionId)?.isExpanded || false;
}

export function getSavedState() {
  const state = {};
  sections.forEach((section, id) => {
    state[id] = !section.isExpanded; // Save as "collapsed" state
  });
  return state;
}

export function restoreState(state) {
  Object.entries(state).forEach(([sectionId, isCollapsed]) => {
    if (isCollapsed) {
      collapse(sectionId);
    } else {
      expand(sectionId);
    }
  });
}
