import { useQuery } from '@tanstack/react-query';
import * as api from './buildOrderService';
import { usePollInterval } from '../contexts/PollIntervalContext';

// Query keys
export const queryKeys = {
  buildOrder: ['buildOrder'] as const,
};

export function useBuildOrder() {
  const { pollInterval } = usePollInterval();
  
  return useQuery({
    queryKey: queryKeys.buildOrder,
    queryFn: api.fetchBuildOrder,
    refetchInterval: pollInterval,
  });
}
