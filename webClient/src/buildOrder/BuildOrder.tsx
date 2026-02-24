import React, { useRef, useEffect } from 'react';
import { useBuildOrder } from "./buildOrderHooks";
import { LoadingState } from "../components/LoadingState";
import { EmptyState } from "../components/EmptyState";
import type { BuildOrderSnapshot, BuildOrderItem } from "./buildOrderService";

function getItemDisplayName(item: BuildOrderItem): string {
  if (item.type === 'Unit') {
    const name = item.unit_type.replace(/^(Terran|Protoss|Zerg)_/, "");
    return item.base_index !== null ? `${name} @base${item.base_index}` : name;
  } else if (item.type === 'Upgrade') {
    return item.upgrade_type.replace(/^(Terran|Protoss|Zerg)_/, "");
  } else if (item.type === 'Squad') {
    return `${item.name} (${item.role})`;
  }
  return "Unknown";
}

function getItemType(item: BuildOrderItem): string {
  return item.type;
}

export const BuildOrder: React.FC = () => {
  const { data, isLoading, error } = useBuildOrder();
  const currentItemRef = useRef<HTMLDivElement>(null);

  // Extract build_order_index for useEffect dependency
  const buildOrderData = data as BuildOrderSnapshot;
  const build_order_index = buildOrderData?.build_order_index;

  // Scroll to current item when it changes
  useEffect(() => {
    if (currentItemRef.current) {
      const container = currentItemRef.current.parentElement;
      if (container) {
        const itemTop = currentItemRef.current.offsetTop;
        const itemHeight = currentItemRef.current.offsetHeight;
        const gap = 8;
        const targetScroll = itemTop - (itemHeight + gap) * 3;
        
        container.scrollTo({
          top: Math.max(0, targetScroll),
          behavior: 'smooth',
        });
      }
    }
  }, [build_order_index]);

  return (
    <div className="flex flex-col h-full">
      
      {isLoading && <LoadingState message="Waiting for build order..." />}
      
      {error && <EmptyState message={`Error: ${error.message}`} />}
      
      {!isLoading && !error && (!buildOrderData || !buildOrderData.build_order || buildOrderData.build_order.length === 0) && (
        <EmptyState message="No build order set" />
      )}
      
      {!isLoading && !error && buildOrderData?.build_order && buildOrderData.build_order.length > 0 && (
        <div className="flex flex-col gap-2 overflow-y-auto p-2">
          {buildOrderData.build_order.map((item, index) => {
            const isComplete = index < buildOrderData.build_order_index;
            const isCurrent = index === buildOrderData.build_order_index;

            const displayName = getItemDisplayName(item);
            const itemType = getItemType(item);

            return (
              <div
                key={index}
                ref={isCurrent ? currentItemRef : null}
                className={`flex items-center gap-3 p-3 rounded border ${
                  isCurrent
                    ? "bg-plasma-500/20 border-plasma-500"
                    : isComplete
                    ? "bg-void-900/50 border-void-700 opacity-60"
                    : "bg-void-950 border-plasma-800"
                }`}
              >
                <span className={`font-bold min-w-4 ${
                  isComplete ? "text-slate-600" : "text-slate-600"
                }`}>
                  {index + 1}
                </span>
                <div className="flex flex-col flex-1 min-w-0">
                  <span
                    className={`font-medium truncate ${
                      isCurrent 
                        ? "text-plasma-500" 
                        : isComplete
                        ? "text-slate-600 line-through"
                        : "text-lavender-400"
                    }`}
                  >
                    {displayName}
                  </span>
                  {itemType === 'Upgrade' && (
                    <span className="text-xs text-slate-500">Upgrade</span>
                  )}
                </div>
                {isComplete && (
                  <span className="ml-auto text-green-500 text-sm">✓</span>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
