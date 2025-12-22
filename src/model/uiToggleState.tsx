import { z } from 'zod'
import { createContext, useContext, useState, useEffect, ReactNode, FC } from 'react'
import { loadingManager } from './loading'

// UI Toggle Types
export const UIToggleTypeList = [
  'combat-log',
  'hp-bars',
  'quintile-1',
  'quintile-2',
  'quintile-3',
  'quintile-4',
  'quintile-5',
  'quintile-6',
  'quintile-7',
  'quintile-8',
  'quintile-9',
  'quintile-10'
] as const;

export const UIToggleTypeSchema = z.enum(UIToggleTypeList)
export type UIToggleType = z.infer<typeof UIToggleTypeSchema>

// UI Toggle State Schema
export const UIToggleStateSchema = z.object({
  id: UIToggleTypeSchema,
  enabled: z.boolean(),
  label: z.string(),
  description: z.string().optional(),
  defaultValue: z.boolean(),
  category: z.enum(['display', 'analysis'])
})

export type UIToggleState = z.infer<typeof UIToggleStateSchema>

// UI Toggle Configuration
export const UIToggleConfigSchema = z.record(UIToggleTypeSchema, UIToggleStateSchema)
export type UIToggleConfig = z.infer<typeof UIToggleConfigSchema>

// Default UI Toggle Configuration
export const defaultUIToggleConfig: UIToggleConfig = {
  'combat-log': {
    id: 'combat-log',
    enabled: true,
    label: 'Combat Log',
    description: 'Show detailed combat event log',
    defaultValue: true,
    category: 'display'
  },
  'hp-bars': {
    id: 'hp-bars',
    enabled: false,
    label: 'Round-by-Round HP Bars',
    description: 'Show HP changes for each round of combat',
    defaultValue: false,
    category: 'display'
  },
  'quintile-1': {
    id: 'quintile-1',
    enabled: true,
    label: 'Decile 1 (Worst Case)',
    defaultValue: true,
    category: 'analysis'
  },
  'quintile-2': {
    id: 'quintile-2',
    enabled: false,
    label: 'Decile 2',
    defaultValue: false,
    category: 'analysis'
  },
  'quintile-3': {
    id: 'quintile-3',
    enabled: true,
    label: 'Decile 3 (Struggle)',
    defaultValue: true,
    category: 'analysis'
  },
  'quintile-4': {
    id: 'quintile-4',
    enabled: false,
    label: 'Decile 4',
    defaultValue: false,
    category: 'analysis'
  },
  'quintile-5': {
    id: 'quintile-5',
    enabled: true,
    label: 'Decile 5 (Typical Case)',
    defaultValue: true,
    category: 'analysis'
  },
  'quintile-6': {
    id: 'quintile-6',
    enabled: false,
    label: 'Decile 6',
    defaultValue: false,
    category: 'analysis'
  },
  'quintile-7': {
    id: 'quintile-7',
    enabled: false,
    label: 'Decile 7',
    defaultValue: false,
    category: 'analysis'
  },
  'quintile-8': {
    id: 'quintile-8',
    enabled: true,
    label: 'Decile 8 (Heroic)',
    defaultValue: true,
    category: 'analysis'
  },
  'quintile-9': {
    id: 'quintile-9',
    enabled: false,
    label: 'Decile 9',
    defaultValue: false,
    category: 'analysis'
  },
  'quintile-10': {
    id: 'quintile-10',
    enabled: true,
    label: 'Decile 10 (Best Case)',
    defaultValue: true,
    category: 'analysis'
  },
}

// UI Toggle Context
export const UIToggleContext = createContext<{
  toggles: UIToggleConfig
  updateToggle: (id: UIToggleType, enabled: boolean) => void
  resetToDefaults: () => void
  getToggleState: (id: UIToggleType) => boolean
  getTogglesByCategory: (category: 'display' | 'analysis') => UIToggleState[]
} | null>(null)

