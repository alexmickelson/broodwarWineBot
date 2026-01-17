const BASE_URL = `http://localhost:3333`;

export type WorkerAssignmentType = "Gathering" | "Scouting" | "Building";

export interface WorkerAssignment {
  assignment_type: WorkerAssignmentType;
  target_unit: number | null;
  target_position: [number, number] | null;
  build_order_index: number | null;
}

export interface WorkerStatusSnapshot {
  worker_assignments: Record<string, WorkerAssignment>;
  build_order: string[];
  frame_count: number;
}

export async function fetchWorkerStatus(): Promise<WorkerStatusSnapshot> {
  const response = await fetch(`${BASE_URL}/worker-status`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.json();
}
