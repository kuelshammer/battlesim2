import React from 'react'
import { useLoading, useLoadingOperation } from '../utils/useLoading'

/**
 * Example component demonstrating the Loading State Management System
 *
 * This component shows how to:
 * - Use the global loading state
 * - Create individual loading operations
 * - Display loading progress and status
 */
export const LoadingExample: React.FC = () => {
  const loading = useLoading()

  // Individual loading operation for a specific task
  const dataLoading = useLoadingOperation('simulation-data', 'high')

  const handleLoadData = async () => {
    dataLoading.start('Loading simulation data...')

    try {
      // Simulate data loading with progress updates
      dataLoading.update(25, 'Connecting to data source...')
      await new Promise(resolve => setTimeout(resolve, 1000))

      dataLoading.update(50, 'Fetching records...')
      await new Promise(resolve => setTimeout(resolve, 1000))

      dataLoading.update(75, 'Processing data...')
      await new Promise(resolve => setTimeout(resolve, 1000))

      dataLoading.update(100, 'Data loaded successfully')
      dataLoading.complete('Simulation data loaded')
    } catch {
      dataLoading.error('Failed to load data', 'Network error occurred')
    }
  }

  const handleSimulateWork = async () => {
    const operationId = loading.startOperation('background-tasks', 'medium', 'Running simulation...')

    try {
      // Simulate work with progress
      for (let i = 0; i <= 100; i += 10) {
        loading.updateProgress(operationId, i, `Processing... ${i}%`)
        await new Promise(resolve => setTimeout(resolve, 200))
      }

      loading.completeOperation(operationId, 'Simulation completed')
    } catch {
      loading.errorOperation(operationId, 'Simulation failed', 'Unexpected error')
    }
  }

  const handleSimulateError = () => {
    loading.startOperation('core-ui', 'critical', 'Critical operation...')
    setTimeout(() => {
      loading.errorOperation('loading-op-1-1234567890', 'Critical system error', 'System failure')
    }, 2000)
  }

  return (
    <div style={{ padding: '20px', fontFamily: 'Arial, sans-serif' }}>
      <h2>Loading State Management Example</h2>

      {/* Global Loading Status */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '1px solid #ccc' }}>
        <h3>Global Loading Status</h3>
        <p>Active Operations: {loading.activeOperations}</p>
        <p>Completed: {loading.completedOperations}</p>
        <p>Failed: {loading.failedOperations}</p>
        <p>Global Progress: {loading.globalProgress.toFixed(1)}%</p>
        <p>Is Loading: {loading.isLoading ? 'Yes' : 'No'}</p>

        {loading.isLoading && (
          <div style={{
            width: '100%',
            height: '10px',
            backgroundColor: '#f0f0f0',
            borderRadius: '5px',
            overflow: 'hidden'
          }}>
            <div style={{
              width: `${loading.globalProgress}%`,
              height: '100%',
              backgroundColor: '#007bff',
              transition: 'width 0.3s ease'
            }} />
          </div>
        )}
      </div>

      {/* Action Buttons */}
      <div style={{ marginBottom: '20px' }}>
        <button
          onClick={handleLoadData}
          disabled={dataLoading.isActive}
          style={{
            padding: '10px 20px',
            marginRight: '10px',
            backgroundColor: dataLoading.isActive ? '#ccc' : '#007bff',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
            cursor: dataLoading.isActive ? 'not-allowed' : 'pointer'
          }}
        >
          {dataLoading.isActive ? 'Loading Data...' : 'Load Data'}
        </button>

        <button
          onClick={handleSimulateWork}
          style={{
            padding: '10px 20px',
            marginRight: '10px',
            backgroundColor: '#28a745',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer'
          }}
        >
          Simulate Work
        </button>

        <button
          onClick={handleSimulateError}
          style={{
            padding: '10px 20px',
            backgroundColor: '#dc3545',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer'
          }}
        >
          Simulate Error
        </button>
      </div>

      {/* Operations Summary */}
      <div style={{ padding: '10px', border: '1px solid #ccc' }}>
        <h3>Operations by Type</h3>
        <ul>
          {Object.entries(loading.summary.operationsByType).map(([type, count]) => (
            <li key={type}>{type}: {count} operations</li>
          ))}
        </ul>

        <h3>Operations by Status</h3>
        <ul>
          {Object.entries(loading.summary.operationsByStatus).map(([status, count]) => (
            <li key={status}>{status}: {count} operations</li>
          ))}
        </ul>
      </div>
    </div>
  )
}

export default LoadingExample