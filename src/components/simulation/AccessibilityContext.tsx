/**
 * Accessibility context for Skyline components
 *
 * Provides:
 * - High-contrast mode toggle
 * - Pattern overlay rendering (diagonal stripes, dots)
 * - Screen reader announcements
 */

import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';

export interface AccessibilityContextValue {
    /** High-contrast mode enabled */
    highContrast: boolean;
    /** Toggle high-contrast mode */
    toggleHighContrast: () => void;
    /** Current pattern density */
    patternDensity: 'none' | 'low' | 'medium' | 'high';
    /** Set pattern density */
    setPatternDensity: (density: 'none' | 'low' | 'medium' | 'high') => void;
    /** Screen reader announcement */
    announce: (message: string) => void;
}

const AccessibilityContext = createContext<AccessibilityContextValue | undefined>(undefined);

export interface AccessibilityProviderProps {
    children: ReactNode;
    /** Initial high-contrast mode state */
    initialHighContrast?: boolean;
}

/**
 * Accessibility Provider for Skyline components
 */
export const AccessibilityProvider: React.FC<AccessibilityProviderProps> = memo(({
    children,
    initialHighContrast = false,
}) => {
    const [highContrast, setHighContrast] = useState(initialHighContrast);
    const [patternDensity, setPatternDensity] = useState<'none' | 'low' | 'medium' | 'high'>('medium');
    const [announcement, setAnnouncement] = useState('');

    const toggleHighContrast = useCallback(() => {
        setHighContrast(prev => !prev);
    }, []);

    const announce = useCallback((message: string) => {
        setAnnouncement(message);
        // Clear announcement after it's been read
        setTimeout(() => setAnnouncement(''), 1000);
    }, []);

    const value: AccessibilityContextValue = {
        highContrast,
        toggleHighContrast,
        patternDensity,
        setPatternDensity,
        announce,
    };

    return (
        <AccessibilityContext.Provider value={value}>
            {children}
            {/* Screen reader announcements */}
            <div
                role="status"
                aria-live="polite"
                aria-atomic="true"
                className="sr-only"
                style={{
                    position: 'absolute',
                    left: '-10000px',
                    width: '1px',
                    height: '1px',
                    overflow: 'hidden',
                }}
            >
                {announcement}
            </div>
        </AccessibilityContext.Provider>
    );
});

AccessibilityProvider.displayName = 'AccessibilityProvider';

/**
 * Hook to use accessibility context
 */
export function useAccessibility(): AccessibilityContextValue {
    const context = useContext(AccessibilityContext);
    if (!context) {
        throw new Error('useAccessibility must be used within AccessibilityProvider');
    }
    return context;
}

/**
 * Canvas pattern rendering utilities
 */
export class AccessibilityPatterns {
    /**
     * Create diagonal stripe pattern for high values
     */
    static createDiagonalStripes(
        ctx: CanvasRenderingContext2D,
        color: string,
        density: 'low' | 'medium' | 'high' = 'medium'
    ): CanvasPattern {
        const patternCanvas = document.createElement('canvas');
        const size = density === 'low' ? 12 : density === 'high' ? 6 : 8;
        patternCanvas.width = size;
        patternCanvas.height = size;
        const patternCtx = patternCanvas.getContext('2d');

        if (!patternCtx) {
            throw new Error('Failed to create pattern context');
        }

        // Transparent background
        patternCtx.clearRect(0, 0, size, size);

        // Draw diagonal stripes
        patternCtx.strokeStyle = color;
        patternCtx.lineWidth = 1;
        patternCtx.beginPath();
        patternCtx.moveTo(0, size);
        patternCtx.lineTo(size, 0);
        patternCtx.stroke();

        return ctx.createPattern(patternCanvas, 'repeat') || ctx.createPattern(patternCanvas, 'repeat-x')!;
    }

