import React from "react";
import { useMilitaryAssignments } from "./militaryAssignmentsHooks";
import { ExpandableSection } from "../components/ExpandableSection";
import { LoadingState } from "../components/LoadingState";
import { EmptyState } from "../components/EmptyState";
import { DataField } from "../components/DataField";
import { DataCard } from "../components/DataCard";
import type { SquadData, MilitaryUnitInfo } from "./militaryAssignmentsService";

const UnitInfo: React.FC<{ unit: MilitaryUnitInfo }> = ({ unit }) => {
  return (
    <div className="p-3 bg-background-elevated rounded border border-border-accent">
      <div className="flex justify-between items-center mb-2">
        <span className="text-text-primary font-medium text-sm">
          {unit.unitType}
        </span>
        <span className="text-plasma-500 text-xs font-bold">
          #{unit.unit_id}
        </span>
      </div>

      <div className="flex flex-col gap-1.5">
        <DataField
          label="position"
          value={`(${unit.current_position[0]}, ${unit.current_position[1]})`}
          type="tuple"
        />
        <DataField label="order" value={unit.order} type="order" />
        {unit.order_target_position && (
          <DataField
            label="order_target"
            value={`(${unit.order_target_position[0]}, ${unit.order_target_position[1]})`}
            type="tuple"
          />
        )}
      </div>
    </div>
  );
};

const SquadCard: React.FC<{ squad: SquadData }> = ({ squad }) => {
  const progress =
    squad.target_path_goal_index && squad.target_path_current_index
      ? Math.round(
          (squad.target_path_current_index / squad.target_path_goal_index) * 100
        )
      : 0;

  return (
    <DataCard>
      <div className="flex justify-between items-center mb-4 pb-3 border-b-2 border-plasma-500">
        <span className="text-text-primary font-bold text-lg">
          {squad.name || "Unnamed Squad"}
        </span>
        <span className="text-plasma-500 font-bold text-sm">
          {squad.units.length} {squad.units.length === 1 ? "unit" : "units"}
        </span>
      </div>

      <div className="mb-4">
        <div className="text-text-muted text-xs uppercase mb-2 font-semibold">
          Squad Mission
        </div>
        <div className="flex flex-col gap-2">
          {squad.target_position ? (
            <DataField
              label="target_position"
              value={`(${squad.target_position[0]}, ${squad.target_position[1]})`}
              type="tuple"
            />
          ) : (
            <DataField label="target_position" value="None" />
          )}

          {squad.target_unit != null && (
            <DataField
              label="target_unit"
              value={squad.target_unit}
              type="number"
            />
          )}

          {squad.target_path && (
            <>
              <DataField
                label="path_length"
                value={squad.target_path.length}
                type="number"
              />
              {squad.target_path_current_index != null && (
                <DataField
                  label="current_index"
                  value={squad.target_path_current_index}
                  type="number"
                />
              )}
              {squad.target_path_goal_index != null && (
                <>
                  <DataField
                    label="goal_index"
                    value={squad.target_path_goal_index}
                    type="number"
                  />
                  <DataField
                    label="progress"
                    value={`${progress}%`}
                    type="number"
                  />
                </>
              )}
            </>
          )}
        </div>
      </div>

      {squad.units.length > 0 && (
        <div>
          <div className="text-text-muted text-xs uppercase mb-3 font-semibold">
            Squad Members
          </div>
          <div className="flex flex-col gap-2">
            {squad.units.map((unit) => (
              <UnitInfo key={unit.unit_id} unit={unit} />
            ))}
          </div>
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

    if (!assignments || assignments.squads.length === 0) {
      return <EmptyState message="No military squads" />;
    }

    return (
      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
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
