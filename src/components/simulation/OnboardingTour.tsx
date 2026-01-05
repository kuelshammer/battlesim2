/**
 * Onboarding Tour Component
 *
 * Provides a guided tour for first-time users using a custom implementation.
 * Tracks completion status in localStorage.
 */

import React, { FC, useState, useCallback, useEffect, useRef } from 'react'

const TOUR_STORAGE_KEY = 'battlesim_tour_completed'

export interface OnboardingTourProps {
  /**
   * Set to true to force the tour to show even if completed before
   */
  forceRun?: boolean

  /**
   * Callback when tour is completed or skipped
   */
  onTourEnd?: () => void
}

/**
 * Check if the tour has been completed
 */
export function hasCompletedTour(): boolean {
  if (typeof window === 'undefined') return false
  try {
    return localStorage.getItem(TOUR_STORAGE_KEY) === 'true'
  } catch {
    return false
  }
}

/**
 * Mark the tour as completed
 */
export function markTourCompleted(): void {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(TOUR_STORAGE_KEY, 'true')
  } catch {
    // Silently fail if localStorage is not available
  }
}

/**
 * Reset tour completion status (useful for testing)
 */
export function resetTour(): void {
  if (typeof window === 'undefined') return
  try {
    localStorage.removeItem(TOUR_STORAGE_KEY)
  } catch {
    // Silently fail if localStorage is not available
  }
}

/**
 * Tour steps highlighting key features of the app
 */
const tourSteps = [
  {
    target: '.encounter-builder-section',
    title: 'Welcome to Battle Sim! 🎲',
    content: (
      <div>
        <p>
          This tool helps you build balanced D&D 5e encounters and simulate them
          to see how they play out.
        </p>
        <p>
          <strong>This tour takes 2 minutes.</strong> Let's get started!
        </p>
      </div>
    ),
  },
  {
    target: '.player-form-section',
    title: 'Add Your Party',
    content: (
      <div>
        <p>
          Start by adding your player characters. You can create custom heroes
          or use pre-built class templates from the SRD.
        </p>
        <ul>
          <li>Click the + button to add players</li>
          <li>Adjust HP, AC, and abilities</li>
          <li>Configure actions and spells</li>
        </ul>
      </div>
    ),
  },
  {
    target: '.monster-form-section',
    title: 'Add Monsters',
    content: (
      <div>
        <p>
          Add monsters to fight against. The simulator includes hundreds of
          monsters from the D&D 5e SRD.
        </p>
        <ul>
          <li>Search by name or CR</li>
          <li>Add multiple monsters</li>
          <li>Set surprise rounds</li>
        </ul>
      </div>
    ),
  },
  {
    target: '.simulation-controls',
    title: 'Automatic Simulations',
    content: (
      <div>
        <p>
          Simulations start <strong>automatically</strong> as you make changes.
        </p>
        <p>
          The engine provides instant feedback and then <strong>progressively refines</strong> accuracy in the background. 
          Use <strong>"High Precision Mode"</strong> for deep decile analysis.
        </p>
      </div>
    ),
  },
  {
    target: '.decile-chart-section',
    title: 'Decile Analysis 📊',
    content: (
      <div>
        <p>
          This powerful feature shows outcomes across all probability bands:
        </p>
        <ul>
          <li><strong>Worst 10%:</strong> Everything goes wrong</li>
          <li><strong>Median (50%):</strong> Typical outcome</li>
          <li><strong>Best 10%:</strong> Lucky rolls and tactics</li>
        </ul>
        <p>
          Click on any decile to see exactly how that combat played out!
        </p>
      </div>
    ),
  },
  {
    target: '.event-log-section',
    title: 'Event Log 📜',
    content: (
      <div>
        <p>
          Dive deep into combat details with the event log. See every attack,
          damage roll, save, and ability use.
        </p>
        <p>
          Perfect for understanding why a simulation played out the way it did!
        </p>
      </div>
    ),
  },
  {
    target: '.auto-balancer-section',
    title: 'Auto-Balancer ⚖️',
    content: (
      <div>
        <p>
          The auto-balancer helps you fine-tune encounters. Use it to:
        </p>
        <ul>
          <li>Adjust monster difficulty up or down</li>
          <li>Target a specific win rate</li>
          <li>Find the perfect challenge for your party</li>
        </ul>
        <p>
          You're all set! Happy gaming! 🎮
        </p>
      </div>
    ),
  },
]

