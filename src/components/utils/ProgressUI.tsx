import React from 'react'
import { useLoading, useLoadingEvents } from './useLoading'
import { LoadingOperation, LoadingStateType } from '../../model/loading'
import LoadingSpinner from './LoadingSpinner'
import styles from './ProgressUI.module.scss'

interface ProgressUIProps {
  /** Filter operations by type */
  operationType?: LoadingStateType
  /** Filter operations by specific operation ID */
  operationId?: string
  /** Whether to show completed operations */
  showCompleted?: boolean
  /** Whether to show failed operations */
  showFailed?: boolean
  /** Maximum number of operations to display */
  maxOperations?: number
  /** Whether to show detailed progress information */
  showDetails?: boolean
  /** Custom title */
  title?: string
  /** Additional CSS classes */
  className?: string
  /** Compact mode for smaller spaces */
  compact?: boolean
}

/**
 * ProgressUI - Component for tracking and displaying background operation progress
 *
 * Shows progress for specific operations or operation types, with detailed information
 * about active, completed, and failed operations.
 */
export const ProgressUI: React.FC<ProgressUIProps> = ({
  operationType,
  operationId,
  showCompleted = true,
  showFailed = true,
  maxOperations = 5,
  showDetails = false,
  title,
  className = '',
  compact = false
}) => {
  const { getOperations, getOperation } = useLoading()
  const events = useLoadingEvents(operationId)

  // Get operations based on filters
  const getFilteredOperations = (): LoadingOperation[] => {
    let operations: LoadingOperation[]

    if (operationId) {
      const operation = getOperation(operationId)
      operations = operation ? [operation] : []
    } else {
      operations = getOperations({
        type: operationType,
        status: undefined // Include all statuses
      })
    }

    // Filter by completion status
    operations = operations.filter(op => {
      if (op.status === 'completed' && !showCompleted) return false
      if (op.status === 'error' && !showFailed) return false
      return true
    })

    // Sort by priority and start time (highest priority first, then most recent)
    operations.sort((a, b) => {
      const priorityOrder = { critical: 4, high: 3, medium: 2, low: 1 }
      const priorityDiff = priorityOrder[b.priority] - priorityOrder[a.priority]
      if (priorityDiff !== 0) return priorityDiff
      return b.startTime.getTime() - a.startTime.getTime()
    })

    return operations.slice(0, maxOperations)
  }

  const operations = getFilteredOperations()
  const activeOperations = operations.filter(op => op.status === 'loading')
  const completedOperations = operations.filter(op => op.status === 'completed')
  const failedOperations = operations.filter(op => op.status === 'error')

  const displayTitle = title || (operationType ? `${operationType} Operations` : 'Background Operations')

  if (operations.length === 0) {
    return null
  }

  return (
    <div className={`${styles.container} ${compact ? styles.compact : ''} ${className}`}>
      <div className={styles.header}>
        <h4 className={styles.title}>{displayTitle}</h4>
        <div className={styles.summary}>
          {activeOperations.length > 0 && (
            <span className={styles.activeCount}>
              {activeOperations.length} active
            </span>
          )}
          {completedOperations.length > 0 && (
            <span className={styles.completedCount}>
              {completedOperations.length} completed
            </span>
          )}
          {failedOperations.length > 0 && (
            <span className={styles.failedCount}>
              {failedOperations.length} failed
            </span>
          )}
        </div>
      </div>

      <div className={styles.operations}>
        {operations.map(operation => (
          <OperationProgress
            key={operation.id}
            operation={operation}
            showDetails={showDetails}
            compact={compact}
          />
        ))}
      </div>

      {events.length > 0 && showDetails && (
        <div className={styles.events}>
          <h5>Recent Events</h5>
          <ul>
            {events.slice(0, 3).map((event, index) => (
              <li key={index} className={styles.event}>
                <span className={styles.eventType}>
                  {event.currentState.status}
                </span>
                <span className={styles.eventMessage}>
                  {event.currentState.message || 'Operation updated'}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}

interface OperationProgressProps {
  operation: LoadingOperation
  showDetails: boolean
  compact: boolean
}

const OperationProgress: React.FC<OperationProgressProps> = ({
  operation,
  showDetails,
  compact
}) => {
  const getStatusIcon = () => {
    switch (operation.status) {
      case 'loading':
        return <LoadingSpinner size="small" variant="primary" inline />
      case 'completed':
        return <span className={styles.statusIcon}>✓</span>
      case 'error':
        return <span className={styles.statusIcon}>✗</span>
      case 'idle':
        return <span className={styles.statusIcon}>○</span>
      default:
        return null
    }
  }

  const getStatusClass = () => {
    switch (operation.status) {
      case 'loading':
        return styles.loading
      case 'completed':
        return styles.completed
      case 'error':
        return styles.error
      case 'idle':
        return styles.idle
      default:
        return ''
    }
  }

  const formatDuration = (startTime: Date, endTime?: Date) => {
    const end = endTime || new Date()
    const duration = end.getTime() - startTime.getTime()
    const seconds = Math.floor(duration / 1000)
    if (seconds < 60) return `${seconds}s`
    const minutes = Math.floor(seconds / 60)
    return `${minutes}m ${seconds % 60}s`
  }

  return (
    <div className={`${styles.operation} ${getStatusClass()} ${compact ? styles.compact : ''}`}>
      <div className={styles.operationHeader}>
        <div className={styles.operationIcon}>
          {getStatusIcon()}
        </div>
        <div className={styles.operationInfo}>
          <div className={styles.operationType}>
            {operation.type}
          </div>
          {operation.message && (
            <div className={styles.operationMessage}>
              {operation.message}
            </div>
          )}
        </div>
        <div className={styles.operationMeta}>
          <span className={styles.priority} data-priority={operation.priority}>
            {operation.priority}
          </span>
        </div>
      </div>

      {operation.status === 'loading' && operation.progress !== undefined && (
        <div className={styles.progressBar}>
          <div
            className={styles.progressFill}
            style={{ width: `${operation.progress}%` }}
          />
          <span className={styles.progressText}>
            {operation.progress.toFixed(0)}%
          </span>
        </div>
      )}

      {operation.error && (
        <div className={styles.errorMessage}>
          {operation.error}
        </div>
      )}

      {showDetails && (
        <div className={styles.operationDetails}>
          <div className={styles.detailRow}>
            <span>Started:</span>
            <span>{operation.startTime.toLocaleTimeString()}</span>
          </div>
          <div className={styles.detailRow}>
            <span>Duration:</span>
            <span>{formatDuration(operation.startTime, operation.endTime)}</span>
          </div>
          {operation.metadata && Object.keys(operation.metadata).length > 0 && (
            <div className={styles.detailRow}>
              <span>Metadata:</span>
              <span>{JSON.stringify(operation.metadata)}</span>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default ProgressUI