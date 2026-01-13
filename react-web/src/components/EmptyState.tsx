import React from 'react';

export const EmptyState: React.FC<{
  message: string;
}> = ({ message }) => {
  return (
    <div className="text-center py-10 text-slate-400 italic bg-void-950 border border-plasma-800 rounded">
      {message}
    </div>
  );
};
