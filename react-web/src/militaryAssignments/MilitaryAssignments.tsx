import React from 'react';
import { useMilitaryAssignments } from './militaryAssignmentsHooks';
import { ExpandableSection } from '../components/ExpandableSection';
import { LoadingState } from '../components/LoadingState';
import { EmptyState } from '../components/EmptyState';
import { DataField } from '../components/DataField';
import { DataCard } from '../components/DataCard';
import type { MilitaryUnitData } from './militaryAssignmentsService';

interface MilitaryCardData extends MilitaryUnitData {
  unitId: number;
}

const MilitaryCard: React.FC<{ assignment: MilitaryCardData }> = ({ assignment }) => {
  const progress = assignment.target_path_goal_index && assignment.target_path_current_index
    ? Math.round((assignment.target_path_current_index / assignment.target_path_goal_index) * 100)
    : 0;

  return (
    <DataCard>
      <div className="flex justify-between items-center mb-3 pb-2 border-b border-border-accent">
        <span className="text-text-secondary font-medium">
          {assignment.unitType || 'Military Unit'}
        </span>
        <span className="text-plasma-500 font-bold">#{assignment.unitId}</span>
      </div>
      
      <div className="mb-3">
        <div className="text-text-muted text-xs uppercase mb-2">Assignment</div>
        <div className="flex flex-col gap-2">
          {assignment.target_position ? (
            <DataField 
              label="target_position" 
              value={`(${assignment.target_position[0]}, ${assignment.target_position[1]})`}
              type="tuple"
            />
          ) : (
            <DataField label="target_position" value="None" />
          )}
          
          {assignment.target_unit != null && (
            <DataField label="target_unit" value={assignment.target_unit} type="number" />
          )}
          
          {assignment.target_path && (
            <>
              <DataField label="path_length" value={assignment.target_path.length} type="number" />
              {assignment.target_path_current_index != null && (
                <DataField label="current_index" value={assignment.target_path_current_index} type="number" />
              )}
              {assignment.target_path_goal_index != null && (
                <>
                  <DataField label="goal_index" value={assignment.target_path_goal_index} type="number" />
                  <DataField label="progress" value={`${progress}%`} type="number" />
                </>
              )}
            </>
          )}
        </div>
      </div>
      
      {assignment.order && (
        <div>
          <div className="text-text-muted text-xs uppercase mb-2">Order</div>
          <DataField label="order" value={assignment.order} type="order" />
        </div>
      )}
    </DataCard>
  );
};

export const MilitaryAssignments: React.FC = () => {
  const { data: assignments, isLoading, error } = useMilitaryAssignments();

  const renderContent = () => {
    if (isLoading) {
      return <LoadingState message="Waiting for military assignment data..." />;
    }

    if (error) {
      return <EmptyState message={`Error: ${error.message}`} />;
    }

    if (!assignments || Object.keys(assignments.military_assignments).length === 0) {
      return <EmptyState message="No military assignments" />;
    }

    const assignmentList: MilitaryCardData[] = Object.entries(assignments.military_assignments).map(([unitId, assignment]) => ({
      unitId: parseInt(unitId),
      ...assignment,
    }));

    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {assignmentList.map((assignment) => (
          <MilitaryCard key={assignment.unitId} assignment={assignment} />
        ))}
      </div>
    );
  };

  return (
    <ExpandableSection title="Military Assignments" defaultExpanded={false}>
      {renderContent()}
    </ExpandableSection>
  );
};
