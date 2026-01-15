import { z } from 'zod'

// Loading state types for different system components
export const LoadingStateTypeList = [
  'core-ui',
  'simulation-engine',
  'simulation-data',
  'background-tasks',
  'ui-toggle-state'
] as const

export const LoadingStateTypeSchema = z.enum(LoadingStateTypeList)
export type LoadingStateType = z.infer<typeof LoadingStateTypeSchema>

// Loading operation status
export const LoadingStatusList = ['idle', 'loading', 'completed', 'error'] as const
export const LoadingStatusSchema = z.enum(LoadingStatusList)
export type LoadingStatus = z.infer<typeof LoadingStatusSchema>

// Loading priority levels
export const LoadingPriorityList = ['low', 'medium', 'high', 'critical'] as const
export const LoadingPrioritySchema = z.enum(LoadingPriorityList)
export type LoadingPriority = z.infer<typeof LoadingPrioritySchema>

// Loading operation interface
export const LoadingOperationSchema = z.object({
  id: z.string(),
  type: LoadingStateTypeSchema,
  status: LoadingStatusSchema,
  priority: LoadingPrioritySchema,
  message: z.string().optional(),
  progress: z.number().min(0).max(100).optional(),
  error: z.string().optional(),
  startTime: z.date(),
  endTime: z.date().optional(),
  metadata: z.any().optional()
})

export type LoadingOperation = z.infer<typeof LoadingOperationSchema>

// Loading state change event
export const LoadingStateChangeEventSchema = z.object({
  operationId: z.string(),
  previousState: LoadingOperationSchema,
  currentState: LoadingOperationSchema,
  timestamp: z.date()
})

export type LoadingStateChangeEvent = z.infer<typeof LoadingStateChangeEventSchema>

// Loading state summary for UI consumption
export const LoadingStateSummarySchema = z.object({
  totalOperations: z.number(),
  activeOperations: z.number(),
  completedOperations: z.number(),
  failedOperations: z.number(),
  operationsByType: z.record(LoadingStateTypeSchema, z.number()),
  operationsByStatus: z.record(LoadingStatusSchema, z.number()),
  highestPriorityActive: LoadingPrioritySchema.optional(),
  isLoading: z.boolean(),
  globalProgress: z.number().min(0).max(100)
})

export type LoadingStateSummary = z.infer<typeof LoadingStateSummarySchema>

// Event listener callback types
export type LoadingStateChangeCallback = (event: LoadingStateChangeEvent) => void
export type LoadingStateSummaryCallback = (summary: LoadingStateSummary) => void

/**
 * LoadingManager - Centralized loading state management system
 *
 * Features:
 * - Track multiple loading states simultaneously
 * - Broadcast state changes to subscribed components
 * - Support priority-based loading operations
 * - Provide methods to start, complete, and error loading operations
 * - Include proper TypeScript typing and error handling
 */
export class LoadingManager {
  private operations = new Map<string, LoadingOperation>()
  private stateChangeListeners = new Set<LoadingStateChangeCallback>()
  private summaryListeners = new Set<LoadingStateSummaryCallback>()
  private operationCounter = 0

  /**
   * Start a new loading operation
   */
  startOperation(
    type: LoadingStateType,
    priority: LoadingPriority = 'medium',
    message?: string,
    metadata?: Record<string, unknown>
  ): string {
    const id = this.generateOperationId()
    const operation: LoadingOperation = {
      id,
      type,
      status: 'loading',
      priority,
      message,
      progress: 0,
      startTime: new Date(),
      metadata
    }

    this.operations.set(id, operation)
    this.broadcastStateChange(operation, operation) // previous state doesn't exist, so use current
    this.broadcastSummaryUpdate()

    return id
  }

  /**
   * Update progress of an existing operation
   */
  updateProgress(operationId: string, progress: number, message?: string): void {
    const operation = this.operations.get(operationId)
    if (!operation) {
      console.warn(`LoadingManager: Operation ${operationId} not found`)
      return
    }

    const previousState = { ...operation }
    operation.progress = Math.max(0, Math.min(100, progress))
    if (message !== undefined) {
      operation.message = message
    }

    this.operations.set(operationId, operation)
    this.broadcastStateChange(previousState, operation)
    this.broadcastSummaryUpdate()
  }

  /**
   * Complete a loading operation
   */
  completeOperation(operationId: string, message?: string): void {
    const operation = this.operations.get(operationId)
    if (!operation) {
      console.warn(`LoadingManager: Operation ${operationId} not found`)
      return
    }

    const previousState = { ...operation }
    operation.status = 'completed'
    operation.progress = 100
    operation.endTime = new Date()
    if (message !== undefined) {
      operation.message = message
    }

    this.operations.set(operationId, operation)
    this.broadcastStateChange(previousState, operation)
    this.broadcastSummaryUpdate()
  }

  /**
   * Mark a loading operation as failed
   */
  errorOperation(operationId: string, error: string, message?: string): void {
    const operation = this.operations.get(operationId)
    if (!operation) {
      console.warn(`LoadingManager: Operation ${operationId} not found`)
      return
    }

    const previousState = { ...operation }
    operation.status = 'error'
    operation.error = error
    operation.endTime = new Date()
    if (message !== undefined) {
      operation.message = message
    }

    this.operations.set(operationId, operation)
    this.broadcastStateChange(previousState, operation)
    this.broadcastSummaryUpdate()
  }

