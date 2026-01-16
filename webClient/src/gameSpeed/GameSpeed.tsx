import React from 'react';
import { useGameSpeed, useSetGameSpeed } from './gameSpeedHooks';
import { usePollInterval } from '../contexts/PollIntervalContext';

export const GameSpeed: React.FC = () => {
  const { data: gameSpeedData } = useGameSpeed();
  const setGameSpeedMutation = useSetGameSpeed();
  const { pollInterval, setPollInterval } = usePollInterval();

  const currentGameSpeed = gameSpeedData?.game_speed ?? 42;

  const gameSpeeds = [
    { value: -1, label: '-1', description: 'Default' },
    { value: 0, label: '0', description: 'Fastest' },
    { value: 1, label: '1', description: 'Fast' },
    { value: 42, label: '42', description: 'Standard' },
  ];

  const pollSpeeds = [
    { value: 100, label: '100ms', description: 'Fast' },
    { value: 250, label: '250ms', description: '' },
    { value: 500, label: '500ms', description: 'Default' },
    { value: 1000, label: '1s', description: 'Slow' },
    { value: 2000, label: '2s', description: '' },
  ];

  const handleGameSpeedClick = (speed: number) => {
    setGameSpeedMutation.mutate(speed);
  };

  const handlePollSpeedClick = (interval: number) => {
    setPollInterval(interval);
  };

  return (
    <div className="shrink-0 p-2 bg-void-950/50 rounded border border-plasma-800">
      <div className="flex gap-8 items-center justify-center">
        <div className="flex items-center gap-3">
          <span className="text-lavender-500 text-sm font-medium min-w-20">Game Speed:</span>
          <div className="flex gap-1.5">
            {gameSpeeds.map((speed) => {
              const isActive = currentGameSpeed === speed.value;
              return (
                <button
                  key={speed.value}
                  onClick={() => handleGameSpeedClick(speed.value)}
                  className={`px-3 py-1.5 text-sm rounded transition-all border font-semibold ${
                    isActive
                      ? 'bg-plasma-500 text-abyss-100 shadow-lg border-plasma-500'
                      : 'bg-abyss-100 border-plasma-800 text-lavender-400 hover:border-plasma-500 hover:text-plasma-500'
                  }`}
                >
                  <span className="font-mono">{speed.label}</span>
                  {speed.description && (
                    <span className="ml-1 text-xs opacity-70">({speed.description})</span>
                  )}
                </button>
              );
            })}
          </div>
        </div>
        <div className="w-px h-8 bg-plasma-800" />
        <div className="flex items-center gap-3">
          <span className="text-lavender-500 text-sm font-medium min-w-20">Poll Speed:</span>
          <div className="flex gap-1.5">
            {pollSpeeds.map((speed) => {
              const isActive = pollInterval === speed.value;
              return (
                <button
                  key={speed.value}
                  onClick={() => handlePollSpeedClick(speed.value)}
                  className={`px-3 py-1.5 text-sm rounded transition-all border font-semibold ${
                    isActive
                      ? 'bg-plasma-500 text-abyss-100 shadow-lg border-plasma-500'
                      : 'bg-abyss-100 border-plasma-800 text-lavender-400 hover:border-plasma-500 hover:text-plasma-500'
                  }`}
                >
                  <span className="font-mono">{speed.label}</span>
                  {speed.description && (
                    <span className="ml-1 text-xs opacity-70">({speed.description})</span>
                  )}
                </button>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
