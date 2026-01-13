import { useQuery } from '@tanstack/react-query';
import * as api from './service';
import { usePollInterval } from '../contexts/PollIntervalContext';

// Query keys
export const queryKeys = {
  workerStatus: ['workerStatus'] as const,
};

export function useWorkerStatus() {
  const { pollInterval } = usePollInterval();
  
  return useQuery({
    queryKey: queryKeys.workerStatus,
    queryFn: api.fetchWorkerStatus,
    refetchInterval: pollInterval,
  });
}
