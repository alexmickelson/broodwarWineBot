import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as api from './gameSpeedService';
import { usePollInterval } from '../contexts/PollIntervalContext';

// Query keys
export const queryKeys = {
  gameSpeed: ['gameSpeed'] as const,
};

export function useGameSpeed() {
  const { pollInterval } = usePollInterval();
  
  return useQuery({
    queryKey: queryKeys.gameSpeed,
    queryFn: api.fetchGameSpeed,
    refetchInterval: pollInterval,
  });
}

export function useSetGameSpeed() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (speed: number) => api.setGameSpeed(speed),
    onSuccess: () => {
      // Invalidate game speed query to refetch
      queryClient.invalidateQueries({ queryKey: queryKeys.gameSpeed });
    },
  });
}
