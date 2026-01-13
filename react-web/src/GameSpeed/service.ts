const BASE_URL = `http://localhost:3333`;

export interface Command {
  command: string;
  value: number;
}

export interface GameSpeedSnapshot {
  game_speed: number;
  frame_count: number;
}

export async function sendCommand(command: string, value: number): Promise<boolean> {
  try {
    const response = await fetch(`${BASE_URL}/command`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ command, value }),
    });
    return response.ok;
  } catch (error) {
    console.error('Failed to send command:', error);
    return false;
  }
}

export async function setGameSpeed(speed: number): Promise<boolean> {
  return sendCommand('set_game_speed', speed);
}

export async function fetchGameSpeed(): Promise<GameSpeedSnapshot> {
  const response = await fetch(`${BASE_URL}/game-speed`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
