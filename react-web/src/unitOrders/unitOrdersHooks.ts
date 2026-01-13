import { useQuery } from '@tanstack/react-query';
import * as api from './unitOrdersService';
import { usePollInterval } from '../contexts/PollIntervalContext';

// Query keys
export const queryKeys = {
  unitOrders: ['unitOrders'] as const,
};

export function useUnitOrders() {
  const { pollInterval } = usePollInterval();
  
  return useQuery({
    queryKey: queryKeys.unitOrders,
    queryFn: api.fetchUnitOrders,
    refetchInterval: pollInterval,
  });
}
