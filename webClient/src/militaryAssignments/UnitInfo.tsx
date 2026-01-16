import React from "react";
import type { MilitaryUnitInfo } from "./militaryAssignmentsService";

export const UnitInfo: React.FC<{ unit: MilitaryUnitInfo }> = ({ unit }) => {
  return (
    <div className="group relative p-3 bg-linear-to-br from-background-elevated to-background-elevated/50 rounded-lg border border-border-accent hover:border-plasma-500/50 transition-all duration-200">
      <div className="flex justify-between items-start mb-3">
        <div className="flex items-center gap-2">
          <div className="h-8 w-8 rounded bg-plasma-500/20 border border-plasma-500/40 flex items-center justify-center">
            <span className="text-plasma-500 font-bold text-xs">U</span>
          </div>
          <div className="flex flex-col">
            <span className="text-text-primary font-semibold text-sm">
              {unit.unitType}
            </span>
            <span className="text-text-muted text-xs">ID: {unit.unit_id}</span>
          </div>
        </div>
      </div>

      <div className="flex flex-col gap-2 text-xs">
        <div className="flex items-center gap-2 text-text-secondary">
          <svg
            className="w-3 h-3 text-azure-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
            />
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
          <span className="text-azure-400 font-medium">
            {unit.current_position[0]}, {unit.current_position[1]}
          </span>
        </div>

        <div className="text-azure-400 font-mono bg-azure-400/10 border border-azure-400/30 px-2 py-1 rounded text-xs font-medium">
          {unit.order}
        </div>

        {unit.order_target_position && (
          <div className="flex items-center gap-2 pt-1 border-t border-border-accent/50">
            <svg
              className="w-3 h-3 text-amber-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span className="text-text-secondary">Target:</span>
            <span className="text-amber-400 font-medium">
              {unit.order_target_position[0]}, {unit.order_target_position[1]}
            </span>
          </div>
        )}
      </div>
    </div>
  );
};
