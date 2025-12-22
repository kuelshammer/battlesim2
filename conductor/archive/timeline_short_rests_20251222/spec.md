# Spec: Timeline-Based Short Rests

## Overview
Short rests are currently a secondary property of an encounter ("Short Rest After"). This refactoring promotes Short Rests to first-class citizens in the Adventuring Day timeline.

## Key Changes
1. **Discrete Events:** Users can now add "Short Rest" events to their adventuring day, just like encounters.
2. **Sequential Simulation:** The engine processes the timeline in order. A short rest event triggers recovery logic between combats.
3. **Refined Scoring:**
    - Combat encounters are scored purely on their own performance (Resources spent during the fight).
    - Short rests are scored based on **Hit Die consumption** (15 points per HD).
    - This allows for precise identification of which fight was the most "expensive."

## Acceptance Criteria
- Adventuring Day Editor shows a linear list of Encounters and Short Rests.
- Short Rests can be reordered or deleted.
- Simulation correctly applies recovery when a Short Rest event is encountered.
- Hit Die usage is visible in the performance metrics and affects the overall efficiency score.
