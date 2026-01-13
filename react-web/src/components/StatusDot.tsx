import React from 'react';

export const StatusDot: React.FC<{
  connected: boolean;
}> = ({ connected }) => {
  return (
    <span
      className={`inline-block w-2.5 h-2.5 rounded-full mr-2 ${
        connected
          ? 'bg-status-success shadow-success'
          : 'bg-status-error'
      }`}
    />
  );
};
