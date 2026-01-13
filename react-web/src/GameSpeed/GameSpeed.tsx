import { useSetGameSpeed } from './hooks';
import { usePollInterval } from '../contexts/PollIntervalContext';

export function GameSpeed() {
  const setGameSpeedMutation = useSetGameSpeed();
  const { setPollInterval } = usePollInterval();

  const gameSpeeds = [
    { value: -1, label: '-1 (Default)' },
    { value: 0, label: '0 (How fast is your computer?)' },
    { value: 1, label: '1 (Fast)' },
    { value: 42, label: '42 (Standard)' },
  ];

  const pollSpeeds = [
    { value: 100, label: '100ms (Fast)' },
    { value: 250, label: '250ms' },
    { value: 500, label: '500ms (Default)' },
    { value: 1000, label: '1s (Slow)' },
    { value: 2000, label: '2s' },
  ];

  const handleGameSpeedClick = (speed: number) => {
    setGameSpeedMutation.mutate(speed);
  };

  const handlePollSpeedClick = (interval: number) => {
    setPollInterval(interval);
  };

  return (
    <div className="shrink-0 p-5 bg-bg-primary shadow-lg">
      <div className="flex gap-10 items-start">
        <div className="flex-1 min-w-0">
          <h2 className="text-text-secondary mt-0 mb-2.5">Game Speed</h2>
          <div className="bg-bg-secondary border border-border-primary rounded p-5 my-5 flex flex-col gap-4 items-center">
            <div className="flex gap-2.5 flex-wrap justify-center">
              {gameSpeeds.map((speed) => (
                <button
                  key={speed.value}
                  onClick={() => handleGameSpeedClick(speed.value)}
                  className="px-4 py-2 bg-bg-secondary border border-border-primary rounded text-text-primary hover:border-text-accent hover:text-text-accent transition-colors cursor-pointer"
                >
                  {speed.label}
                </button>
              ))}
            </div>
          </div>
        </div>
        <div className="flex-1 min-w-0">
          <h2 className="text-text-secondary mt-0 mb-2.5">Poll Speed</h2>
          <div className="bg-bg-secondary border border-border-primary rounded p-5 my-5 flex flex-col gap-4 items-center">
            <div className="flex gap-2.5 flex-wrap justify-center">
              {pollSpeeds.map((speed) => (
                <button
                  key={speed.value}
                  onClick={() => handlePollSpeedClick(speed.value)}
                  className="px-4 py-2 bg-bg-secondary border border-border-primary rounded text-text-primary hover:border-text-accent hover:text-text-accent transition-colors cursor-pointer"
                >
                  {speed.label}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
