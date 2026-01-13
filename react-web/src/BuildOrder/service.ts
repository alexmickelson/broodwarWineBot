const BASE_URL = `http://localhost:3333`;

export interface BuildOrderSnapshot {
  build_order: string[];
  build_order_index: number;
  frame_count: number;
}

export async function fetchBuildOrder(): Promise<BuildOrderSnapshot> {
  const response = await fetch(`${BASE_URL}/build-order`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
