import React from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faSpinner } from '@fortawesome/free-solid-svg-icons';
import { useSimulation } from '@/model/simulationContext';

export const ProgressTracker: React.FC = () => {
  const { state } = useSimulation();

  return (
    <div className="bg-white rounded-lg shadow-md p-6 max-w-md w-full">
      <h2 className="text-xl font-semibold mb-4">Progress</h2>
      <div className="mb-4">
        <div className="flex justify-between text-sm text-gray-600 mb-2">
          <span>Progress</span>
          <span>{state.progress}%</span>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-3">
          <div
            className="bg-blue-500 h-3 rounded-full transition-all duration-300"
            style={{ width: `${state.progress}%` }}
          ></div>
        </div>
      </div>
      {state.isRunning && (
        <div className="flex items-center gap-2 text-gray-600">
          <FontAwesomeIcon icon={faSpinner} spin className="text-blue-500" />
          <span>
            {state.estimatedTime ? `Estimated time remaining: ${state.estimatedTime}` : 'Processing...'}
          </span>
        </div>
      )}
      {!state.isRunning && state.progress === 0 && (
        <p className="text-gray-500">Click "Run Simulation" to start</p>
      )}
      {!state.isRunning && state.progress > 0 && state.progress < 100 && (
        <p className="text-gray-500">Simulation completed</p>
      )}
      {state.progress === 100 && (
        <p className="text-green-600 font-medium">Simulation complete!</p>
      )}
    </div>
  );
};