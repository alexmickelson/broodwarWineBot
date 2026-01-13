import React from 'react';
import { StatusDot } from './StatusDot';

export const ConnectionStatus: React.FC<{
  connected: boolean;
  text: string;
}> = ({ connected, text }) => {
  return (
    <div className="text-center my-5">
      <StatusDot connected={connected} />
      <span>{text}</span>
    </div>
  );
};
