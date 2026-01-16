import React from "react";
import { useMilitaryAssignments } from "./militaryAssignmentsHooks";
import { ExpandableSection } from "../components/ExpandableSection";
import { LoadingState } from "../components/LoadingState";
import { EmptyState } from "../components/EmptyState";
import { SquadCard } from "./SquadCard";

export const MilitaryAssignments: React.FC = () => {
  const { data: assignments, isLoading, error } = useMilitaryAssignments();

  const renderContent = () => {
    if (isLoading) {
      return <LoadingState message="Waiting for military assignment data..." />;
    }

    if (error) {
      return <EmptyState message={`Error: ${error.message}`} />;
    }

    if (!assignments || assignments.squads.length === 0) {
      return <EmptyState message="No military squads" />;
    }

    return (
      <div className="">
        {assignments.squads.map((squad, index) => (
          <SquadCard key={`${squad.name}-${index}`} squad={squad} />
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
