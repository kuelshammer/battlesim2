# Spec: Adventuring Day Evaluation Update

## Overview
Update the evaluation logic for adventuring days to be more nuanced and visually representative of the quality of the encounter sequence.

## Requirements
1.  **Safety Acceptance:**
    *   Grade A and Grade B are now accepted.
    *   Grade C or worse is rejected.
2.  **Visual Representation (Safety):**
    *   Grade A: Green
    *   Grade B: Orange
    *   Grade C or worse: Red
3.  **Intensity Rating (Stars):**
    *   Tier 4: 3 Stars (Ideal)
    *   Tier 3 or 5: 2 Stars
    *   Tier 2: 1 Star
    *   Tier 1: 0 Stars
4.  **Target Labeling:**
    *   The "A+++" rating should be the goal (Grade A + Tier 4).
5.  **Implementation Scope:**
    *   Update Rust backend logic for evaluation results.
    *   Update Frontend components (`DecileAnalysis.tsx` or similar) to reflect these changes.
