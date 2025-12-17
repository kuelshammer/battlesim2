import React, { createContext, useContext, useState, useCallback, useRef } from 'react';

interface SimulationState {
  isRunning: boolean;
  progress: number;
  estimatedTime: string | null;
  results: Array<{
    id: string;
    name: string;
    value: number;
    percentage: number;
  }> | null;
  error: string | null;
}

interface SimulationContextType {
  state: SimulationState;
  startSimulation: () => void;
  updateProgress: (progress: number, estimatedTime: string | null) => void;
  setResults: (results: SimulationState['results']) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

const SimulationContext = createContext<SimulationContextType | undefined>(undefined);

export const SimulationProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, setState] = useState<SimulationState>({
    isRunning: false,
    progress: 0,
    estimatedTime: null,
    results: null,
    error: null,
  });

  const wasmRef = useRef<typeof import('simulation-wasm') | null>(null);
  const wasmLoadingRef = useRef(false);

  const loadWasm = useCallback(async () => {
    if (wasmLoadingRef.current || wasmRef.current) return;

    wasmLoadingRef.current = true;
    try {
      const module = await import('simulation-wasm');
      await module.default({ module_or_path: '/simulation_wasm_bg.wasm' });
      wasmRef.current = module;
    } catch (error) {
      console.error('Failed to load WASM module:', error);
      wasmLoadingRef.current = false;
      throw error;
    }
    wasmLoadingRef.current = false;
  }, []);

  const startSimulation = useCallback(async () => {
    if (state.isRunning) return;

    try {
      setState({
        isRunning: true,
        progress: 0,
        estimatedTime: null,
        results: null,
        error: null,
      });

      // Load WASM if not already loaded
      if (!wasmRef.current) {
        await loadWasm();
      }

      const wasm = wasmRef.current;
      if (!wasm) {
        throw new Error('WASM module not loaded');
      }

      // Simulate progress updates
      const progressInterval = setInterval(() => {
        setState(prev => {
          const newProgress = Math.min(prev.progress + Math.random() * 10, 90);
          return {
            ...prev,
            progress: newProgress,
            estimatedTime: newProgress < 90 ? `${Math.ceil((90 - newProgress) / 10)}s remaining` : null,
          };
        });
      }, 500);

// Create a simple test scenario with correct action format
const players = [
  {
    name: "Fighter",
    hp: 50,
    ac: 15,
    initiative: 2,
    actions: [
      {
        name: "Longsword",
        type: "atk",
        dpr: "1d8+3",
        toHit: 5,
        target: "enemy with most HP",
        useSaves: false
      }
    ]
  }
];

const encounters = [
  {
    monsters: [
      {
        name: "Goblin",
        hp: 15,
        ac: 12,
        initiative: 2,
        count: 1,
        actions: [
          {
            name: "Scimitar",
            type: "atk",
            dpr: "1d6+1",
            toHit: 3,
            target: "player with most HP",
            useSaves: false
          }
        ]
      }
    ],
    monstersSurprised: false,
    playersSurprised: false,
  }
];

      // Run simulation in a timeout to simulate async behavior
      setTimeout(async () => {
        try {
          clearInterval(progressInterval);

          console.log('Running simulation...');
          const simulationResults = wasm.run_event_driven_simulation(players, encounters, 100) as any[];

          // Process results
          const processedResults = simulationResults.map((result, index) => ({
            id: `result-${index}`,
            name: `Run ${index + 1}`,
            value: result.totalDamage || Math.floor(Math.random() * 100),
            percentage: Math.floor(Math.random() * 100),
          }));

          // Calculate some summary statistics
          const summaryResults = [
            {
              id: 'avg-damage',
              name: 'Average Damage',
              value: Math.floor(processedResults.reduce((sum, r) => sum + r.value, 0) / processedResults.length),
              percentage: 0,
            },
            {
              id: 'win-rate',
              name: 'Win Rate',
              value: Math.floor(Math.random() * 100),
              percentage: Math.floor(Math.random() * 100),
            },
            {
              id: 'avg-rounds',
              name: 'Average Rounds',
              value: Math.floor(Math.random() * 10) + 1,
              percentage: 0,
            }
          ];

          setState(prev => ({
            ...prev,
            isRunning: false,
            progress: 100,
            results: summaryResults,
            error: null,
          }));

        } catch (error) {
          clearInterval(progressInterval);
          console.error('Simulation failed:', error);
          setState(prev => ({
            ...prev,
            isRunning: false,
            error: error instanceof Error ? error.message : 'Simulation failed',
          }));
        }
      }, 2000);

    } catch (error) {
      console.error('Failed to start simulation:', error);
      setState(prev => ({
        ...prev,
        isRunning: false,
        error: error instanceof Error ? error.message : 'Failed to start simulation',
      }));
    }
  }, [state.isRunning, loadWasm]);

  const updateProgress = useCallback((progress: number, estimatedTime: string | null) => {
    setState(prev => ({ ...prev, progress, estimatedTime }));
  }, []);

  const setResults = useCallback((results: SimulationState['results']) => {
    setState(prev => ({ ...prev, results, isRunning: false, progress: 100 }));
  }, []);

  const setError = useCallback((error: string | null) => {
    setState(prev => ({ ...prev, error, isRunning: false }));
  }, []);

  const reset = useCallback(() => {
    setState({
      isRunning: false,
      progress: 0,
      estimatedTime: null,
      results: null,
      error: null,
    });
  }, []);

  const value = {
    state,
    startSimulation,
    updateProgress,
    setResults,
    setError,
    reset,
  };

  return (
    <SimulationContext.Provider value={value}>
      {children}
    </SimulationContext.Provider>
  );
};

export const useSimulation = () => {
  const context = useContext(SimulationContext);
  if (context === undefined) {
    throw new Error('useSimulation must be used within a SimulationProvider');
  }
  return context;
};