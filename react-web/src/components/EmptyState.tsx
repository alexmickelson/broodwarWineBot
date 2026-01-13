import React from 'react';

export const EmptyState: React.FC<{
  message: string;
}> = ({ message }) => {
  return (
    <div className="text-center py-10 text-text-muted italic bg-bg-secondary border border-border-primary rounded">
      {message}
    </div>
  );
};