// UI Toggle Provider Props
export const UIToggleProviderPropsSchema = z.object({
  children: z.custom<ReactNode>()
})

export type UIToggleProviderProps = z.infer<typeof UIToggleProviderPropsSchema>

/**
 * UIToggleProvider - Centralized UI toggle state management provider
 *
 * Features:
 * - Manages UI toggle states with persistence to localStorage
 * - Integrates with LoadingManager for state change tracking
 * - Provides hooks for accessing and updating toggle states
 * - Includes error handling and validation
 * - Supports default value restoration
 */
export const UIToggleProvider: FC<UIToggleProviderProps> = ({ children }) => {
  const [toggles, setToggles] = useState<UIToggleConfig>(defaultUIToggleConfig)
  const [isInitialized, setIsInitialized] = useState(false)

  // Load toggle states from localStorage on mount
  useEffect(() => {
    const loadToggleStates = async () => {
      const operationId = loadingManager.startOperation('core-ui', 'low', 'Loading UI toggle states')

      try {
        if (typeof window !== 'undefined' && localStorage) {
          const stored = localStorage.getItem('ui-toggle-states')
          if (stored) {
            const parsed = JSON.parse(stored)
            const validated = UIToggleConfigSchema.safeParse(parsed)

            if (validated.success) {
              setToggles(validated.data)
            } else {
              console.warn('Invalid UI toggle state in localStorage, using defaults:', validated.error)
              // Reset to defaults if stored data is invalid
              localStorage.removeItem('ui-toggle-states')
            }
          }
        }
      } catch (error) {
        console.error('Error loading UI toggle states:', error)
      } finally {
        loadingManager.completeOperation(operationId)
        setIsInitialized(true)
      }
    }

    loadToggleStates()
  }, [])

  // Save toggle states to localStorage whenever they change
  useEffect(() => {
    if (!isInitialized) return

    const saveToggleStates = async () => {
      const operationId = loadingManager.startOperation('core-ui', 'low', 'Saving UI toggle states')

      try {
        if (typeof window !== 'undefined' && localStorage) {
          localStorage.setItem('ui-toggle-states', JSON.stringify(toggles))
        }
      } catch (error) {
        console.error('Error saving UI toggle states:', error)
      } finally {
        loadingManager.completeOperation(operationId)
      }
    }

    saveToggleStates()
  }, [toggles, isInitialized])

  const updateToggle = (id: UIToggleType, enabled: boolean) => {
    setToggles(prev => ({
      ...prev,
      [id]: {
        ...prev[id],
        enabled
      }
    }))
  }

  const resetToDefaults = () => {
    setToggles(defaultUIToggleConfig)
  }

  const getToggleState = (id: UIToggleType): boolean => {
    return toggles[id]?.enabled ?? defaultUIToggleConfig[id].enabled
  }

  const getTogglesByCategory = (category: 'display' | 'analysis'): UIToggleState[] => {
    return Object.values(toggles).filter(toggle => toggle.category === category)
  }

  const contextValue = {
    toggles,
    updateToggle,
    resetToDefaults,
    getToggleState,
    getTogglesByCategory
  }

  return (
    <UIToggleContext.Provider value={contextValue}>
      {children}
    </UIToggleContext.Provider>
  )
}

/**
 * useUIToggles - Hook for accessing UI toggle state and controls
 *
 * @returns Object containing toggle state and control functions
 */
export const useUIToggles = () => {
  const context = useContext(UIToggleContext)

  if (!context) {
    throw new Error('useUIToggles must be used within a UIToggleProvider')
  }

  return context
}

/**
 * useUIToggle - Hook for accessing a specific UI toggle state
 *
 * @param id - The toggle ID to access
 * @returns Tuple of [enabled, setter] similar to useState
 */
export const useUIToggle = (id: UIToggleType) => {
  const { getToggleState, updateToggle } = useUIToggles()

  const enabled = getToggleState(id)
  const setEnabled = (enabled: boolean) => updateToggle(id, enabled)

  return [enabled, setEnabled] as const
}