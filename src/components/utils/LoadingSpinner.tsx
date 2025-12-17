import React from 'react'
import styles from './LoadingSpinner.module.scss'

interface LoadingSpinnerProps {
  /** Size of the spinner */
  size?: 'small' | 'medium' | 'large'
  /** Color variant */
  variant?: 'primary' | 'secondary' | 'white'
  /** Whether to show a label */
  showLabel?: boolean
  /** Custom label text */
  label?: string
  /** Additional CSS classes */
  className?: string
  /** Whether the spinner is inline (doesn't take full container) */
  inline?: boolean
}

/**
 * LoadingSpinner - Reusable spinner component for individual component loading
 *
 * Provides various sizes and styles for different loading contexts.
 * Can be used inline or as a block element.
 */
export const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({
  size = 'medium',
  variant = 'primary',
  showLabel = false,
  label = 'Loading...',
  className = '',
  inline = false
}) => {
  const containerClasses = [
    styles.container,
    styles[size],
    styles[variant],
    inline ? styles.inline : styles.block,
    className
  ].filter(Boolean).join(' ')

  return (
    <div className={containerClasses} role="status" aria-live="polite">
      <div className={styles.spinner}>
        <div className={styles.spinnerRing}></div>
        <div className={styles.spinnerRing}></div>
        <div className={styles.spinnerRing}></div>
      </div>
      {showLabel && (
        <span className={styles.label}>
          {label}
        </span>
      )}
      {/* Screen reader text */}
      <span className={styles.srOnly}>Loading</span>
    </div>
  )
}

export default LoadingSpinner