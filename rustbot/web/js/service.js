// HTTP service for backend communication

const BASE_URL = `${window.location.protocol}//${window.location.host}`;

export async function fetchStatus() {
  const response = await fetch(`${BASE_URL}/status`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}

export async function sendCommand(command, value) {
  const response = await fetch(`${BASE_URL}/command`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ command, value }),
  });
  return response.ok;
}

export async function setGameSpeed(speed) {
  return sendCommand("set_game_speed", speed);
}

export async function fetchWorkerStatus() {
  const response = await fetch(`${BASE_URL}/worker-status`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}

export async function fetchUnitOrders() {
  const response = await fetch(`${BASE_URL}/unit-orders`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
