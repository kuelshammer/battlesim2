/**
 * Shared utilities for Skyline Canvas components
 *
 * This module provides common functionality for all skyline canvas visualizations:
 * - High-DPI (Retina) scaling setup
 * - Grid and axis drawing
 * - Hover and crosshair rendering
 * - Mouse event handling factories
 * - Placeholder rendering
 *
 * @module skylineCanvasUtils
 */

import type { ColorScale } from '@/model/skylineTypes';
import { valueToColor } from '@/model/skylineTypes';

// ============================================================================
// Types
// ============================================================================

/**
 * Canvas dimensions including DPI-scaled values
 */
export interface CanvasSize {
    /** Display width in CSS pixels */
    displayWidth: number;
    /** Display height in CSS pixels */
    displayHeight: number;
    /** Actual canvas width scaled for DPI */
    canvasWidth: number;
    /** Actual canvas height scaled for DPI */
    canvasHeight: number;
    /** Device pixel ratio (1 = standard, 2 = Retina, etc.) */
    pixelRatio: number;
}

/**
 * Padding configuration for chart layout
 */
export interface ChartPadding {
    top: number;
    right: number;
    bottom: number;
    left: number;
}

/**
 * Chart dimensions calculated from width, height, and padding
 */
export interface ChartDimensions {
    width: number;
    height: number;
    padding: ChartPadding;
    chartWidth: number;
    chartHeight: number;
}

/**
 * Mouse event handler configuration
 */
export interface MouseHandlerConfig {
    width: number;
    padding: ChartPadding;
    bucketCount: number;
}

/**
 * Result of bucket calculation from mouse position
 */
export interface BucketCalculation {
    bucketIndex: number; // 1-based index
    isValid: boolean;
}

/**
 * Configuration for drawing grids
 */
export interface GridDrawConfig {
    color?: string;
    lineWidth?: number;
    percentages?: number[]; // Y-axis positions (0, 25, 50, 75, 100)
    showLabels?: boolean;
    labelColor?: string;
    labelFont?: string;
}

/**
 * Configuration for drawing crosshairs
 */
export interface CrosshairDrawConfig {
    color?: string;
    lineWidth?: number;
    dashed?: boolean;
}

// ============================================================================
// DPI and Canvas Setup
// ============================================================================

/**
 * Get high-DPI scaled canvas dimensions
 *
 * @param canvas - The canvas element
 * @param width - Desired display width in CSS pixels
 * @param height - Desired display height in CSS pixels
 * @returns Canvas size information including DPI-scaled dimensions
 */
export function getCanvasSize(
    canvas: HTMLCanvasElement,
    width: number,
    height: number
): CanvasSize {
    const dpr = window.devicePixelRatio || 1;

    return {
        displayWidth: width,
        displayHeight: height,
        canvasWidth: width * dpr,
        canvasHeight: height * dpr,
        pixelRatio: dpr,
    };
}

/**
 * Setup canvas with high-DPI scaling
 *
 * Sets the canvas display size (CSS) and actual size (scaled for DPI),
 * then scales the context to normalize the coordinate system.
 *
 * @param canvas - The canvas element to setup
 * @param width - Desired display width in CSS pixels
 * @param height - Desired display height in CSS pixels
 * @returns The 2D rendering context (or null if unavailable)
 */
export function setupCanvas(
    canvas: HTMLCanvasElement,
    width: number,
    height: number
): CanvasRenderingContext2D | null {
    const dpr = window.devicePixelRatio || 1;

    // Set display size (CSS pixels)
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;

    // Set actual canvas size (scaled for DPI)
    canvas.width = width * dpr;
    canvas.height = height * dpr;

    // Normalize coordinate system so drawing uses CSS pixels
    const ctx = canvas.getContext('2d');
    if (ctx) {
        ctx.scale(dpr, dpr);
    }

    return ctx;
}

/**
 * Calculate chart dimensions from width, height, and padding
 *
 * @param width - Total canvas width
 * @param height - Total canvas height
 * @param padding - Padding configuration
 * @returns Chart dimensions including usable width/height
 */
export function getChartDimensions(
    width: number,
    height: number,
    padding: ChartPadding
): ChartDimensions {
    return {
        width,
        height,
        padding,
        chartWidth: width - padding.left - padding.right,
        chartHeight: height - padding.top - padding.bottom,
    };
}

// ============================================================================
// Drawing Utilities
// ============================================================================

