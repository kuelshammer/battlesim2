# Spec: Intensity Representation Refinement

## Overview
Replace the "Star" rating (which implies quality) with a "Lightning" symbol (representing energy/resource drain). This better communicates that higher intensity means higher resource consumption, which is a descriptive metric rather than a purely qualitative one.

## Requirements
1.  **Symbol Change:** Replace `faStar` with `faBolt` (lightning bolt) in the UI.
2.  **Semantic Shift:** The number of lightning bolts represents "Intensity / Energy Needed".
3.  **Intensity Mapping (unchanged but re-labeled):**
    *   Tier 4: 3 Bolts (Heavy Energy)
    *   Tier 3 or 5: 2 Bolts (Moderate Energy)
    *   Tier 2: 1 Bolt (Light Energy)
    *   Tier 1: 0 Bolts (Trivial Energy)
4.  **Visual Update:** Use a color appropriate for lightning (e.g., bright yellow or electric blue) for the filled bolts.
