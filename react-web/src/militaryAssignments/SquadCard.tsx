import React from "react";
import { UnitInfo } from "./UnitInfo";
import type { SquadData } from "./militaryAssignmentsService";

export const SquadCard: React.FC<{ squad: SquadData }> = ({ squad }) => {
  const progress =
    squad.target_path && squad.target_path_index
      ? Math.round((squad.target_path_index / squad.target_path.length) * 100)
      : 0;

  return (
    <div className="">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h3 className="text-text-primary font-bold">
            {squad.name || "Unnamed Squad"}
          </h3>
          <div className="flex gap-2 text-xs text-text-secondary mt-1">
            <span>{squad.role}</span>
            <span>•</span>
            <span>{squad.status}</span>
          </div>
        </div>
        <span className="text-plasma-400 font-bold">{squad.units.length}</span>
      </div>

      {squad.target_position && (
        <div className="mb-4">
          <div className="text-sm text-text-secondary mb-2">
            →{" "}
            <span className="text-azure-400">
              ({squad.target_position[0]}, {squad.target_position[1]})
            </span>
            {squad.target_path_index != null && squad.target_path && (
              <span className="ml-2 text-amber-400">
                {squad.target_path_index}/{squad.target_path.length}
              </span>
            )}
          </div>
          {squad.target_path_index != null && squad.target_path && (
            <div className="flex items-center gap-2">
              <div className="flex-1 h-1.5 bg-background-elevated rounded-full overflow-hidden border border-border-accent">
                <div
                  className="h-full bg-amber-400 transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <span className="text-amber-400 text-xs font-medium">
                {progress}%
              </span>
            </div>
          )}
        </div>
      )}

      {squad.units.length > 0 && (
        <div className="flex gap-1 flex-wrap">
          {squad.units.map((unit) => (
            <UnitInfo key={unit.unit_id} unit={unit} />
          ))}
        </div>
      )}
    </div>
  );
};
