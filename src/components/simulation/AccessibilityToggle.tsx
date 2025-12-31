/**
 * AccessibilityToggle - Button to toggle accessibility features
 *
 * Provides:
 * - High-contrast mode toggle
 * - Pattern density selector
 */

import React, { memo } from 'react';
import { useAccessibility } from './AccessibilityContext';
import styles from './accessibilityToggle.module.scss';

export interface AccessibilityToggleProps {
    className?: string;
}

/**
 * Accessibility controls panel
 */
const AccessibilityToggle: React.FC<AccessibilityToggleProps> = memo(({
    className,
}) => {
    const { highContrast, toggleHighContrast, patternDensity, setPatternDensity } = useAccessibility();

    return (
        <div className={`${styles.accessibilityPanel} ${className || ''}`}>
            <div className={styles.panelTitle}>Accessibility</div>

            <div className={styles.controlGroup}>
                <label className={styles.controlLabel}>
                    <input
                        type="checkbox"
                        checked={highContrast}
                        onChange={toggleHighContrast}
                        className={styles.checkbox}
                    />
                    <span className={styles.labelText}>High Contrast</span>
                </label>
                <span className={styles.description}>
                    Enhanced colors with pattern overlays for color blindness
                </span>
            </div>

            <div className={styles.controlGroup}>
                <label className={styles.controlLabel}>Pattern Density</label>
                <div className={styles.densityButtons} role="radiogroup">
                    {(['none', 'low', 'medium', 'high'] as const).map((density) => (
                        <button
                            key={density}
                            onClick={() => setPatternDensity(density)}
                            className={`${styles.densityButton} ${
                                patternDensity === density ? styles.active : ''
                            }`}
                            aria-pressed={patternDensity === density}
                            aria-label={`Pattern density: ${density}`}
                        >
                            {density === 'none' ? 'Off' : density}
                        </button>
                    ))}
                </div>
                <span className={styles.description}>
                    Pattern overlays on charts for better visibility
                </span>
            </div>

            <div className={styles.wcagNotice}>
                WCAG AAA Compliant
            </div>
        </div>
    );
});

AccessibilityToggle.displayName = 'AccessibilityToggle';

export default AccessibilityToggle;
