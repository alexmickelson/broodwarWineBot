// Generic expandable section component
// Manages expanding/collapsing UI sections with optional callbacks

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

  // If section starts expanded, trigger onExpand callback
  if (config.defaultExpanded && config.onExpand) {
    config.onExpand();
  }

  return {
    expand: () => expand(sectionId),
    collapse: () => collapse(sectionId),
    toggle: () => toggle(sectionId),
    isExpanded: () => sections.get(sectionId)?.isExpanded || false,
  };
}

export function expand(sectionId) {
  const section = sections.get(sectionId);
  if (!section || section.isExpanded) return;

  const container = document.getElementById(`${sectionId}-container`);
  const toggle = document.querySelector(
    `.section-toggle[data-section="${sectionId}"]`
  );

  if (container && toggle) {
    container.style.display = "block";
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
    container.style.display = "none";
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
