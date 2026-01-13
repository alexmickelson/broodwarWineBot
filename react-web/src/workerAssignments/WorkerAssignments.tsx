import React from 'react';
import { useWorkerStatus } from './workerAssignmentsHooks';
import { ExpandableSection } from '../components/ExpandableSection';
import { LoadingState } from '../components/LoadingState';
import { EmptyState } from '../components/EmptyState';
import { DataField } from '../components/DataField';
import type { WorkerAssignment, WorkerAssignmentType } from './workerAssignmentsService';

interface WorkerCardData extends WorkerAssignment {
  workerId: number;
}

const WorkerCard: React.FC<{ worker: WorkerCardData }> = ({ worker }) => {
  const typeClass = worker.assignment_type.toLowerCase() as 'gathering' | 'scouting' | 'building';
  
  return (
    <div className="bg-bg-secondary border border-border-primary rounded p-4">
      <div className="flex justify-between items-center mb-3 pb-2 border-b border-border-accent">
        <span className="text-text-muted text-xs uppercase tracking-wider">Worker</span>
        <span className="text-text-accent font-bold text-lg">#{worker.workerId}</span>
      </div>
      <div className="flex flex-col gap-2">
        <DataField 
          label="assignment_type" 
          value={worker.assignment_type}
          type="enum"
          enumType={typeClass}
        />
        {worker.target_unit != null && (
          <DataField 
            label="target_unit" 
            value={worker.target_unit}
            type="number"
          />
        )}
        {worker.target_position && (
          <DataField 
            label="target_position" 
            value={`(${worker.target_position[0]}, ${worker.target_position[1]})`}
            type="tuple"
          />
        )}
      </div>
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
        <span className="bg-text-accent/20 text-text-accent px-3 py-1 rounded-full text-sm font-bold">
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

    Object.entries(assignments.worker_assignments).forEach(([workerIdStr, assignment]) => {
      grouped[assignment.assignment_type].push({
        ...assignment,
        workerId: parseInt(workerIdStr),
      });
    });

    return (
      <>
        <AssignmentGroup type="Gathering" workers={grouped.Gathering} />
        <AssignmentGroup type="Scouting" workers={grouped.Scouting} />
        <AssignmentGroup type="Building" workers={grouped.Building} />
      </>
    );
  };

  return (
    <ExpandableSection title="Worker Assignments">
      {renderContent()}
    </ExpandableSection>
  );
};
