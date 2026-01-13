import React from 'react';

// Reusable UI components with Tailwind styling

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

export const ExpandableSection: React.FC<{
  title: string;
  children: React.ReactNode;
  defaultExpanded?: boolean;
}> = ({ title, children, defaultExpanded = true }) => {
  const [isExpanded, setIsExpanded] = React.useState(defaultExpanded);

  return (
    <div className="my-5">
      <h2 className="text-text-secondary py-2 my-2.5 cursor-pointer flex items-center gap-2"
          onClick={() => setIsExpanded(!isExpanded)}>
        <span className="transition-transform duration-200" style={{ transform: isExpanded ? 'rotate(0deg)' : 'rotate(-90deg)' }}>
          ▼
        </span>
        {title}
      </h2>
      {isExpanded && (
        <div className="mt-5">
          {children}
        </div>
      )}
    </div>
  );
};

export const LoadingState: React.FC<{
  message?: string;
}> = ({ message = 'Loading...' }) => {
  return (
    <div className="opacity-70 text-center py-10">
      {message}
    </div>
  );
};

export const EmptyState: React.FC<{
  message: string;
}> = ({ message }) => {
  return (
    <div className="text-center py-10 text-text-muted italic bg-bg-secondary border border-border-primary rounded">
      {message}
    </div>
  );
};

export const DataCard: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className = '' }) => {
  return (
    <div className={`bg-bg-secondary border border-border-primary rounded p-4 ${className}`}>
      {children}
    </div>
  );
};

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
