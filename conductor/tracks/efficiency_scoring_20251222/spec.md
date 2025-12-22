# Spec: Efficiency-Based Performance Scoring

## Overview
Currently, the simulation evaluates performance based on survival and remaining HP. While effective for simple encounters, it fails to account for the "cost" of victory in an Adventuring Day context. A Wizard spending a 9th-level slot to kill a Goblin should score lower than a Fighter using a Cantrip.

This track implements an **Efficiency-Based Scoring Algorithm** that penalizes resource consumption.

## Scoring Formula
```
Score = (Survivors * 1,000,000) + Remaining_Player_HP - Resource_Penalty - Remaining_Monster_HP
```

### Resource Penalties
1. **Spell Slots:** `15 * (Slot_Level ^ 1.6)`
2. **Short Rest Resources:** 20 points (e.g., Action Surge, Ki)
3. **Long Rest Resources:** 40 points (e.g., Rage, Indomitable)
4. **Consumables:** 20 points (e.g., Potions)

## Acceptance Criteria
- Runs are sorted primarily by survival, then by resource efficiency, then by remaining HP.
- The "Legend" timeline (Best Decile) reflects the most resource-efficient victories.
- The "Struggle" timeline correctly identifies "Nova Panic" scenarios where players burn all resources to stay at high HP.
- Rust simulation engine handles these calculations with minimal performance overhead.
