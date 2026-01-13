const BASE_URL = `http://localhost:3333`;

export interface UnitOrder {
  unit_id: number;
  unit_type: string;
  order_name: string;
  target_id: number | null;
  target_type: string | null;
  current_position: [number, number];
  target_position: [number, number] | null;
}

export interface UnitOrdersSnapshot {
  unit_orders: Record<string, UnitOrder>;
  frame_count: number;
}

export async function fetchUnitOrders(): Promise<UnitOrdersSnapshot> {
  const response = await fetch(`${BASE_URL}/unit-orders`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
