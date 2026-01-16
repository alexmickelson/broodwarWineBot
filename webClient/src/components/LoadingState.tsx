import React from 'react';

export const LoadingState: React.FC<{
  message?: string;
}> = ({ message = 'Loading...' }) => {
  return (
    <div className="opacity-70 text-center py-10">
      {message}
    </div>
  );
};
