import React, { useState, useEffect } from 'react';
import { SimulationProvider } from '@/model/simulationContext';
import { SimulationControls } from '@/components/ui/SimulationControls';
import { ProgressTracker } from '@/components/ui/ProgressTracker';
import { SimulationResults } from '@/components/ui/SimulationResults';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';

const SimulationPage: React.FC = () => {
  const [isInitialLoad, setIsInitialLoad] = useState(true);

  useEffect(() => {
    // Simulate initial loading
    const timer = setTimeout(() => {
      setIsInitialLoad(false);
    }, 1000);
    return () => clearTimeout(timer);
  }, []);

  if (isInitialLoad) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <LoadingSkeleton className="w-64" />
      </div>
    );
  }

  return (
    <SimulationProvider>
      <div className="container mx-auto p-6">
        <h1 className="text-3xl font-bold mb-8">Battlesim2 - Simulation Dashboard</h1>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <SimulationControls />
          <ProgressTracker />
        </div>
        
        <div className="mt-6">
          <SimulationResults />
        </div>
      </div>
    </SimulationProvider>
  );
};

export default SimulationPage;