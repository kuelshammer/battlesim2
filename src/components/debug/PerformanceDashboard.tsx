/**
 * Performance Dashboard Component
 *
 * Displays real-time performance metrics for the simulation engine.
 * Only visible in development mode.
 */

import React, { FC, useEffect, useState } from 'react'
import styles from './PerformanceDashboard.module.scss'

export interface PerformanceMetrics {
  lastSimulationTime: number // Total time for last simulation in ms
  averageRoundTime: number // Average time per round in ms
  slowestRound: number // Slowest round time in ms
  fastestRound: number // Fastest round time in ms
  totalRounds: number // Total number of rounds
  wasmSize: number // WASM bundle size in bytes
  memoryUsage: number // Memory usage in bytes (if available)
}

interface RoundTime {
  round: number
  duration: number
}

const EMPTY_METRICS: PerformanceMetrics = {
  lastSimulationTime: 0,
  averageRoundTime: 0,
  slowestRound: 0,
  fastestRound: 0,
  totalRounds: 0,
  wasmSize: 0,
  memoryUsage: 0,
}

export interface PerformanceDashboardProps {
  /**
   * Whether the dashboard is visible
   */
  isVisible: boolean

  /**
   * Callback to close the dashboard
   */
  onClose?: () => void
}

/**
 * Format bytes to human readable string
 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(i === 0 ? 0 : 1)} ${sizes[i]}`
}

/**
 * Format milliseconds to human readable string
 */
