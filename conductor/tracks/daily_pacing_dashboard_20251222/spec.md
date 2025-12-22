# Spec: Daily Pacing Dashboard

## Overview
Implement a comprehensive dashboard for Game Masters to visualize resource attrition over an adventuring day. The dashboard will use a "Fuel Gauge" metaphor for macro pacing, "Delta Badges" for micro-encounter analysis, and a "Descent Graph" to visualize risk and trend.

## Functional Requirements

### 1. The 5-Zone Threshold System
All delta calculations (`Actual Cost % - Target Cost %`) will use the following logic:
*   **Major Under (< -10%):** 🔵 Blue (Undertuned)
*   **Minor Under (-10% to -5%):** 💠 Cyan (Slightly Easy)
*   **Perfect (-5% to +5%):** 🟢 Green (On Target)
*   **Minor Over (+5% to +10%):** 🟡 Yellow (Minor Drift)
*   **Major Over (> +10%):** 🔴 Red (Overtuned / Budget Hog)
*   **Implementation Note:** Logic uses exact floating-point comparisons; UI rounds to the nearest whole percentage.

### 2. Macro View: Daily Fuel Gauge
*   **Location:** Top of the Adventuring Day summary.
*   **Component:** Two stacked horizontal segmented bars.
    *   **Top Bar (The Plan):** Segments sized by `Weight / Total_Weights`.
    *   **Bottom Bar (The Reality):** Segments sized by actual median EHP cost.
    *   **Over-budget indicator:** If total cost > 100%, show a "Tank Empty" marker.

### 3. Micro View: Delta Badges
*   **Location:** Every Encounter Card in the simulation results.
*   **Content:**
    *   Current role weight and resulting target %.
    *   Simulated actual % drain.
    *   Color-coded status icon and text (e.g., "🔴 Over Budget (+12%)").
    *   Cumulative drift indicator (e.g., "Total Day Drift: +4%").

### 4. Descent Graph (SVG Attrition Chart)
*   **Implementation:** Custom SVG (no external libraries).
*   **X-Axis:** Timeline steps (Start $\to$ Enc 1 $\to$ Rest $\to$ Enc 2...).
*   **Y-Axis:** Party Net Worth % (100% down to 0%).
*   **Data Lines:**
    *   **Dotted Line (The Plan):** Straight-line slope between rest points based on target weights.
    *   **Solid Line (Simulation):** Median resource state after each step.
    *   **Shaded Region (The Risk):** Shaded area between the 25th percentile (Struggle) and 75th percentile (Heroic) runs.

### 5. Assistant Voice Summary
*   Generate a natural language assessment:
    *   **Green:** "✅ Balanced. The party is expected to reach the finale with [X]% resources."
    *   **Yellow/Cyan:** "⚠️ Minor Pacing Drift. [Encounter X] is harder than planned."
    *   **Red:** "⛔ Impossible Day. Party runs out of resources at [Encounter Y]."

## Technical Requirements
*   Update `AggregateOutput` to include per-step EHP data for all 10 deciles (needed for the shaded region and line graph).
*   Implement `PacingDashboard.tsx` and `DescentGraph.tsx` (SVG).
