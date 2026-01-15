import { useState, useEffect, useCallback } from 'react'
import {
  LoadingManager,
  LoadingStateSummary,
  LoadingStateChangeEvent,
  LoadingStateType,
  LoadingPriority,
  loadingManager
} from '../../model/loading'

/**
 * Hook for using the LoadingManager in React components
 *
 * @param manager - Optional LoadingManager instance (defaults to global singleton)
 * @returns Loading state and control functions
 */
export function useLoading(manager: LoadingManager = loadingManager) {
  const [summary, setSummary] = useState<LoadingStateSummary>(() => manager.getSummary())

  useEffect(() => {
    const unsubscribe = manager.onSummaryUpdate(setSummary)
    return unsubscribe
  }, [manager])

  const startOperation = useCallback(
    (type: LoadingStateType, priority?: LoadingPriority, message?: string, metadata?: Record<string, unknown>) => {
      return manager.startOperation(type, priority, message, metadata)
    },
    [manager]
  )

  const updateProgress = useCallback(
    (operationId: string, progress: number, message?: string) => {
      manager.updateProgress(operationId, progress, message)
    },
    [manager]
  )

  const completeOperation = useCallback(
    (operationId: string, message?: string) => {
      manager.completeOperation(operationId, message)
    },
    [manager]
  )

  const errorOperation = useCallback(
    (operationId: string, error: string, message?: string) => {
      manager.errorOperation(operationId, error, message)
    },
    [manager]
  )

  const cancelOperation = useCallback(
    (operationId: string) => {
      manager.cancelOperation(operationId)
    },
    [manager]
  )

  return {
    // State
    summary,
    isLoading: summary.isLoading,
    globalProgress: summary.globalProgress,
    activeOperations: summary.activeOperations,
    completedOperations: summary.completedOperations,
    failedOperations: summary.failedOperations,

    // Actions
    startOperation,
    updateProgress,
    completeOperation,
    errorOperation,
    cancelOperation,

    // Utilities
    getOperation: manager.getOperation.bind(manager),
    getOperations: manager.getOperations.bind(manager),
    cleanup: manager.cleanup.bind(manager),
    reset: manager.reset.bind(manager)
  }
}

/**
 * Hook for subscribing to loading state changes for specific operations
 *
 * @param operationId - Optional operation ID to filter events
 * @param manager - Optional LoadingManager instance
 * @returns Loading state change events
 */
export function useLoadingEvents(
  operationId?: string,
  manager: LoadingManager = loadingManager
) {
  const [events, setEvents] = useState<LoadingStateChangeEvent[]>([])

  useEffect(() => {
    const callback = (event: LoadingStateChangeEvent) => {
      if (!operationId || event.operationId === operationId) {
        setEvents(prev => [...prev.slice(-9), event]) // Keep last 10 events
      }
    }

    const unsubscribe = manager.onStateChange(callback)
    return unsubscribe
  }, [operationId, manager])

  return events
}

/**
 * Hook for managing a single loading operation with automatic cleanup
 *
 * @param type - Loading state type
 * @param priority - Loading priority
 * @param manager - Optional LoadingManager instance
 * @returns Operation control functions and state
 */
export function useLoadingOperation(
  type: LoadingStateType,
  priority: LoadingPriority = 'medium',
  manager: LoadingManager = loadingManager
) {
  const [operationId, setOperationId] = useState<string | null>(null)
  const [isActive, setIsActive] = useState(false)

  const start = useCallback(
    (message?: string, metadata?: Record<string, unknown>) => {
      if (operationId) {
        console.warn('Loading operation already active')
        return operationId
      }

      const id = manager.startOperation(type, priority, message, metadata)
      setOperationId(id)
      setIsActive(true)
      return id
    },
    [type, priority, manager, operationId]
  )

  const update = useCallback(
    (progress: number, message?: string) => {
      if (!operationId) {
        console.warn('No active loading operation')
        return
      }
      manager.updateProgress(operationId, progress, message)
    },
    [operationId, manager]
  )

  const complete = useCallback(
    (message?: string) => {
      if (!operationId) {
        console.warn('No active loading operation')
        return
      }
      manager.completeOperation(operationId, message)
      setOperationId(null)
      setIsActive(false)
    },
    [operationId, manager]
  )

  const error = useCallback(
    (error: string, message?: string) => {
      if (!operationId) {
        console.warn('No active loading operation')
        return
      }
      manager.errorOperation(operationId, error, message)
      setOperationId(null)
      setIsActive(false)
    },
    [operationId, manager]
  )

  const cancel = useCallback(
    () => {
      if (!operationId) {
        console.warn('No active loading operation')
        return
      }
      manager.cancelOperation(operationId)
      setOperationId(null)
      setIsActive(false)
    },
    [operationId, manager]
  )

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (operationId) {
        manager.cancelOperation(operationId)
      }
    }
  }, [operationId, manager])

  return {
    operationId,
    isActive,
    start,
    update,
    complete,
    error,
    cancel
  }
}