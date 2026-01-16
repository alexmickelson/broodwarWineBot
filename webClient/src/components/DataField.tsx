import React from 'react';

export const DataField: React.FC<{
  label: string;
  value: React.ReactNode;
  type?: 'default' | 'number' | 'tuple' | 'enum' | 'order' | 'unit' | 'target';
  enumType?: 'gathering' | 'scouting' | 'building';
}> = ({ label, value, type = 'default', enumType }) => {
  const getValueClassName = () => {
    const base = 'font-medium';
    
    switch (type) {
      case 'number':
        return `${base} text-amber-400`;
      case 'tuple':
        return `${base} text-azure-400`;
      case 'enum':
        if (enumType === 'gathering') {
          return `${base} bg-emerald-500/20 text-emerald-500 px-2 py-0.5 rounded font-bold`;
        } else if (enumType === 'scouting') {
          return `${base} bg-azure-500/20 text-azure-500 px-2 py-0.5 rounded font-bold`;
        } else if (enumType === 'building') {
          return `${base} bg-amber-500/20 text-amber-500 px-2 py-0.5 rounded font-bold`;
        }
        return base;
      case 'order':
        return `${base} text-azure-400 font-mono bg-azure-400/15 px-1.5 py-0.5 rounded`;
      case 'unit':
        return `${base} text-plasma-500 font-bold bg-plasma-500/15 px-2 py-0.5 rounded`;
      case 'target':
        return `${base} text-amber-500 bg-amber-500/15 px-1.5 py-0.5 rounded`;
      default:
        return `${base} text-lavender-400`;
    }
  };

  return (
    <div className="flex gap-2 items-baseline font-mono text-sm">
      <span className="text-lavender-500 min-w-35">{label}:</span>
      <span className={getValueClassName()}>{value}</span>
    </div>
  );
};
