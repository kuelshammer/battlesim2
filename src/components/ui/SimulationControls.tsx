import React from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faPlay, faRedo } from '@fortawesome/free-solid-svg-icons';
import { useSimulation } from '@/model/simulationContext';

export const SimulationControls: React.FC = () => {
  const { state, startSimulation, reset } = useSimulation();

  return (
    <div className="bg-white rounded-lg shadow-md p-6 max-w-md w-full">
      <h2 className="text-xl font-semibold mb-4">Simulation Controls</h2>
      <div className="flex flex-col gap-4">
        <button
          onClick={startSimulation}
          disabled={state.isRunning}
          className="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          <FontAwesomeIcon icon={faPlay} />
          {state.isRunning ? 'Running...' : 'Run Simulation'}
        </button>
        {state.results && (
          <button
            onClick={reset}
            className="bg-gray-200 hover:bg-gray-300 text-gray-800 font-medium py-2 px-4 rounded flex items-center justify-center gap-2"
          >
            <FontAwesomeIcon icon={faRedo} />
            Reset Results
          </button>
        )}
      </div>
    </div>
  );
};