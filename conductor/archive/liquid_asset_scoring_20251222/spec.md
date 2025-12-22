# Spec: Liquid Asset Intensity Scoring (Budget Share Edition)

## Overview
Implement a "Liquid vs. Frozen" asset formula for Intensity calculation, using a "Total Daily Budget Share" approach. Instead of rating encounters against a static baseline, each encounter's resource "Cost" (the Delta) is measured against the **Total Daily Net Worth (TDNW)** and compared to a **Target Drain** percentage scaled by the number of encounters ($N$). 

## Functional Requirements
1.  **Effective HP (EHP) Weights:**
    *   1 HP / 1 Temp HP = 1 Point.
    *   1 Hit Die = 8 Points.
    *   1 Spell Slot = $15 \times (\text{Level}^{1.5})$ Points.
    *   Short Rest Feature (Action Surge, Ki, etc.) = 15 Points.
    *   Long Rest Feature (Rage, Indomitable, etc.) = 30 Points.

2.  **Accounting Logic:**
    *   **TDNW:** Sum of all party resources at maximum (Long + Short rest capacity).
    *   **Encounter Cost %:** $\frac{\text{Resources at Start} - \text{Resources at End}}{\text{TDNW}}$
    *   **Target Roles & Weights:**
        *   **Skirmish (Weight 1):** Patrols, Guards.
        *   **Standard (Weight 2):** Typical challenge.
        *   **Elite (Weight 3):** Lieutenants, Gatekeepers.
        *   **Boss (Weight 4+):** The Climax.
    *   **Target Drain (Weighted):** $\text{Target\_Cost} = \frac{\text{Encounter\_Weight}}{\text{Sum\_of\_All\_Weights\_in\_Day}} \times 100\%$

3.  **Dynamic Rating Tiers (per Encounter):**
    *   Compare **Cost %** to the **Weighted Target Drain** for that specific encounter role.
    *   **Tier 1 (Trivial):** Cost < $0.2 \times \text{Target}$ (0 Bolts)
    *   **Tier 2 (Light):** Cost < $0.6 \times \text{Target}$ (1 Bolt)
    *   **Tier 3 (Moderate):** Cost $\approx 1.0 \times \text{Target}$ (2 Bolts)
    *   **Tier 4 (Heavy):** Cost > $1.5 \times \text{Target}$ (3 Bolts)
    *   **Tier 5 (Extreme):** Cost > $2.0 \times \text{Target}$ (4 Bolts)

4.  **Auto-Adjuster Logic:**
    *   Adjust monsters until the Resource Delta matches the encounter's weighted **Target\_Cost**.
    *   If balancing to target requires nerfing a monster below reasonable limits (e.g. Boss becoming weaker than Standard), return an "Impossible" warning.

5.  **Backend Integration:**
    *   Update `decile_analysis.rs` to calculate weighted targets.
    *   Add `target_role` to `Encounter` struct.

6.  **Frontend Update:**
    *   Add a dropdown to `EncounterForm.tsx` to select Skirmish / Standard / Elite / Boss.
    *   Update `AnalysisComponents.tsx` to render Lightning Bolts based on the new $(Tier - 1)$ logic.

## Acceptance Criteria
*   In a 1-encounter day, an encounter burning 30% of resources is rated "Light."
*   In a 3-encounter day, an encounter burning 30% of resources is rated "Moderate."
*   In a 6-encounter day, an encounter burning 30% of resources is rated "Heavy/Extreme."
