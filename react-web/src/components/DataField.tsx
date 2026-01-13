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
        return `${base} text-data-number`;
      case 'tuple':
        return `${base} text-data-tuple`;
      case 'enum':
        if (enumType === 'gathering') {
          return `${base} bg-assignment-gathering/20 text-assignment-gathering px-2 py-0.5 rounded font-bold`;
        } else if (enumType === 'scouting') {
          return `${base} bg-assignment-scouting/20 text-assignment-scouting px-2 py-0.5 rounded font-bold`;
        } else if (enumType === 'building') {
          return `${base} bg-assignment-building/20 text-assignment-building px-2 py-0.5 rounded font-bold`;
        }
        return base;
      case 'order':
        return `${base} text-data-tuple font-mono bg-data-tuple/15 px-1.5 py-0.5 rounded`;
      case 'unit':
        return `${base} text-text-accent font-bold bg-text-accent/15 px-2 py-0.5 rounded`;
      case 'target':
        return `${base} text-assignment-building bg-assignment-building/15 px-1.5 py-0.5 rounded`;
      default:
        return `${base} text-text-primary`;
    }
  };

  return (
    <div className="flex gap-2 items-baseline font-mono text-sm">
      <span className="text-text-secondary min-w-35">{label}:</span>
      <span className={getValueClassName()}>{value}</span>
    </div>
  );
};
