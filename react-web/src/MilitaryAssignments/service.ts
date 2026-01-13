const BASE_URL = `http://localhost:3333`;

export interface MilitaryUnitData {
  unitType: string;
  target_position: [number, number] | null;
  target_unit: number | null;
  target_path: Array<[number, number]> | null;
  target_path_current_index: number | null;
  target_path_goal_index: number | null;
  order: string;
  order_target_position: [number, number] | null;
}

export interface MilitaryAssignmentsSnapshot {
  military_assignments: Record<string, MilitaryUnitData>;
  frame_count: number;
}

export async function fetchMilitaryAssignments(): Promise<MilitaryAssignmentsSnapshot> {
  const response = await fetch(`${BASE_URL}/military-assignments`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
