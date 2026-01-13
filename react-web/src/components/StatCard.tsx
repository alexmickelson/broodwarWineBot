import React from 'react';

export const StatCard: React.FC<{
  label: string;
  value: number | string;
}> = ({ label, value }) => {
  return (
    <div className="bg-bg-secondary border border-border-primary rounded p-4 text-center">
      <div className="text-2xl font-bold text-text-accent">{value}</div>
      <div className="text-sm text-text-muted">{label}</div>
    </div>
  );
};
