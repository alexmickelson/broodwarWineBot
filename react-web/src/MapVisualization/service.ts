const BASE_URL = `http://localhost:3333`;

export interface UnitInfo {
  x: number;
  y: number;
  unit_type: string;
  is_ally: boolean;
}

export interface ResourceInfo {
  x: number;
  y: number;
  resource_type: string;
  amount: number;
}

export interface MapData {
  width: number;
  height: number;
  walkability: boolean[][];
  explored: boolean[][];
  units: UnitInfo[];
  resources: ResourceInfo[];
}

export interface MapSnapshot {
  map_data: MapData;
  frame_count: number;
}

export async function fetchMap(): Promise<MapSnapshot> {
  const response = await fetch(`${BASE_URL}/map`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