/**
 * Clear canvas with background color
 *
 * @param ctx - Canvas rendering context
 * @param width - Canvas width
 * @param height - Canvas height
 * @param backgroundColor - Background fill color (default: dark semi-transparent)
 */
export function clearCanvas(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    backgroundColor: string = 'rgba(26, 26, 26, 0.95)'
): void {
    ctx.fillStyle = backgroundColor;
    ctx.fillRect(0, 0, width, height);
}

/**
 * Draw horizontal grid lines with optional labels
 *
 * @param ctx - Canvas rendering context
 * @param dims - Chart dimensions
 * @param config - Grid drawing configuration
 */
export function drawGridLines(
    ctx: CanvasRenderingContext2D,
    dims: ChartDimensions,
    config: GridDrawConfig = {}
): void {
    const {
        color = 'rgba(255, 255, 255, 0.1)',
        lineWidth = 1,
        percentages = [0, 25, 50, 75, 100],
        showLabels = true,
        labelColor = 'rgba(255, 255, 255, 0.6)',
        labelFont = '10px Courier New',
    } = config;

    ctx.strokeStyle = color;
    ctx.lineWidth = lineWidth;

    percentages.forEach((pct) => {
        const y = dims.padding.top + dims.chartHeight - (pct / 100) * dims.chartHeight;

        // Draw grid line
        ctx.beginPath();
        ctx.moveTo(dims.padding.left, y);
        ctx.lineTo(dims.width - dims.padding.right, y);
        ctx.stroke();

        // Draw Y-axis label
        if (showLabels) {
            ctx.fillStyle = labelColor;
            ctx.font = labelFont;
            ctx.textAlign = 'right';
            ctx.fillText(`${pct}%`, dims.padding.left - 5, y + 3);
        }
    });
}

/**
 * Draw X-axis percentile labels
 *
 * @param ctx - Canvas rendering context
 * @param dims - Chart dimensions
 * @param bucketCount - Number of buckets (determines label positions)
 * @param percentages - Percentile positions to label (default: [1, 50, 100])
 * @param labelColor - Label text color
 * @param labelFont - Label font specification
 */
export function drawXAxisLabels(
    ctx: CanvasRenderingContext2D,
    dims: ChartDimensions,
    bucketCount: number,
    percentages: number[] = [1, 50, 100],
    labelColor: string = 'rgba(255, 255, 255, 0.6)',
    labelFont: string = '10px Courier New'
): void {
    const bucketWidth = dims.chartWidth / bucketCount;

    ctx.fillStyle = labelColor;
    ctx.font = labelFont;
    ctx.textAlign = 'center';

    percentages.forEach((pct) => {
        const idx = pct - 1; // Convert to 0-based index
        if (idx < bucketCount) {
            const x = dims.padding.left + idx * bucketWidth + bucketWidth / 2;
            ctx.fillText(`${pct}%`, x, dims.height - dims.padding.bottom + 15);
        }
    });
}

/**
 * Draw crosshair line at hovered bucket
 *
 * @param ctx - Canvas rendering context
 * @param dims - Chart dimensions
 * @param bucketIndex - 1-based bucket index (1-100)
 * @param bucketCount - Total number of buckets
 * @param config - Crosshair drawing configuration
 */
export function drawCrosshair(
    ctx: CanvasRenderingContext2D,
    dims: ChartDimensions,
    bucketIndex: number,
    bucketCount: number,
    config: CrosshairDrawConfig = {}
): void {
    const { color = 'rgba(212, 175, 55, 0.8)', lineWidth = 1, dashed = true } = config;

    if (bucketIndex < 1 || bucketIndex > bucketCount) return;

    const bucketWidth = dims.chartWidth / bucketCount;
    const x = dims.padding.left + (bucketIndex - 1) * bucketWidth + bucketWidth / 2;

    ctx.strokeStyle = color;
    ctx.lineWidth = lineWidth;

    if (dashed) {
        ctx.setLineDash([4, 4]);
    }

    ctx.beginPath();
    ctx.moveTo(x, dims.padding.top);
    ctx.lineTo(x, dims.height - dims.padding.bottom);
    ctx.stroke();

    ctx.setLineDash([]); // Reset dash
}

