const BASE_URL = 'http://127.0.0.1:3333';

export type DebugFlag = 
  | 'ShowWorkerAssignments'
  | 'ShowMilitaryAssignments'
  | 'ShowPathToEnemyBase'
  | 'ShowRegions';

export interface DebugFlagsSnapshot {
  debug_flags: DebugFlag[];
  frame_count: number;
}

export const fetchDebugFlags = async (): Promise<DebugFlagsSnapshot> => {
  const response = await fetch(`${BASE_URL}/debug-flags`);
  if (!response.ok) {
    throw new Error('Failed to fetch debug flags');
  }
  return response.json();
};

export const updateDebugFlags = async (flags: DebugFlag[]): Promise<void> => {
  const response = await fetch(`${BASE_URL}/debug-flags`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ debug_flags: flags }),
  });
  if (!response.ok) {
    throw new Error('Failed to update debug flags');
  }
};
