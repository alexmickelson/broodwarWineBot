import { useQuery } from '@tanstack/react-query';
import * as api from './militaryAssignmentsService';
import { usePollInterval } from '../contexts/PollIntervalContext';

// Query keys
export const queryKeys = {
  militaryAssignments: ['militaryAssignments'] as const,
};

export function useMilitaryAssignments() {
  const { pollInterval } = usePollInterval();
  
  return useQuery({
    queryKey: queryKeys.militaryAssignments,
    queryFn: api.fetchMilitaryAssignments,
    refetchInterval: pollInterval,
  });
}
