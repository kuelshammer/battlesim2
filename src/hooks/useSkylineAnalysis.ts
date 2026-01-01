/**
 * useSkylineAnalysis - Hook to generate Skyline data from simulation results
 *
 * Calls WASM run_skyline_analysis_wasm() to transform simulation results
 * into 100-bucket percentile aggregation for visualization.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { SimulationResult } from '@/model/model';
import { SkylineAnalysis } from '@/model/skylineTypes';
import * as simulation_wasm from 'simulation-wasm';

export interface UseSkylineAnalysisOptions {
    /** Party size (number of characters) */
    partySize: number;
    /** Encounter index in adventuring day (null if single encounter) */
    encounterIndex?: number | null;
    /** Whether to enable the analysis */
    enabled?: boolean;
}

export interface UseSkylineAnalysisResult {
    /** Generated Skyline data (null if not ready) */
    data: SkylineAnalysis | null;
    /** Whether analysis is in progress */
    loading: boolean;
    /** Error message if analysis failed */
    error: string | null;
    /** Manually trigger analysis */
    refetch: () => void;
}

/**
 * Hook to run Skyline analysis on simulation results
 *
 * @param results - Array of simulation results (100 iterations recommended)
 * @param options - Configuration options
 * @returns Skyline data and state
 */
export function useSkylineAnalysis(
    results: SimulationResult[],
    options: UseSkylineAnalysisOptions
): UseSkylineAnalysisResult {
    const { partySize, encounterIndex = null, enabled = true } = options;

    const [data, setData] = useState<SkylineAnalysis | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Track previous results to detect changes
    const prevResultsRef = useRef<SimulationResult[]>([]);

    /**
     * Run the Skyline analysis
     */
    const runAnalysis = useCallback(() => {
        // Skip if not enabled or no results
        if (!enabled || results.length === 0) {
            setData(null);
            setError(null);
            setLoading(false);
            return;
        }

        // Check if results actually changed (by reference or length)
        const resultsChanged =
            prevResultsRef.current !== results ||
            prevResultsRef.current.length !== results.length;

        if (!resultsChanged && data !== null) {
            return; // Already analyzed
        }

        setLoading(true);
        setError(null);

        try {
            // Call WASM function to generate Skyline data
            // WASM will handle sorting by encounter-specific score when encounterIndex is provided
            const wasmResult = simulation_wasm.run_skyline_analysis_wasm(
                results,
                partySize,
                encounterIndex
            );

            // Parse the WASM result
            const analysis: SkylineAnalysis = JSON.parse(JSON.stringify(wasmResult));

            setData(analysis);
            prevResultsRef.current = results;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : String(err);
            console.error('Skyline analysis failed:', errorMessage);
            setError(errorMessage);
            setData(null);
        } finally {
            setLoading(false);
        }
    }, [results, partySize, encounterIndex, enabled, data]);

    // Run analysis when dependencies change
    useEffect(() => {
        runAnalysis();
    }, [runAnalysis]);

    return {
        data,
        loading,
        error,
        refetch: runAnalysis,
    };
}

/**
 * Hook to run Skyline analysis with debouncing (for large result sets)
 *
 * @param results - Array of simulation results
 * @param options - Configuration options
 * @param debounceMs - Debounce delay in milliseconds (default: 500ms)
 * @returns Skyline data and state
 */
export function useSkylineAnalysisDebounced(
    results: SimulationResult[],
    options: UseSkylineAnalysisOptions,
    debounceMs: number = 500
): UseSkylineAnalysisResult {
    const [data, setData] = useState<SkylineAnalysis | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        // Clear previous timeout
        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }

        // Skip if not enabled or no results
        if (!options.enabled || results.length === 0) {
            setData(null);
            setLoading(false);
            setError(null);
            return;
        }

        // Set loading state
        setLoading(true);
        setError(null);

        // Debounce the analysis
        timeoutRef.current = setTimeout(() => {
            try {
                // WASM will handle sorting by encounter-specific score when encounterIndex is provided
                const wasmResult = simulation_wasm.run_skyline_analysis_wasm(
                    results,
                    options.partySize,
                    options.encounterIndex
                );
                const analysis: SkylineAnalysis = JSON.parse(JSON.stringify(wasmResult));
                setData(analysis);
            } catch (err) {
                const errorMessage = err instanceof Error ? err.message : String(err);
                console.error('Skyline analysis failed:', errorMessage);
                setError(errorMessage);
                setData(null);
            } finally {
                setLoading(false);
            }
        }, debounceMs);

        // Cleanup timeout on unmount
        return () => {
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
            }
        };
    }, [results, options.partySize, options.encounterIndex, options.enabled, debounceMs]);

    const refetch = useCallback(() => {
        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }
        setLoading(true);
        // Trigger immediate re-run by setting a new timeout with 0ms
        timeoutRef.current = setTimeout(() => {
            // The useEffect will re-run with the same deps
            setLoading(false);
        }, 0);
    }, []);

    return { data, loading, error, refetch };
}
