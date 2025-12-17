import React from 'react';
import { useSimulation } from '@/model/simulationContext';

export const SimulationResults: React.FC = () => {
  const { state } = useSimulation();

  if (state.isRunning) {
    return (
      <div className="animate-pulse h-64">
        <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
        <div className="h-4 bg-gray-200 rounded w-full mb-2"></div>
        <div className="h-4 bg-gray-200 rounded w-5/6"></div>
      </div>
    );
  }

  if (state.error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <p className="text-red-800">Error: {state.error}</p>
      </div>
    );
  }

  if (!state.results || state.results.length === 0) {
    return (
      <div className="bg-gray-50 rounded-lg p-8 text-center">
        <p className="text-gray-500">No results to display</p>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-md p-6 w-full">
      <h2 className="text-xl font-semibold mb-4">Simulation Results</h2>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b">
              <th className="text-left py-2 px-4 font-medium">Name</th>
              <th className="text-left py-2 px-4 font-medium">Value</th>
              <th className="text-left py-2 px-4 font-medium">Percentage</th>
            </tr>
          </thead>
          <tbody>
            {state.results.map((result) => (
              <tr key={result.id} className="border-b">
                <td className="py-2 px-4">{result.name}</td>
                <td className="py-2 px-4">{result.value}</td>
                <td className="py-2 px-4">{result.percentage}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

const LoadingSkeleton: React.FC<{ className?: string }> = ({ className }) => {
  return (
    <div className={`animate-pulse ${className || ''}`}>
      <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
      <div className="h-4 bg-gray-200 rounded w-full mb-2"></div>
      <div className="h-4 bg-gray-200 rounded w-5/6"></div>
    </div>
  );
};