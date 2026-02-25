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
  target_player: string | null;
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

export interface PlayerLocationInfo {
  starting_location: [number, number];
  player_name: string;
  path_from_my_base: Array<[number, number]> | null;
}

export interface StartLocationsSnapshot {
  locations: PlayerLocationInfo[];
  frame_count: number;
}

export async function fetchStartLocations(): Promise<string[]> {
  const response = await fetch(`${BASE_URL}/start-locations`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  const data: StartLocationsSnapshot = await response.json();
  return data.locations.map((loc) => loc.player_name);
}

export async function updateSquadTargetPlayer(
  squadName: string,
  targetPlayer: string,
): Promise<void> {
  const response = await fetch(
    `${BASE_URL}/military-assignments/update-target-player`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        squad_name: squadName,
        target_player: targetPlayer,
      }),
    },
  );
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
}
