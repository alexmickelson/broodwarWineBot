import React, { useState } from "react";
import { UnitInfo } from "./UnitInfo";
import type { SquadData } from "./militaryAssignmentsService";
import { updateSquadTargetPercentage } from "./militaryAssignmentsService";

export const SquadCard: React.FC<{ squad: SquadData }> = ({ squad }) => {
  const [localPercentage, setLocalPercentage] = useState(
    squad.target_percentage,
  );

  const progress =
    squad.target_path && squad.target_path_index
      ? Math.round((squad.target_path_index / squad.target_path.length) * 100)
      : 0;

  const handleSliderChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = parseFloat(e.target.value);
    setLocalPercentage(newValue);

    try {
      await updateSquadTargetPercentage(squad.name, newValue);
    } catch (error) {
      console.error("Failed to update squad target percentage:", error);
    }
  };

  const handleBarClick = async (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const newValue = Math.max(0, Math.min(1, x / rect.width));
    setLocalPercentage(newValue);

    try {
      await updateSquadTargetPercentage(squad.name, newValue);
    } catch (error) {
      console.error("Failed to update squad target percentage:", error);
    }
  };

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

      {/* Target Percentage Slider */}
      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <label className="text-sm text-text-secondary">Target Position</label>
          <span className="text-azure-400 text-sm font-medium">
            {Math.round(localPercentage * 100)}%
          </span>
        </div>
        <div className="flex items-center gap-2">
          <div
            className="flex-1 relative cursor-pointer py-2"
            onClick={handleBarClick}
          >
            <div className="h-1.5 bg-background-elevated rounded-full border border-border-accent">
              {/* Squad's actual percentage from server */}
              <div
                className="h-full bg-text-secondary/30 transition-all duration-300 rounded-full pointer-events-none absolute inset-0"
                style={{ width: `${squad.target_percentage * 100}%` }}
              />
              {/* User's local percentage */}
              <div
                className="h-full bg-azure-400 transition-all duration-150 rounded-full pointer-events-none relative"
                style={{ width: `${localPercentage * 100}%` }}
              />
            </div>
            {/* Draggable button/handle */}
            <div
              className="absolute top-1/2 -translate-y-1/2 w-4 h-4 bg-azure-400 border-2 border-white rounded-full shadow-lg pointer-events-none transition-all duration-150"
              style={{ left: `calc(${localPercentage * 100}% - 0.5rem)` }}
            />
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={localPercentage}
              onChange={handleSliderChange}
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
            />
          </div>
        </div>
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