  /**
   * Cancel a loading operation
   */
  cancelOperation(operationId: string): void {
    const operation = this.operations.get(operationId)
    if (!operation) {
      console.warn(`LoadingManager: Operation ${operationId} not found`)
      return
    }

    const previousState = { ...operation }
    operation.status = 'idle'
    operation.endTime = new Date()

    this.operations.set(operationId, operation)
    this.broadcastStateChange(previousState, operation)
    this.broadcastSummaryUpdate()
  }

  /**
   * Get a specific operation by ID
   */
  getOperation(operationId: string): LoadingOperation | undefined {
    return this.operations.get(operationId)
  }

  /**
   * Get all operations, optionally filtered by type or status
   */
  getOperations(
    filter?: {
      type?: LoadingStateType
      status?: LoadingStatus
      priority?: LoadingPriority
    }
  ): LoadingOperation[] {
    let operations = Array.from(this.operations.values())

    if (filter) {
      operations = operations.filter(op => {
        if (filter.type && op.type !== filter.type) return false
        if (filter.status && op.status !== filter.status) return false
        if (filter.priority && op.priority !== filter.priority) return false
        return true
      })
    }

    return operations
  }

  /**
   * Get loading state summary
   */
  getSummary(): LoadingStateSummary {
    const operations = Array.from(this.operations.values())
    const activeOperations = operations.filter(op => op.status === 'loading')
    const completedOperations = operations.filter(op => op.status === 'completed')
    const failedOperations = operations.filter(op => op.status === 'error')

    const operationsByType = operations.reduce((acc, op) => {
      acc[op.type] = (acc[op.type] || 0) + 1
      return acc
    }, {} as Record<LoadingStateType, number>)

    const operationsByStatus = operations.reduce((acc, op) => {
      acc[op.status] = (acc[op.status] || 0) + 1
      return acc
    }, {} as Record<LoadingStatus, number>)

    const highestPriorityActive = activeOperations.length > 0
      ? activeOperations.reduce((highest, op) =>
          this.getPriorityWeight(op.priority) > this.getPriorityWeight(highest.priority) ? op : highest
        ).priority
      : undefined

    // Calculate global progress based on weighted average of active operations
    const globalProgress = activeOperations.length > 0
      ? activeOperations.reduce((sum, op) => sum + (op.progress || 0), 0) / activeOperations.length
      : 0

    return {
      totalOperations: operations.length,
      activeOperations: activeOperations.length,
      completedOperations: completedOperations.length,
      failedOperations: failedOperations.length,
      operationsByType,
      operationsByStatus,
      highestPriorityActive,
      isLoading: activeOperations.length > 0,
      globalProgress
    }
  }

  /**
   * Subscribe to loading state changes
   */
  onStateChange(callback: LoadingStateChangeCallback): () => void {
    this.stateChangeListeners.add(callback)
    return () => this.stateChangeListeners.delete(callback)
  }

  /**
   * Subscribe to summary updates
   */
  onSummaryUpdate(callback: LoadingStateSummaryCallback): () => void {
    this.summaryListeners.add(callback)
    return () => this.summaryListeners.delete(callback)
  }

  /**
   * Clean up completed/failed operations older than the specified time
   */
  cleanup(maxAge: number = 300000): void { // 5 minutes default
    const now = Date.now()
    const toDelete: string[] = []

    for (const [id, operation] of this.operations) {
      if ((operation.status === 'completed' || operation.status === 'error' || operation.status === 'idle') &&
          operation.endTime &&
          (now - operation.endTime.getTime()) > maxAge) {
        toDelete.push(id)
      }
    }

    toDelete.forEach(id => this.operations.delete(id))

    if (toDelete.length > 0) {
      this.broadcastSummaryUpdate()
    }
  }

  /**
   * Reset all operations (useful for testing or full reset)
   */
  reset(): void {
    this.operations.clear()
    this.operationCounter = 0
    this.broadcastSummaryUpdate()
  }

  private generateOperationId(): string {
    return `loading-op-${++this.operationCounter}-${Date.now()}`
  }

  private broadcastStateChange(previousState: LoadingOperation, currentState: LoadingOperation): void {
    const event: LoadingStateChangeEvent = {
      operationId: currentState.id,
      previousState,
      currentState,
      timestamp: new Date()
    }

    this.stateChangeListeners.forEach(callback => {
      try {
        callback(event)
      } catch (error) {
        console.error('LoadingManager: Error in state change callback:', error)
      }
    })
  }

  private broadcastSummaryUpdate(): void {
    const summary = this.getSummary()
    this.summaryListeners.forEach(callback => {
      try {
        callback(summary)
      } catch (error) {
        console.error('LoadingManager: Error in summary update callback:', error)
      }
    })
  }

  private getPriorityWeight(priority: LoadingPriority): number {
    const weights: Record<LoadingPriority, number> = { low: 1, medium: 2, high: 3, critical: 4 }
    return weights[priority]
  }
}

// Singleton instance for global use
export const loadingManager = new LoadingManager()