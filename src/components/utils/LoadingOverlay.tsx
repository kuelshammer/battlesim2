import React from 'react'
import { useLoading } from './useLoading'
import styles from './LoadingOverlay.module.scss'

interface LoadingOverlayProps {
  /** Whether the overlay should be visible */
  visible?: boolean
  /** Custom message to display instead of the default */
  message?: string
  /** Whether to show detailed operation information */
  showDetails?: boolean
  /** Additional CSS classes */
  className?: string
}

/**
 * LoadingOverlay - Full-screen overlay component for displaying overall loading progress
 *
 * Shows global loading state with progress bar, status messages, and optional detailed information
 * about active operations. Uses the LoadingManager system for state management.
 */
export const LoadingOverlay: React.FC<LoadingOverlayProps> = ({
  visible = true,
  message,
  showDetails = false,
  className = ''
}) => {
  const { summary, isLoading, globalProgress } = useLoading()

  // Don't render if not visible or not loading
  if (!visible || !isLoading) {
    return null
  }

  const displayMessage = message || summary.highestPriorityActive
    ? `Loading ${summary.highestPriorityActive} priority operations...`
    : 'Loading...'

  return (
    <div className={`${styles.overlay} ${className}`} role="dialog" aria-live="polite" aria-label="Loading overlay">
      <div className={styles.content}>
        <div className={styles.spinner}>
          <div className={styles.spinnerRing}></div>
          <div className={styles.spinnerRing}></div>
          <div className={styles.spinnerRing}></div>
        </div>

        <div className={styles.message}>
          <h3>{displayMessage}</h3>
          <div className={styles.progressContainer}>
            <div className={styles.progressBar}>
              <div
                className={styles.progressFill}
                style={{ width: `${globalProgress}%` }}
                aria-valuenow={globalProgress}
                aria-valuemin={0}
                aria-valuemax={100}
              />
            </div>
            <span className={styles.progressText}>
              {globalProgress.toFixed(0)}%
            </span>
          </div>
        </div>

        {showDetails && (
          <div className={styles.details}>
            <div className={styles.stats}>
              <span>Active: {summary.activeOperations}</span>
              <span>Completed: {summary.completedOperations}</span>
              <span>Failed: {summary.failedOperations}</span>
            </div>

            {summary.activeOperations > 0 && (
              <div className={styles.operations}>
                <h4>Active Operations:</h4>
                <ul>
                  {summary.operationsByType &&
                    Object.entries(summary.operationsByType)
                      .filter(([, count]) => count > 0)
                      .map(([type, count]) => (
                        <li key={type}>
                          {type}: {count} operation{count !== 1 ? 's' : ''}
                        </li>
                      ))
                  }
                </ul>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}

export default LoadingOverlay