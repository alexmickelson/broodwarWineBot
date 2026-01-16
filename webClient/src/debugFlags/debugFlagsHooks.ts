import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as api from './debugFlagsService';
import type { DebugFlag } from './debugFlagsService';
import { usePollInterval } from '../contexts/PollIntervalContext';

export const queryKeys = {
  debugFlags: ['debugFlags'] as const,
};

export const useDebugFlags = () => {
  const { pollInterval } = usePollInterval();
  
  return useQuery({
    queryKey: queryKeys.debugFlags,
    queryFn: api.fetchDebugFlags,
    refetchInterval: pollInterval,
  });
};

export const useUpdateDebugFlags = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (flags: DebugFlag[]) => api.updateDebugFlags(flags),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.debugFlags });
    },
  });
};