    /**
     * Create dot pattern for mid-range values
     */
    static createDots(
        ctx: CanvasRenderingContext2D,
        color: string,
        density: 'low' | 'medium' | 'high' = 'medium'
    ): CanvasPattern {
        const patternCanvas = document.createElement('canvas');
        const size = density === 'low' ? 10 : density === 'high' ? 5 : 7;
        patternCanvas.width = size;
        patternCanvas.height = size;
        const patternCtx = patternCanvas.getContext('2d');

        if (!patternCtx) {
            throw new Error('Failed to create pattern context');
        }

        patternCtx.clearRect(0, 0, size, size);

        // Draw dot in center
        patternCtx.fillStyle = color;
        patternCtx.beginPath();
        patternCtx.arc(size / 2, size / 2, 1, 0, Math.PI * 2);
        patternCtx.fill();

        return ctx.createPattern(patternCanvas, 'repeat') || ctx.createPattern(patternCanvas, 'repeat-x')!;
    }

    /**
     * Create crosshatch pattern for critical/dead values
     */
    static createCrosshatch(
        ctx: CanvasRenderingContext2D,
        color: string,
        density: 'low' | 'medium' | 'high' = 'medium'
    ): CanvasPattern {
        const patternCanvas = document.createElement('canvas');
        const size = density === 'low' ? 14 : density === 'high' ? 7 : 10;
        patternCanvas.width = size;
        patternCanvas.height = size;
        const patternCtx = patternCanvas.getContext('2d');

        if (!patternCtx) {
            throw new Error('Failed to create pattern context');
        }

        patternCtx.clearRect(0, 0, size, size);

        // Draw X pattern
        patternCtx.strokeStyle = color;
        patternCtx.lineWidth = 1;

        // Diagonal 1
        patternCtx.beginPath();
        patternCtx.moveTo(0, 0);
        patternCtx.lineTo(size, size);
        patternCtx.stroke();

        // Diagonal 2
        patternCtx.beginPath();
        patternCtx.moveTo(size, 0);
        patternCtx.lineTo(0, size);
        patternCtx.stroke();

        return ctx.createPattern(patternCanvas, 'repeat') || ctx.createPattern(patternCanvas, 'repeat-x')!;
    }

    /**
     * Apply pattern overlay based on value
     */
    static applyPatternOverlay(
        ctx: CanvasRenderingContext2D,
        x: number,
        y: number,
        width: number,
        height: number,
        value: number, // 0-100
        highContrast: boolean,
        patternDensity: 'none' | 'low' | 'medium' | 'high'
    ): void {
        if (!highContrast || patternDensity === 'none') return;

        let pattern: CanvasPattern | null = null;
        const density = patternDensity === 'low' || patternDensity === 'high' ? patternDensity : 'medium';

        if (value >= 75) {
            // High values - diagonal stripes
            pattern = this.createDiagonalStripes(ctx, '#ffffff', density);
        } else if (value >= 40 && value < 75) {
            // Mid values - dots
            pattern = this.createDots(ctx, '#ffffff', density);
        } else if (value < 20) {
            // Critical/dead values - crosshatch
            pattern = this.createCrosshatch(ctx, '#ff6b6b', density);
        }

        if (pattern) {
            ctx.save();
            ctx.fillStyle = pattern;
            ctx.globalAlpha = 0.3;
            ctx.fillRect(x, y, width, height);
            ctx.restore();
        }
    }

    /**
     * Get high-contrast color (WCAG AAA compliant)
     */
    static getHighContrastColor(value: number, type: 'hp' | 'resources'): string {
        if (type === 'hp') {
            // HP: Red-Yellow-Green scale (high contrast)
            if (value >= 75) return '#00ff00'; // Bright green
            if (value >= 50) return '#ffff00'; // Yellow
            if (value >= 25) return '#ff9900'; // Orange
            return '#ff0000'; // Bright red
        } else {
            // Resources: Cyan-Blue-Purple scale
            if (value >= 75) return '#00ffff'; // Cyan
            if (value >= 50) return '#0099ff'; // Bright blue
            if (value >= 25) return '#9900ff'; // Purple
            return '#ff00ff'; // Magenta
        }
    }
}
