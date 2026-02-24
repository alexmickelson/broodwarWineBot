import React from "react";
import type { SquadData } from "./militaryAssignmentsService";
import { updateSquadTargetIndex } from "./militaryAssignmentsService";

interface UnitTypeCount {
  type: string;
  count: number;
}

export const SquadCard: React.FC<{ squad: SquadData }> = ({ squad }) => {
  const [sliderValue, setSliderValue] = React.useState(
    squad.target_path_index ?? 0,
  );
  const [isUpdating, setIsUpdating] = React.useState(false);

  // Update slider when squad data changes
  React.useEffect(() => {
    if (squad.target_path_index != null) {
      setSliderValue(squad.target_path_index);
    }
  }, [squad.target_path_index]);

  const handleSliderChange = async (newValue: number) => {
    setSliderValue(newValue);
    setIsUpdating(true);
    try {
      await updateSquadTargetIndex(squad.name, newValue);
    } catch (error) {
      console.error("Failed to update squad target:", error);
    } finally {
      setIsUpdating(false);
    }
  };

  // Group units by type and count them
  const unitTypeCounts: UnitTypeCount[] = React.useMemo(() => {
    const counts = new Map<string, number>();
    squad.units.forEach((unit) => {
      const currentCount = counts.get(unit.unitType) || 0;
      counts.set(unit.unitType, currentCount + 1);
    });
    return Array.from(counts.entries())
      .map(([type, count]) => ({ type, count }))
      .sort((a, b) => b.count - a.count);
  }, [squad.units]);

  // Extract enemy names from target info
  const targetEnemies = React.useMemo(() => {
    const enemies = new Map<string, number>();
    squad.units.forEach((unit) => {
      if (unit.target_unit) {
        const currentCount = enemies.get(unit.target_unit) || 0;
        enemies.set(unit.target_unit, currentCount + 1);
      }
    });
    return Array.from(enemies.entries())
      .map(([type, count]) => ({ type, count }))
      .sort((a, b) => b.count - a.count);
  }, [squad.units]);

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

      {/* Unit Type Counts */}
      {unitTypeCounts.length > 0 && (
        <div className="mb-3">
          <h4 className="text-xs font-semibold text-text-secondary mb-2">
            Squad Composition
          </h4>
          <div className="space-y-2">
            {unitTypeCounts.map((unitType) => (
              <div
                key={unitType.type}
                className="flex items-center justify-between bg-background-elevated border border-border-accent rounded px-3 py-2"
              >
                <span className="text-text-primary text-sm">
                  {unitType.type.replace("Zerg_", "").replace("_", " ")}
                </span>
                <span className="text-azure-400 font-semibold text-lg">
                  {unitType.count}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Target Enemies */}
      {targetEnemies.length > 0 && (
        <div className="mb-3">
          <h4 className="text-xs font-semibold text-text-secondary mb-2">
            Attacking
          </h4>
          <div className="space-y-2">
            {targetEnemies.map((enemy) => (
              <div
                key={enemy.type}
                className="flex items-center justify-between bg-red-900/20 border border-red-700/50 rounded px-3 py-2"
              >
                <span className="text-red-400 text-sm">
                  {enemy.type
                    .replace("Zerg_", "")
                    .replace("Terran_", "")
                    .replace("Protoss_", "")
                    .replace("_", " ")}
                </span>
                <span className="text-red-300 font-semibold text-lg">
                  {enemy.count}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

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
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <input
                  type="range"
                  min={0}
                  max={squad.target_path.length - 1}
                  value={sliderValue}
                  onChange={(e) => handleSliderChange(Number(e.target.value))}
                  disabled={isUpdating}
                  className="flex-1 h-2 bg-background-elevated rounded-lg appearance-none cursor-pointer accent-amber-400"
                  style={{
                    background: `linear-gradient(to right, rgb(251, 191, 36) 0%, rgb(251, 191, 36) ${(sliderValue / (squad.target_path.length - 1)) * 100}%, rgb(30, 41, 59) ${(sliderValue / (squad.target_path.length - 1)) * 100}%, rgb(30, 41, 59) 100%)`,
                  }}
                />
                <span className="text-amber-400 text-xs font-medium min-w-12 text-right">
                  {Math.round(
                    (sliderValue / (squad.target_path.length - 1)) * 100,
                  )}
                  %
                </span>
              </div>
              <div className="flex justify-between text-xs text-text-secondary">
                <span>
                  {sliderValue}/{squad.target_path.length - 1}
                </span>
                {isUpdating && (
                  <span className="text-amber-400">Updating...</span>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
