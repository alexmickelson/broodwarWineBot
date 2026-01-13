import { useQuery } from '@tanstack/react-query';
import * as api from './service';
import { usePollInterval } from '../contexts/PollIntervalContext';

// Query keys
export const queryKeys = {
  map: ['map'] as const,
};

export function useMap() {
  const { pollInterval } = usePollInterval();
  
  return useQuery({
    queryKey: queryKeys.map,
    queryFn: api.fetchMap,
    refetchInterval: pollInterval,
  });
}
