import React from 'react'
import styles from './ProgressUI.module.scss'

export interface ProgressVisual {
  percentage: number
  formatted_progress: string
  phase: string
  time_remaining: string
  iterations_completed: number
  total_iterations: number
  progress_segments: ProgressSegment[]
}

export interface ProgressSegment {
  start: number
  end: number
  style: 'Normal' | 'Warning' | 'Error' | 'Success' | 'Active' | { Custom: string }
  label: string
}

export interface ProgressInfo {
  simulation_id: string
  state: 'Idle' | 'Running' | 'Completed' | 'Failed' | 'Cancelled'
  visual: ProgressVisual
  messages: string[]
  start_time: number
  end_time: number | null
  execution_time_ms: number | null
  metadata: Record<string, string>
}

interface ProgressVisualizerProps {
  progressInfo: ProgressInfo
  /** Whether to show detailed progress information */
  showDetails?: boolean
  /** Additional CSS classes */
  className?: string
}

/**
 * ProgressVisualizer - Component for rendering structured progress data
 *
 * Renders progress segments, text, and indicators from ProgressVisual data,
 * matching the CSS classes and attributes from the original HTML generation.
 */
export const ProgressVisualizer: React.FC<ProgressVisualizerProps> = ({
  progressInfo,
  showDetails = true,
  className = ''
}) => {
  const getStateClass = () => {
    switch (progressInfo.state) {
      case 'Idle':
        return styles.idle
      case 'Running':
        return styles.running
      case 'Completed':
        return styles.completed
      case 'Failed':
        return styles.failed
      case 'Cancelled':
        return styles.cancelled
      default:
        return ''
    }
  }

  const getSegmentColor = (style: ProgressSegment['style']) => {
    switch (style) {
      case 'Normal':
        return '#3b82f6' // blue-500
      case 'Active':
        return '#10b981' // emerald-500
      case 'Success':
        return '#22c55e' // green-500
      case 'Warning':
        return '#f59e0b' // amber-500
      case 'Error':
        return '#ef4444' // red-500
      default:
        if (typeof style === 'object' && 'Custom' in style) {
          return style.Custom
        }
        return '#3b82f6'
    }
  }

  return (
    <div className={`${styles.progressContainer} ${className}`} data-simulation-id={progressInfo.simulation_id}>
      {/* Progress bar */}
      <div className={styles.progressBar}>
        {progressInfo.visual.progress_segments.map((segment, index) => (
          <div
            key={index}
            className={styles.progressSegment}
            style={{
              left: `${segment.start * 100}%`,
              width: `${(segment.end - segment.start) * 100}%`,
              backgroundColor: getSegmentColor(segment.style)
            }}
            title={segment.label || undefined}
          />
        ))}
      </div>

      {/* Progress text */}
      {showDetails && (
        <div className={styles.progressText}>
          {progressInfo.visual.formatted_progress} - {progressInfo.visual.phase}
        </div>
      )}

      {/* Status indicator */}
      <div className={`${styles.progressIndicator} ${getStateClass()}`}>
        {progressInfo.visual.percentage.toFixed(1)}%
      </div>
    </div>
  )
}

interface CompactProgressIndicatorProps {
  progressInfo: ProgressInfo
  /** Additional CSS classes */
  className?: string
}

/**
 * CompactProgressIndicator - Minimal progress indicator
 *
 * Shows a compact progress indicator with state-based styling.
 */
export const CompactProgressIndicator: React.FC<CompactProgressIndicatorProps> = ({
  progressInfo,
  className = ''
}) => {
  const getStateClass = () => {
    switch (progressInfo.state) {
      case 'Idle':
        return styles.idle
      case 'Running':
        return styles.running
      case 'Completed':
        return styles.completed
      case 'Failed':
        return styles.failed
      case 'Cancelled':
        return styles.cancelled
      default:
        return ''
    }
  }

  return (
    <div
      className={`${styles.progressIndicator} ${getStateClass()} ${className}`}
      data-simulation-id={progressInfo.simulation_id}
      title={progressInfo.visual.phase}
    >
      {progressInfo.visual.percentage.toFixed(1)}%
    </div>
  )
}

export default ProgressVisualizer