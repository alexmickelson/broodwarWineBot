import React from 'react';
import { useLarvae } from './larvaeAssignmentsHooks';
import { ExpandableSection } from '../components/ExpandableSection';
import { LoadingState } from '../components/LoadingState';
import { EmptyState } from '../components/EmptyState';
import { DataField } from '../components/DataField';
import { DataCard } from '../components/DataCard';

export const LarvaeAssignments: React.FC = () => {
  const { data: responsibilities, isLoading, error } = useLarvae();

  const renderContent = () => {
    if (isLoading) {
      return <LoadingState message="Waiting for larvae data..." />;
    }

    if (error) {
      return <EmptyState message={`Error: ${error.message}`} />;
    }

    if (!responsibilities || Object.keys(responsibilities.larva_responsibilities).length === 0) {
      return <EmptyState message="No larvae assignments" />;
    }

    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {Object.entries(responsibilities.larva_responsibilities).map(([larvaId, buildOrderIndex]) => (
          <DataCard key={larvaId}>
            <div className="flex justify-between items-center mb-3 pb-2 border-b border-border-accent">
              <span className="text-text-muted text-xs uppercase tracking-wider">Larva</span>
              <span className="text-plasma-500 font-bold text-lg">#{larvaId}</span>
            </div>
            <DataField 
              label="build_order_index" 
              value={buildOrderIndex as number}
              type="number"
            />
          </DataCard>
        ))}
      </div>
    );
  };

  return (
    <ExpandableSection title="Larvae Assignments" defaultExpanded={false}>
      {renderContent()}
    </ExpandableSection>
  );
};
