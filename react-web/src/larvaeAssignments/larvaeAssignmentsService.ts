const BASE_URL = `http://localhost:3333`;

export interface LarvaeSnapshot {
  larva_responsibilities: Record<string, number>;
  frame_count: number;
}

export async function fetchLarvae(): Promise<LarvaeSnapshot> {
  const response = await fetch(`${BASE_URL}/larvae`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
