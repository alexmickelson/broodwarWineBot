// Game speed control feature

export function update(speed) {
  document.querySelectorAll(".speed-btn").forEach((btn) => {
    btn.classList.remove("active");
  });

  const activeBtn = document.querySelector(`.speed-btn[data-speed="${speed}"]`);
  if (activeBtn) {
    activeBtn.classList.add("active");
  }
}

export function createButtons() {
  const speeds = [
    { value: -1, label: "-1 (Default)" },
    { value: 0, label: "0 (How fast is your computer?)" },
    { value: 1, label: "1 (Fast)" },
    { value: 42, label: "42 (Standard)" },
  ];

  return speeds
    .map(
      (speed) => `
    <button class="speed-btn" data-speed="${speed.value}">${speed.label}</button>
  `
    )
    .join("");
}

export function createSection() {
  return `
    <div class="fixed-header">
      <h2>Game Speed</h2>
      <div class="control-section">
        <div class="button-group" id="speed-controls"></div>
      </div>
    </div>
  `;
}