const OnboardingTour: FC<OnboardingTourProps> = ({ forceRun = false, onTourEnd }) => {
  const [currentStep, setCurrentStep] = useState(0)
  const [isVisible, setIsVisible] = useState(false)
  const [targetElement, setTargetElement] = useState<HTMLElement | null>(null)
  const position = useRef({ top: 0, left: 0 })

  // Initialize tour state
  useEffect(() => {
    const shouldRun = forceRun || !hasCompletedTour()
    setIsVisible(shouldRun)
    if (shouldRun) {
      // Disable body scroll
      document.body.style.overflow = 'hidden'
    }
    return () => {
      document.body.style.overflow = ''
    }
  }, [forceRun])

  // Find target element for current step
  useEffect(() => {
    if (!isVisible || currentStep >= tourSteps.length) return

    const step = tourSteps[currentStep]
    const element = document.querySelector(step.target) as HTMLElement | null

    if (element) {
      setTargetElement(element)
      const rect = element.getBoundingClientRect()
      const scrollTop = window.pageYOffset || document.documentElement.scrollTop
      const scrollLeft = window.pageXOffset || document.documentElement.scrollLeft

      position.current = {
        top: rect.top + scrollTop,
        left: rect.left + scrollLeft,
      }

      // Scroll element into view
      if (typeof element.scrollIntoView === 'function') {
        element.scrollIntoView({ behavior: 'smooth', block: 'center' })
      }

      // Highlight the element
      element.style.boxShadow = '0 0 0 4px rgba(124, 58, 237, 0.5), 0 0 20px rgba(124, 58, 237, 0.3)'
      element.style.zIndex = '10001'
      element.style.position = 'relative'

      return () => {
        element.style.boxShadow = ''
        element.style.zIndex = ''
      }
    } else {
      setTargetElement(null)
    }
  }, [isVisible, currentStep])

  const handleNext = useCallback(() => {
    if (currentStep < tourSteps.length - 1) {
      setCurrentStep(currentStep + 1)
    } else {
      handleFinish()
    }
  }, [currentStep])

  const handlePrevious = useCallback(() => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1)
    }
  }, [currentStep])

  const handleSkip = useCallback(() => {
    handleFinish()
  }, [])

  const handleFinish = useCallback(() => {
    setIsVisible(false)
    markTourCompleted()
    onTourEnd?.()
  }, [onTourEnd])

  if (!isVisible || currentStep >= tourSteps.length) {
    return null
  }

  const step = tourSteps[currentStep]

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        zIndex: 10000,
        pointerEvents: 'none',
      }}
    >
      {targetElement && (
        <div
          style={{
            position: 'absolute',
            top: `${Math.max(10, position.current.top - 180)}px`,
            left: `${Math.min(position.current.left, window.innerWidth - 420)}px`,
            maxWidth: '400px',
            backgroundColor: 'white',
            borderRadius: '12px',
            padding: '24px',
            boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
            pointerEvents: 'auto',
            zIndex: 10002,
          }}
        >
          <div style={{ marginBottom: '16px' }}>
            <h3
              style={{
                margin: '0 0 8px 0',
                fontSize: '18px',
                fontWeight: 'bold',
                color: '#1f2937',
              }}
            >
              {step.title}
            </h3>
            <div
              style={{
                fontSize: '14px',
                lineHeight: '1.5',
                color: '#4b5563',
              }}
            >
              {step.content}
            </div>
          </div>

          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              gap: '12px',
            }}
          >
            <button
              onClick={handleSkip}
              style={{
                padding: '8px 16px',
                border: 'none',
                background: 'transparent',
                color: '#6b7280',
                fontSize: '14px',
                cursor: 'pointer',
                fontWeight: 500,
              }}
              onMouseEnter={(e) => (e.currentTarget.style.color = '#374151')}
              onMouseLeave={(e) => (e.currentTarget.style.color = '#6b7280')}
            >
              Skip Tour
            </button>

            <div style={{ display: 'flex', gap: '8px' }}>
              {currentStep > 0 && (
                <button
                  onClick={handlePrevious}
                  style={{
                    padding: '8px 16px',
                    border: '1px solid #d1d5db',
                    background: 'white',
                    color: '#7c3aed',
                    borderRadius: '6px',
                    fontSize: '14px',
                    cursor: 'pointer',
                    fontWeight: 500,
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.borderColor = '#7c3aed')}
                  onMouseLeave={(e) => (e.currentTarget.style.borderColor = '#d1d5db')}
                >
                  Back
                </button>
              )}

              <button
                onClick={handleNext}
                style={{
                  padding: '8px 16px',
                  border: 'none',
                  background: '#7c3aed',
                  color: 'white',
                  borderRadius: '6px',
                  fontSize: '14px',
                  cursor: 'pointer',
                  fontWeight: 500,
                }}
                onMouseEnter={(e) => (e.currentTarget.style.background = '#6d28d9')}
                onMouseLeave={(e) => (e.currentTarget.style.background = '#7c3aed')}
              >
                {currentStep === tourSteps.length - 1 ? 'Finish' : 'Next'} ({currentStep + 1}/{tourSteps.length})
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default OnboardingTour