/**
 * Draw highlight rectangle for hovered bucket
 *
 * @param ctx - Canvas rendering context
 * @param dims - Chart dimensions
 * @param bucketIndex - 1-based bucket index (1-100)
 * @param bucketCount - Total number of buckets
 * @param valuePercent - Value percentage (0-100) for height calculation
 * @param color - Highlight color
 * @param lineWidth - Border width
 */
export function drawBucketHighlight(
    ctx: CanvasRenderingContext2D,
    dims: ChartDimensions,
    bucketIndex: number,
    bucketCount: number,
    valuePercent: number,
    color: string = 'rgba(212, 175, 55, 0.8)',
    lineWidth: number = 2
): void {
    if (bucketIndex < 1 || bucketIndex > bucketCount) return;

    const bucketWidth = dims.chartWidth / bucketCount;
    const x = dims.padding.left + (bucketIndex - 1) * bucketWidth;
    const valueHeight = (valuePercent / 100) * dims.chartHeight;
    const y = dims.padding.top + dims.chartHeight - valueHeight;

    ctx.strokeStyle = color;
    ctx.lineWidth = lineWidth;
    ctx.strokeRect(x, y, bucketWidth - 1, valueHeight);
}

/**
 * Draw area fill under a data series
 *
 * @param ctx - Canvas rendering context
 * @param dims - Chart dimensions
 * @param dataPoints - Array of percentage values (0-100)
 * @param color - Fill color (with alpha if desired)
 */
export function drawAreaFill(
    ctx: CanvasRenderingContext2D,
    dims: ChartDimensions,
    dataPoints: number[],
    color: string
): void {
    if (dataPoints.length === 0) return;

    const bucketWidth = dims.chartWidth / dataPoints.length;

    ctx.beginPath();
    ctx.moveTo(dims.padding.left, dims.padding.top + dims.chartHeight);

    dataPoints.forEach((value, i) => {
        const x = dims.padding.left + i * bucketWidth;
        const y = dims.padding.top + dims.chartHeight - (value / 100) * dims.chartHeight;

        if (i === 0) {
            ctx.moveTo(x, y);
        } else {
            ctx.lineTo(x, y);
        }
    });

    const lastX = dims.padding.left + (dataPoints.length - 1) * bucketWidth;
    ctx.lineTo(lastX, dims.padding.top + dims.chartHeight);
    ctx.closePath();

    ctx.fillStyle = color;
    ctx.fill();
}

/**
 * Draw vertical bars for a data series with color mapping
 *
 * @param ctx - Canvas rendering context
 * @param dims - Chart dimensions
 * @param dataPoints - Array of percentage values (0-100)
 * @param colorScale - Color scale for value-to-color mapping
 */
export function drawVerticalBars(
    ctx: CanvasRenderingContext2D,
    dims: ChartDimensions,
    dataPoints: number[],
    colorScale: ColorScale
): void {
    const bucketWidth = dims.chartWidth / dataPoints.length;

    dataPoints.forEach((value, i) => {
        const x = dims.padding.left + i * bucketWidth;
        const barHeight = (value / 100) * dims.chartHeight;
        const y = dims.padding.top + dims.chartHeight - barHeight;

        const color = valueToColor(value, colorScale);

        ctx.fillStyle = color;
        ctx.fillRect(x, y, bucketWidth - 1, barHeight);
    });
}

// ============================================================================
// Placeholder Rendering
// ============================================================================

/**
 * Draw placeholder text when no data is available
 *
 * @param ctx - Canvas rendering context
 * @param width - Canvas width
 * @param height - Canvas height
 * @param text - Placeholder text to display
 * @param color - Text color
 * @param font - Font specification
 */
export function drawPlaceholder(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    text: string,
    color: string = 'rgba(212, 175, 55, 0.5)',
    font: string = '14px Courier New'
): void {
    ctx.fillStyle = color;
    ctx.font = font;
    ctx.textAlign = 'center';
    ctx.fillText(text, width / 2, height / 2);
}

/**
 * Draw "No data available" placeholder with custom message
 *
 * @param ctx - Canvas rendering context
 * @param width - Canvas width
 * @param height - Canvas height
 * @param message - Optional custom message
 */
export function drawNoDataPlaceholder(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    message?: string
): void {
    const text = message || 'No data available';
    drawPlaceholder(ctx, width, height, text);
}

// ============================================================================
// Mouse Event Handler Factories
// ============================================================================

/**
 * Calculate bucket index from mouse X position
 *
 * @param mouseX - Mouse X position within canvas
 * @param config - Handler configuration
 * @returns Bucket calculation result
 */
