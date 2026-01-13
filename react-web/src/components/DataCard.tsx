import React from 'react';

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
