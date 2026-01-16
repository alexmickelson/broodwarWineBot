import React from 'react';

export const StatCard: React.FC<{
  label: string;
  value: number | string;
}> = ({ label, value }) => {
  return (
    <div className="bg-void-950 border border-plasma-800 rounded p-4 text-center">
      <div className="text-2xl font-bold text-plasma-500">{value}</div>
      <div className="text-sm text-slate-400">{label}</div>
    </div>
  );
};
