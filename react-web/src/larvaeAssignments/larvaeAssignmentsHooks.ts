import { useQuery } from '@tanstack/react-query';
import * as api from './larvaeAssignmentsService';
import { usePollInterval } from '../contexts/PollIntervalContext';

// Query keys
export const queryKeys = {
  larvae: ['larvae'] as const,
};

export function useLarvae() {
  const { pollInterval } = usePollInterval();
  
  return useQuery({
    queryKey: queryKeys.larvae,
    queryFn: api.fetchLarvae,
    refetchInterval: pollInterval,
  });
}