function formatDuration(ms: number): string {
  if (ms < 1) return `${(ms * 1000).toFixed(1)}µs`
  if (ms < 1000) return `${ms.toFixed(1)}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

const PerformanceDashboard: FC<PerformanceDashboardProps> = ({ isVisible, onClose }) => {
  const [metrics, setMetrics] = useState<PerformanceMetrics>(EMPTY_METRICS)
  const [roundTimes, setRoundTimes] = useState<RoundTime[]>([])

  // Fetch WASM size
  useEffect(() => {
    const fetchWasmSize = async () => {
      try {
        const response = await fetch('/pkg/simulation_wasm_bg.wasm')
        if (response.ok) {
          const blob = await response.blob()
          setMetrics((prev) => ({ ...prev, wasmSize: blob.size }))
        }
      } catch (error) {
        console.error('Failed to fetch WASM size:', error)
      }
    }

    if (isVisible) {
      fetchWasmSize()
    }
  }, [isVisible])

  // Listen for performance events from the simulation
  useEffect(() => {
    if (!isVisible) return

    const handleSimulationComplete = (event: Event) => {
      const customEvent = event as CustomEvent<{
        totalTime: number
        roundTimes: RoundTime[]
      }>

      if (customEvent.detail) {
        const { totalTime, roundTimes: newRoundTimes } = customEvent.detail

        const roundDurations = newRoundTimes.map((r) => r.duration)
        const avgRoundTime =
          roundDurations.length > 0
            ? roundDurations.reduce((a, b) => a + b, 0) / roundDurations.length
            : 0
        const slowestRound = roundDurations.length > 0 ? Math.max(...roundDurations) : 0
        const fastestRound = roundDurations.length > 0 ? Math.min(...roundDurations) : 0

        setMetrics({
          lastSimulationTime: totalTime,
          averageRoundTime: avgRoundTime,
          slowestRound,
          fastestRound,
          totalRounds: newRoundTimes.length,
          wasmSize: metrics.wasmSize,
          memoryUsage: 0, // @ts-expect-error - performance.memory is non-standard
          ...(typeof performance !== 'undefined' &&
          (performance as { memory?: { usedJSHeapSize: number } }).memory &&
          {
            memoryUsage: (performance as { memory?: { usedJSHeapSize: number } }).memory!.usedJSHeapSize,
          }),
        })

        setRoundTimes(newRoundTimes)
      }
    }

    // Listen for custom event from simulation
    window.addEventListener('simulationComplete', handleSimulationComplete)

    // Also listen for console logs (from Rust)
    const originalLog = console.log
    console.log = (...args: unknown[]) => {
      originalLog.apply(console, args)

      // Parse performance logs
      const message = args.join(' ')
      const roundMatch = message.match(/Round (\d+) completed in ([\d.]+)(ms|µs)/)
      const encounterMatch = message.match(/Encounter completed in ([\d.]+)(ms|µs|s)/)

      if (roundMatch) {
        const round = parseInt(roundMatch[1])
        const duration = parseFloat(roundMatch[2])
        const unit = roundMatch[3]
        const durationMs = unit === 'µs' ? duration / 1000 : duration

        setRoundTimes((prev) => {
          const existing = prev.find((r) => r.round === round)
          if (existing) {
            return prev.map((r) => (r.round === round ? { ...r, duration: durationMs } : r))
          }
          return [...prev, { round, duration: durationMs }]
        })
      }

      if (encounterMatch) {
        const duration = parseFloat(encounterMatch[1])
        const unit = encounterMatch[2]
        const durationMs = unit === 's' ? duration * 1000 : unit === 'µs' ? duration / 1000 : duration

        setMetrics((prev) => ({
          ...prev,
          lastSimulationTime: durationMs,
        }))
      }
    }

    return () => {
      window.removeEventListener('simulationComplete', handleSimulationComplete)
      console.log = originalLog
    }
  }, [isVisible, metrics.wasmSize])

  // Get memory usage periodically
  useEffect(() => {
    if (!isVisible) return

    const interval = setInterval(() => {
      // @ts-expect-error - performance.memory is non-standard but available in Chrome
      if (typeof performance !== 'undefined' && (performance as { memory?: { usedJSHeapSize: number } }).memory) {
        setMetrics((prev) => ({
          ...prev,
          memoryUsage: (performance as { memory?: { usedJSHeapSize: number } }).memory!.usedJSHeapSize,
        }))
      }
    }, 1000)

    return () => clearInterval(interval)
  }, [isVisible])

  const maxRoundTime = Math.max(...roundTimes.map((r) => r.duration), 1)

  if (!isVisible) return null

  return (
    <div className={styles.dashboard}>
      <div className={styles.header}>
        <h4>Performance Dashboard</h4>
        {onClose && (
          <button className={styles.closeButton} onClick={onClose}>
            ×
          </button>
        )}
      </div>

      <div className={styles.metrics}>
        <div className={styles.metric}>
          <span className={styles.label}>Last Simulation:</span>
          <span className={styles.value}>{formatDuration(metrics.lastSimulationTime)}</span>
        </div>

        <div className={styles.metric}>
          <span className={styles.label}>Avg Round:</span>
          <span className={styles.value}>{formatDuration(metrics.averageRoundTime)}</span>
        </div>

        <div className={styles.metric}>
          <span className={styles.label}>Slowest Round:</span>
          <span className={styles.value}>{formatDuration(metrics.slowestRound)}</span>
        </div>

        <div className={styles.metric}>
          <span className={styles.label}>Fastest Round:</span>
          <span className={styles.value}>{formatDuration(metrics.fastestRound)}</span>
        </div>

        <div className={styles.metric}>
          <span className={styles.label}>Total Rounds:</span>
          <span className={styles.value}>{metrics.totalRounds}</span>
        </div>

        <div className={styles.metric}>
          <span className={styles.label}>WASM Bundle:</span>
          <span
            className={`${styles.value} ${
              metrics.wasmSize > 1024 * 1024 ? styles.warning : ''
            }`}
          >
            {formatBytes(metrics.wasmSize)}
            {metrics.wasmSize > 1024 * 1024 && ' ⚠️'}
          </span>
        </div>

        <div className={styles.metric}>
          <span className={styles.label}>Memory:</span>
          <span className={styles.value}>{formatBytes(metrics.memoryUsage)}</span>
        </div>
      </div>

      {roundTimes.length > 0 && (
        <div className={styles.chartContainer}>
          <h5>Round Times</h5>
          <div className={styles.chart}>
            {roundTimes.map((roundTime) => (
              <div
                key={roundTime.round}
                className={styles.bar}
                style={{
                  height: `${(roundTime.duration / maxRoundTime) * 100}%`,
                  width: `${100 / Math.max(roundTimes.length, 1)}%`,
                }}
                title={`Round ${roundTime.round}: ${formatDuration(roundTime.duration)}`}
              />
            ))}
          </div>
          <div className={styles.chartLabels}>
            <span>R1</span>
            <span>R{roundTimes.length}</span>
          </div>
        </div>
      )}

      {metrics.wasmSize > 1024 * 1024 && (
        <div className={styles.alert}>
          ⚠️ WASM bundle is larger than 1MB. Consider:
          <ul>
            <li>Removing unused dependencies</li>
            <li>Enabling LTO (Link Time Optimization)</li>
            <li>Using wasm-opt for size optimization</li>
          </ul>
        </div>
      )}
    </div>
  )
}

export default PerformanceDashboard
