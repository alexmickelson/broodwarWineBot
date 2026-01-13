import { useBuildOrder } from "./hooks";
import { ExpandableSection, LoadingState, EmptyState } from "../components/Common";
import type { BuildOrderSnapshot } from "./service";

export function BuildOrder() {
  const { data, isLoading, error } = useBuildOrder();

  const renderContent = () => {
    if (isLoading) {
      return <LoadingState message="Waiting for build order..." />;
    }

    if (error) {
      return <EmptyState message={`Error: ${error.message}`} />;
    }

    const buildOrderData = data as BuildOrderSnapshot;

    if (
      !buildOrderData ||
      !buildOrderData.build_order ||
      buildOrderData.build_order.length === 0
    ) {
      return <EmptyState message="No build order set" />;
    }

    const { build_order, build_order_index } = buildOrderData;

    return (
      <div className="flex flex-col gap-2">
        {build_order.map((unit, index) => {
          const isComplete = index < build_order_index;
          const isCurrent = index === build_order_index;

          // Skip completed items
          if (isComplete) return null;

          const displayName = unit.replace(/^(Terran|Protoss|Zerg)_/, "");

          return (
            <div
              key={index}
              className={`flex items-center gap-3 p-3 rounded border ${
                isCurrent
                  ? "bg-text-accent/20 border-text-accent"
                  : "bg-bg-secondary border-border-primary"
              }`}
            >
              <span className="text-text-muted font-bold min-w-8">
                {index + 1}
              </span>
              <span
                className={`font-medium ${
                  isCurrent ? "text-text-accent" : "text-text-primary"
                }`}
              >
                {displayName}
              </span>
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <ExpandableSection title="Build Order" defaultExpanded={false}>
      {renderContent()}
    </ExpandableSection>
  );
}
