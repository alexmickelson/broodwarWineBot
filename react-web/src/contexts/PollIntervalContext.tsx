import React, { createContext, useContext, useState, ReactNode } from 'react';

interface PollIntervalContextType {
  pollInterval: number;
  setPollInterval: (interval: number) => void;
}

const PollIntervalContext = createContext<PollIntervalContextType | undefined>(undefined);

export const PollIntervalProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [pollInterval, setPollInterval] = useState(1000); // Default 1 second

  return (
    <PollIntervalContext.Provider value={{ pollInterval, setPollInterval }}>
      {children}
    </PollIntervalContext.Provider>
  );
};

export const usePollInterval = () => {
  const context = useContext(PollIntervalContext);
  if (context === undefined) {
    throw new Error('usePollInterval must be used within a PollIntervalProvider');
  }
  return context;
};
