const BASE_URL = `http://localhost:3333`;

export interface MilitaryUnitInfo {
  unit_id: number;
  unitType: string;
  order: string;
  order_target_position: [number, number] | null;
  current_position: [number, number];
}

export interface SquadData {
  name: string;
  units: MilitaryUnitInfo[];
  target_position: [number, number] | null;
  target_unit: number | null;
  target_path: Array<[number, number]> | null;
  target_path_current_index: number | null;
  target_path_goal_index: number | null;
}

export interface MilitaryAssignmentsSnapshot {
  squads: SquadData[];
  frame_count: number;
}

export async function fetchMilitaryAssignments(): Promise<MilitaryAssignmentsSnapshot> {
  const response = await fetch(`${BASE_URL}/military-assignments`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
