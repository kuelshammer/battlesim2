# Spec: Efficiency-Aware Intensity Scoring

## Overview
Fix a critical bug in the intensity calculation where encounters are under-reported as "Tier 1" (Trivial) even when the party suffers significant HP and resource loss.

## Problem Statement
The current `IntensityTier` calculation uses a fragile reverse-engineering method to extract HP loss from the tiered combat score. 
When the `resource_penalty` (spent spell slots, Action Surges, etc.) is large enough, it can cause the `survivors` count to be under-calculated (via `floor()`), which in turn zeroes out the calculated `hp_lost` metric, leading to an incorrect "Tier 1" rating.

## Requirements
1.  **Robust Survivor Counting:** Calculate the number of survivors directly from the last round of the last encounter in the simulation result, rather than reverse-engineering it from the score.
2.  **Correct Attrition Formula:** Use a robust formula to calculate total attrition: `total_attrition = max_hp + (survivors * 1,000,000) - score`. This formula correctly captures HP loss, monster survival (as a penalty), and resource consumption.
3.  **Validate Tier Thresholds:** Ensure that HP-only loss (like the fighters dropping from 75 to 30) correctly maps to Tier 2 or Tier 3.