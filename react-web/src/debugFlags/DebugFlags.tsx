import React from "react";
import { useDebugFlags, useUpdateDebugFlags } from "./debugFlagsHooks";
import type { DebugFlag } from "./debugFlagsService";
import { ExpandableSection } from "../components/ExpandableSection";
import { LoadingState } from "../components/LoadingState";
import { EmptyState } from "../components/EmptyState";

const ALL_FLAGS: DebugFlag[] = [
  "ShowWorkerAssignments",
  "ShowMilitaryAssignments",
  "ShowPathToEnemyBase",
  "ShowRegions",
];

const FLAG_LABELS: Record<DebugFlag, string> = {
  ShowWorkerAssignments: "Show Worker Assignments",
  ShowMilitaryAssignments: "Show Military Assignments",
  ShowPathToEnemyBase: "Show Path to Enemy Base",
  ShowRegions: "Show Regions",
};

export const DebugFlags: React.FC = () => {
  const { data, isLoading, error } = useDebugFlags();
  const updateMutation = useUpdateDebugFlags();

  const handleToggle = (flag: DebugFlag) => {
    if (!data) return;

    const currentFlags = data.debug_flags || [];
    const newFlags = currentFlags.includes(flag)
      ? currentFlags.filter((f) => f !== flag)
      : [...currentFlags, flag];

    updateMutation.mutate(newFlags);
  };

  const renderContent = () => {
    if (isLoading) {
      return <LoadingState message="Loading debug flags..." />;
    }

    if (error) {
      return <EmptyState message={`Error: ${error.message}`} />;
    }

    if (!data) {
      return <EmptyState message="No debug flags data" />;
    }

    const activeFlags = data.debug_flags || [];

    return (
      <div className="bg-bg-secondary border border-border-primary rounded p-5">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {ALL_FLAGS.map((flag) => {
            const isActive = activeFlags.includes(flag);

            return (
              <div
                key={flag}
                onClick={() => !updateMutation.isPending && handleToggle(flag)}
                className={`flex items-center justify-between p-3 rounded border border-border-primary hover:border-text-accent transition-colors ${
                  updateMutation.isPending
                    ? "opacity-50 cursor-not-allowed"
                    : "cursor-pointer"
                }`}
              >
                <span
                  className={`text-sm ${
                    isActive
                      ? "text-text-accent font-medium"
                      : "text-text-primary"
                  }`}
                >
                  {FLAG_LABELS[flag]}
                </span>
                <div
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    isActive ? "bg-text-accent" : "bg-border-primary"
                  }`}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                      isActive ? "translate-x-6" : "translate-x-1"
                    }`}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  return (
    <ExpandableSection title="Debug Flags" defaultExpanded={false}>
      {renderContent()}
    </ExpandableSection>
  );
};
