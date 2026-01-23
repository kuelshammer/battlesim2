import init, { ChunkedSimulationRunner, auto_adjust_encounter_wasm } from 'simulation-wasm';
import wasmUrl from 'simulation-wasm/simulation_wasm_bg.wasm';

export type SimulationResultType = 'completed' | 'cancelled' | 'errored';

export interface StructuredSimulationResult {
  type: SimulationResultType;
  genId: number;
  results?: any[];
  analysis?: any;
  events?: any[];
  kFactor?: number;
  isFinal?: boolean;
  error?: string;
  result?: any;
}

export class SimulationWorkerController {
  private worker: Worker | null = null;
  private abortController: AbortController | null = null;
  private currentGenId = 0;
  private activeRunner: ChunkedSimulationRunner | null = null;
  private wasmInitialized = false;

  // Instrumentation for testing worker lifecycle
  private static instanceCount = 0;
  private instanceId: number;

  constructor(worker?: Worker) {
    this.instanceId = ++SimulationWorkerController.instanceCount;

    if (worker) {
      this.worker = worker;
      this.worker.onmessage = this.handleMessage.bind(this);
    } else {
      this.initializeWorker();
    }
  }

  private async initializeWorker() {
    if (this.worker) {
      this.worker.terminate();
    }

    this.worker = new Worker(new URL('./simulation.worker.ts', import.meta.url));
    this.worker.onmessage = this.handleMessage.bind(this);
  }

  private async ensureWasmInitialized() {
    if (!this.wasmInitialized) {
      await init({ module_or_path: wasmUrl });
      this.wasmInitialized = true;
    }
  }

  private handleMessage(e: MessageEvent<StructuredSimulationResult>) {
    // The worker will send structured results that can be handled externally
    // We'll let the controller consumer handle the message
  }

  public async startSimulation(
    players: any[],
    timeline: any[],
    genId: number,
    maxK: number = 51,
    seed?: number,
    onResult?: (result: StructuredSimulationResult) => void
  ) {
    this.currentGenId = genId;
    this.abortController = new AbortController();

    try {
      await this.ensureWasmInitialized();

      // Initialize runner
      this.activeRunner = new ChunkedSimulationRunner(players, timeline, seed ? BigInt(seed) : null);

      // Initial pass
      this.activeRunner.run_chunk(100);
      const output = this.activeRunner.get_analysis(1);
      const { results, analysis, firstRunEvents } = output;

      const isFinal = maxK <= 1;
      const result: StructuredSimulationResult = {
        type: 'completed',
        genId,
        results,
        analysis,
        events: firstRunEvents,
        kFactor: 1,
        isFinal
      };

      onResult?.(result);

      // Start background refinement if needed
      if (maxK > 1 && !this.abortController.signal.aborted) {
        this.refineSimulationAsync(genId, 2, maxK, onResult);
      }

    } catch (error) {
      const result: StructuredSimulationResult = {
        type: 'errored',
        genId,
        error: error instanceof Error ? error.message : String(error)
      };
      onResult?.(result);
    }
  }

  private async refineSimulationAsync(
    genId: number,
    targetK: number,
    maxK: number,
    onResult?: (result: StructuredSimulationResult) => void
  ) {
    // Use an abortable loop instead of recursive setTimeout
    while (targetK <= maxK && genId === this.currentGenId && this.activeRunner && !this.abortController?.signal.aborted) {
      try {
        // Run incremental chunk
        this.activeRunner.run_chunk(200);

        const output = this.activeRunner.get_analysis(targetK);
        const { results, analysis, firstRunEvents } = output;

        const isFinal = targetK === maxK;
        const result: StructuredSimulationResult = {
          type: 'completed',
          genId,
          results,
          analysis,
          events: firstRunEvents,
          kFactor: targetK,
          isFinal
        };

        onResult?.(result);

        // Exit loop if this is the final iteration
        if (isFinal) {
          break;
        }

        targetK++;

        // Yield control to allow abort signal checks and UI updates
        await new Promise(resolve => setTimeout(resolve, 0));

      } catch (error) {
        const result: StructuredSimulationResult = {
          type: 'errored',
          genId,
          error: error instanceof Error ? error.message : String(error)
        };
        onResult?.(result);
        break;
      }
    }

    // Check if we exited due to cancellation
    if (this.abortController?.signal.aborted && genId === this.currentGenId) {
      const result: StructuredSimulationResult = {
        type: 'cancelled',
        genId
      };
      onResult?.(result);
    }
  }

  public async autoAdjustEncounter(
    players: any[],
    monsters: any[],
    timeline: any[],
    encounterIndex: number,
    genId: number,
    onResult?: (result: StructuredSimulationResult) => void
  ) {
    this.currentGenId = genId;
    this.abortController = new AbortController();

    try {
      await this.ensureWasmInitialized();
      const adjustmentResult = auto_adjust_encounter_wasm(players, monsters, timeline, encounterIndex);

      if (!this.abortController.signal.aborted) {
        const result: StructuredSimulationResult = {
          type: 'completed',
          genId,
          result: adjustmentResult
        };
        onResult?.(result);
      } else {
        const result: StructuredSimulationResult = {
          type: 'cancelled',
          genId
        };
        onResult?.(result);
      }

    } catch (error) {
      const result: StructuredSimulationResult = {
        type: 'errored',
        genId,
        error: error instanceof Error ? error.message : String(error)
      };
      onResult?.(result);
    }
  }

  public cancel() {
    if (this.abortController) {
      this.abortController.abort();
    }
  }

  public restart() {
    this.cancel();
    this.initializeWorker();
  }

  public terminate() {
    this.cancel();
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
    }
    this.activeRunner = null;
    SimulationWorkerController.instanceCount--;
  }

  // Instrumentation methods for testing
  public static getInstanceCount(): number {
    return SimulationWorkerController.instanceCount;
  }

  public getInstanceId(): number {
    return this.instanceId;
  }
}