import React from 'react';

export const DataCard: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className = '' }) => {
  return (
    <div className={`bg-void-950 border border-plasma-800 rounded p-4 ${className}`}>
      {children}
    </div>
  );
};
