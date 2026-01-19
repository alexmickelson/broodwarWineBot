import React from "react";
import { useWorkerStatus } from "./workerAssignmentsHooks";
import { ExpandableSection } from "../components/ExpandableSection";
import { LoadingState } from "../components/LoadingState";
import { EmptyState } from "../components/EmptyState";
import type {
  WorkerAssignment,
  WorkerAssignmentType,
} from "./workerAssignmentsService";

interface WorkerCardData extends WorkerAssignment {
  workerId: number;
  buildingType?: string;
}

const WorkerCard: React.FC<{ worker: WorkerCardData }> = ({ worker }) => {
  const typeClass = worker.assignment_type.toLowerCase() as
    | "gathering"
    | "scouting"
    | "building";

  const typeColors = {
    gathering: "bg-emerald-500/20 text-emerald-500 border-emerald-500/30",
    scouting: "bg-azure-500/20 text-azure-500 border-azure-500/30",
    building: "bg-amber-500/20 text-amber-500 border-amber-500/30",
  };

  const typeIcons = {
    gathering: "⛏",
    scouting: "👁",
    building: "🏗",
  };

  return (
    <div className={`border-2 rounded-lg p-4 ${typeColors[typeClass]}`}>
      {/* Header with icon and worker ID */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <span className="text-2xl">{typeIcons[typeClass]}</span>
          <span className="font-bold text-lg">Worker #{worker.workerId}</span>
        </div>
      </div>

      {/* Assignment type badge */}
      <div className="mb-2">
        <span className={`inline-block px-3 py-1 rounded-full text-sm font-bold ${typeColors[typeClass]}`}>
          {worker.assignment_type}
        </span>
      </div>

      {/* Building info */}
      {worker.buildingType && (
        <div className="mt-3 pt-3 border-t border-current/20">
          <div className="text-xs opacity-70 mb-1">Building</div>
          <div className="font-bold">
            {worker.buildingType.replace(/^(Terran|Protoss|Zerg)_/, "")}
          </div>
        </div>
      )}

      {/* Target unit */}
      {worker.target_unit != null && (
        <div className="mt-3 pt-3 border-t border-current/20">
          <div className="text-xs opacity-70 mb-1">Target Unit</div>
          <div className="font-mono font-bold">#{worker.target_unit}</div>
        </div>
      )}

      {/* Target position */}
      {worker.target_position && (
        <div className="mt-3 pt-3 border-t border-current/20">
          <div className="text-xs opacity-70 mb-1">Target Position</div>
          <div className="font-mono text-sm">
            ({worker.target_position[0]}, {worker.target_position[1]})
          </div>
        </div>
      )}
    </div>
  );
};

const AssignmentGroup: React.FC<{
  type: WorkerAssignmentType;
  workers: WorkerCardData[];
}> = ({ type, workers }) => {
  if (workers.length === 0) return null;

  const typeClass = type.toLowerCase();

  return (
    <div className={`assignment-group-${typeClass} mb-6`}>
      <div className="flex justify-between items-center mb-3 pb-2 border-b-2 border-border-primary">
        <h3 className="text-text-secondary text-lg font-semibold">{type}</h3>
        <span className="bg-plasma-500/20 text-plasma-500 px-3 py-1 rounded-full text-sm font-bold">
          {workers.length}
        </span>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {workers.map((worker) => (
          <WorkerCard key={worker.workerId} worker={worker} />
        ))}
      </div>
    </div>
  );
};

export const WorkerAssignments: React.FC = () => {
  const { data: assignments, isLoading, error } = useWorkerStatus();

  const renderContent = () => {
    if (isLoading) {
      return <LoadingState message="Waiting for assignment data..." />;
    }

    if (error) {
      return <EmptyState message={`Error: ${error.message}`} />;
    }

    if (!assignments || Object.keys(assignments).length === 0) {
      return <EmptyState message="No worker assignments" />;
    }

    // Group workers by assignment type
    const grouped: Record<WorkerAssignmentType, WorkerCardData[]> = {
      Gathering: [],
      Scouting: [],
      Building: [],
    };

    Object.entries(assignments.worker_assignments).forEach(
      ([workerIdStr, assignment]) => {
        let buildingType: string | undefined;

        // If worker is building, get the building type from build_order
        if (
          assignment.assignment_type === "Building" &&
          assignment.build_order_index != null
        ) {
          buildingType = assignments.build_order[assignment.build_order_index];
        }

        grouped[assignment.assignment_type].push({
          ...assignment,
          workerId: parseInt(workerIdStr),
          buildingType,
        });
      }
    );

    return (
      <>
        <AssignmentGroup type="Building" workers={grouped.Building} />
        <AssignmentGroup type="Gathering" workers={grouped.Gathering} />
        <AssignmentGroup type="Scouting" workers={grouped.Scouting} />
      </>
    );
  };

  return (
    <ExpandableSection title="Worker Assignments">
      {renderContent()}
    </ExpandableSection>
  );
};
