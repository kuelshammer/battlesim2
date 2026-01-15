import React, { FC, useState } from 'react'
import { UIToggleState, useUIToggles, UIToggleID } from '@/model/uiToggleState'
import styles from './UiTogglePanel.module.scss'

type ToggleControlProps = {
  toggle: UIToggleState
  onToggle: (id: UIToggleID, enabled: boolean) => void
}

const ToggleControl: FC<ToggleControlProps> = ({ toggle, onToggle }) => {
  const [isFocused, setIsFocused] = useState(false)

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      onToggle(toggle.id, !toggle.enabled)
    }
  }

  return (
    <div className={`${styles.toggleControl} ${toggle.enabled ? styles.enabled : ''}`}>
      <label
        className={styles.toggleLabel}
        htmlFor={`toggle-${toggle.id}`}
        title={toggle.description}
      >
        <input
          id={`toggle-${toggle.id}`}
          type="checkbox"
          checked={toggle.enabled}
          onChange={(e) => onToggle(toggle.id, e.target.checked)}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
          onKeyDown={handleKeyDown}
          className={styles.toggleInput}
          aria-describedby={toggle.description ? `desc-${toggle.id}` : undefined}
        />
        <span
          className={`${styles.toggleSwitch} ${isFocused ? styles.focused : ''}`}
          role="switch"
          aria-checked={toggle.enabled}
          tabIndex={-1}
        >
          <span className={styles.toggleSlider} />
        </span>
        <span className={styles.toggleText}>
          {toggle.label}
        </span>
      </label>
      {toggle.description && (
        <div
          id={`desc-${toggle.id}`}
          className={styles.toggleDescription}
        >
          {toggle.description}
        </div>
      )}
    </div>
  )
}

type ToggleSectionProps = {
  title: string
  toggles: UIToggleState[]
  onToggle: (id: string, enabled: boolean) => void
}

const ToggleSection: FC<ToggleSectionProps> = ({ title, toggles, onToggle }) => {
  if (toggles.length === 0) return null

  return (
    <div className={styles.toggleSection}>
      <h4 className={styles.sectionTitle}>{title}</h4>
      <div className={styles.toggleGrid}>
        {toggles.map(toggle => (
          <ToggleControl
            key={toggle.id}
            toggle={toggle}
            onToggle={onToggle}
          />
        ))}
      </div>
    </div>
  )
}

type UiTogglePanelProps = {
  isOpen?: boolean
  onClose?: () => void
  className?: string
}

const UiTogglePanel: FC<UiTogglePanelProps> = ({
  isOpen = true,
  onClose,
  className = ''
}) => {
  const { getTogglesByCategory, updateToggle, resetToDefaults } = useUIToggles()
  const [isExpanded, setIsExpanded] = useState(isOpen)

  const displayToggles = getTogglesByCategory('display')
  const analysisToggles = getTogglesByCategory('analysis')

  const handleToggle = (id: string, enabled: boolean) => {
    updateToggle(id as UIToggleID, enabled)
  }

  const handleReset = () => {
    if (window.confirm('Reset all UI toggles to their default values?')) {
      resetToDefaults()
    }
  }

  if (!isExpanded) {
    return (
      <div className={`${styles.togglePanelCollapsed} ${className}`}>
        <button
          onClick={() => setIsExpanded(true)}
          className={styles.expandButton}
          aria-label="Open UI controls panel"
          title="Open UI controls"
        >
          ⚙️
        </button>
      </div>
    )
  }

  return (
    <div className={`${styles.togglePanel} ${className}`}>
      <div className={styles.panelHeader}>
        <h3 className={styles.panelTitle}>UI Controls</h3>
        <div className={styles.panelActions}>
          <button
            onClick={handleReset}
            className={styles.resetButton}
            aria-label="Reset to defaults"
            title="Reset all toggles to default values"
          >
            🔄
          </button>
          {onClose && (
            <button
              onClick={() => {
                setIsExpanded(false)
                onClose()
              }}
              className={styles.closeButton}
              aria-label="Close panel"
              title="Close UI controls panel"
            >
              ✕
            </button>
          )}
          {!onClose && (
            <button
              onClick={() => setIsExpanded(false)}
              className={styles.collapseButton}
              aria-label="Collapse panel"
              title="Collapse UI controls panel"
            >
              −
            </button>
          )}
        </div>
      </div>

      <div className={styles.panelContent}>
        <ToggleSection
          title="Display Options"
          toggles={displayToggles}
          onToggle={handleToggle}
        />

        <ToggleSection
          title="Analysis Options"
          toggles={analysisToggles}
          onToggle={handleToggle}
        />
      </div>

      <div className={styles.panelFooter}>
        <p className={styles.footerText}>
          Changes are saved automatically
        </p>
      </div>
    </div>
  )
}

export default UiTogglePanel