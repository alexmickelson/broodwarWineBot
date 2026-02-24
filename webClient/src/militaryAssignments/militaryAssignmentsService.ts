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
  role: string;
  status: string;
  units: MilitaryUnitInfo[];
  target_position: [number, number] | null;
  target_path: Array<[number, number]> | null;
  target_path_index: number | null;
  target_percentage: number;
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

export async function updateSquadTargetPercentage(
  squadName: string,
  targetPercentage: number,
): Promise<void> {
  const response = await fetch(
    `${BASE_URL}/military-assignments/update-target-percentage`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        squad_name: squadName,
        target_percentage: targetPercentage,
      }),
    },
  );
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
}