export function calculateBucketFromMouseX(
    mouseX: number,
    config: MouseHandlerConfig
): BucketCalculation {
    const chartWidth = config.width - config.padding.left - config.padding.right;
    const bucketWidth = chartWidth / config.bucketCount;
    const bucketIndex = Math.floor((mouseX - config.padding.left) / bucketWidth) + 1;

    return {
        bucketIndex,
        isValid: bucketIndex >= 1 && bucketIndex <= config.bucketCount,
    };
}

/**
 * Create a mouse move handler for bucket hover
 *
 * @param config - Handler configuration
 * @param onHover - Callback when hover state changes
 * @returns Mouse event handler function
 */
export function createMouseMoveHandler(
    config: MouseHandlerConfig,
    onHover?: (bucketIndex: number | null) => void
): (event: React.MouseEvent<HTMLCanvasElement>) => void {
    return (e: React.MouseEvent<HTMLCanvasElement>) => {
        const rect = e.currentTarget.getBoundingClientRect();
        const x = e.clientX - rect.left;

        const { bucketIndex, isValid } = calculateBucketFromMouseX(x, config);

        if (isValid) {
            onHover?.(bucketIndex);
        } else {
            onHover?.(null);
        }
    };
}

/**
 * Create a mouse leave handler
 *
 * @param onHover - Callback when hover state changes
 * @returns Mouse event handler function
 */
export function createMouseLeaveHandler(
    onHover?: (bucketIndex: number | null) => void
): () => void {
    return () => {
        onHover?.(null);
    };
}

// ============================================================================
// ARIA and Accessibility Utilities
// ============================================================================

/**
 * Generate ARIA label for a skyline canvas
 *
 * @param title - Skyline type title (e.g., "HP", "Resource")
 * @param bucketCount - Number of percentile buckets
 * @param characterName - Optional character name for single-character views
 * @returns ARIA label string
 */
export function generateAriaLabel(
    title: string,
    bucketCount: number,
    characterName?: string
): string {
    const base = `${title} skyline across ${bucketCount} percentile buckets`;
    return characterName ? `${characterName} ${base}` : base;
}

/**
 * Generate ARIA label for full analysis view
 *
 * @param title - Skyline type title
 * @param totalRuns - Total simulation runs
 * @param bucketCount - Number of buckets
 * @returns ARIA label string
 */
export function generateAnalysisAriaLabel(
    title: string,
    totalRuns: number,
    bucketCount: number
): string {
    return `${title} Skyline showing ${totalRuns} simulation runs across ${bucketCount} percentile buckets`;
}

// ============================================================================
// Animation Utilities
// ============================================================================

/**
 * Easing function for smooth transitions (ease-out)
 *
 * @param progress - Progress value (0-1)
 * @returns Eased value (0-1)
 */
export function easeOutQuad(progress: number): number {
    return progress * (2 - progress);
}

/**
 * Interpolate between two numbers
 *
 * @param start - Starting value
 * @param end - Ending value
 * @param progress - Progress (0-1)
 * @returns Interpolated value
 */
export function lerp(start: number, end: number, progress: number): number {
    return start + (end - start) * progress;
}

// ============================================================================
// Constants
// ============================================================================

/**
 * Default padding configuration for skyline charts
 */
export const DEFAULT_PADDING: ChartPadding = {
    top: 20,
    right: 10,
    bottom: 30,
    left: 40,
};

/**
 * Default grid draw configuration
 */
export const DEFAULT_GRID_CONFIG: GridDrawConfig = {
    color: 'rgba(255, 255, 255, 0.1)',
    lineWidth: 1,
    percentages: [0, 25, 50, 75, 100],
    showLabels: true,
    labelColor: 'rgba(255, 255, 255, 0.6)',
    labelFont: '10px Courier New',
};

/**
 * Default crosshair configuration
 */
export const DEFAULT_CROSSHAIR_CONFIG: CrosshairDrawConfig = {
    color: 'rgba(212, 175, 55, 0.8)',
    lineWidth: 1,
    dashed: true,
};

/**
 * Default highlight color for bucket selection
 */
export const DEFAULT_HIGHLIGHT_COLOR = 'rgba(212, 175, 55, 0.8)';

/**
 * Default background color for canvas
 */
export const DEFAULT_BACKGROUND_COLOR = 'rgba(26, 26, 26, 0.95)';
