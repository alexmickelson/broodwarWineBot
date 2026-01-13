import { useUnitOrders } from './hooks';
import { ExpandableSection, LoadingState, EmptyState, DataField, DataCard } from '../components/Common';
import type { UnitOrder } from './service';

function UnitOrderCard({ order }: { order: UnitOrder }) {
  return (
    <DataCard>
      <div className="flex justify-between items-center mb-3 pb-2 border-b border-border-accent">
        <span className="text-text-accent font-bold bg-text-accent/15 px-2 py-1 rounded">
          {order.unit_type}
        </span>
        <span className="text-text-muted">#{order.unit_id}</span>
      </div>
      
      <div className="flex flex-col gap-2">
        <DataField label="order" value={order.order_name} type="order" />
        <DataField 
          label="position" 
          value={`(${order.current_position[0]}, ${order.current_position[1]})`}
          type="tuple"
        />
        
        {order.target_id != null && (
          <DataField label="target_id" value={order.target_id} type="number" />
        )}
        
        {order.target_type && (
          <DataField label="target_type" value={order.target_type} type="target" />
        )}
        
        {order.target_position && (
          <DataField 
            label="target_position" 
            value={`(${order.target_position[0]}, ${order.target_position[1]})`}
            type="tuple"
          />
        )}
      </div>
    </DataCard>
  );
}

export function UnitOrders() {
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
}
