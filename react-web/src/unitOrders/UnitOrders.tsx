import React from "react";
import { useUnitOrders } from "./unitOrdersHooks";
import { ExpandableSection } from "../components/ExpandableSection";
import { LoadingState } from "../components/LoadingState";
import { EmptyState } from "../components/EmptyState";
import { DataCard } from "../components/DataCard";
import type { UnitOrder } from "./unitOrdersService";

const UnitOrderCard: React.FC<{ order: UnitOrder }> = ({ order }) => {
  return (
    <DataCard>
      <div className="flex justify-between items-center mb-3 pb-2 border-b border-border-accent">
        <span className="text-plasma-500 font-bold bg-plasma-500/15 px-2 py-1 rounded">
          {order.unit_type}
        </span>
        <span className="text-text-muted">#{order.unit_id}</span>
      </div>

      <div className="flex flex-col gap-2 text-sm">
        <div className="text-azure-400 font-mono bg-azure-400/15 px-1.5 py-0.5 rounded inline-block">
          {order.order_name}
        </div>
        <div>
          <span className="text-text-secondary">Position: </span>
          <span className="text-azure-400 font-medium">
            ({order.current_position[0]}, {order.current_position[1]})
          </span>
        </div>

        {order.target_id != null && (
          <div>
            <span className="text-text-secondary">Target ID: </span>
            <span className="text-amber-400 font-medium">
              {order.target_id}
            </span>
          </div>
        )}

        {order.target_type && (
          <div className="text-amber-500 bg-amber-500/15 px-1.5 py-0.5 rounded inline-block">
            {order.target_type}
          </div>
        )}

        {order.target_position && (
          <div>
            <span className="text-text-secondary">Target: </span>
            <span className="text-azure-400 font-medium">
              ({order.target_position[0]}, {order.target_position[1]})
            </span>
          </div>
        )}
      </div>
    </DataCard>
  );
};

export const UnitOrders: React.FC = () => {
  const { data: orders, isLoading, error } = useUnitOrders();

  const renderContent = () => {
    if (isLoading) {
      return <LoadingState message="Waiting for unit order data..." />;
    }

    if (error) {
      return <EmptyState message={`Error: ${error.message}`} />;
    }

    if (!orders || Object.keys(orders.unit_orders).length === 0) {
      return <EmptyState message="No unit orders" />;
    }

    const orderList = Object.values(orders.unit_orders);

    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {orderList.map((order) => (
          <UnitOrderCard key={order.unit_id} order={order} />
        ))}
      </div>
    );
  };

  return (
    <ExpandableSection title="Unit Orders" defaultExpanded={false}>
      {renderContent()}
    </ExpandableSection>
  );
};
