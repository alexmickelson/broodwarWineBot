const BASE_URL = `http://localhost:3333`;

export type BuildOrderItem =
  | { type: 'Unit'; unit_type: string; base_index: number | null }
  | { type: 'Upgrade'; upgrade_type: string }
  | { type: 'Squad'; name: string; role: string; status: string };

export interface BuildOrderSnapshot {
  build_order: BuildOrderItem[];
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
