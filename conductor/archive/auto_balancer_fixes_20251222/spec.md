# Spec: Auto-Balancer Backend Improvements

## Overview
Address critical bugs and logical flaws in the Auto-Balancer engine that cause it to report success (Grade A/B) without actually fixing the encounter's lethality or modifying key monster actions.

## Requirements
1. **Support Template Actions:** Ensure `adjust_damage` and `adjust_dc` correctly modify `Action::Template` (used for Breath Weapons and other high-impact abilities).
2. **Dynamic Stopping Condition:** Change the `is_balanced` target from Grade B (5% survival) to Grade A (100% survival) for any encounter that starts as "Broken" (Grade F).
3. **Improved Role Detection:** 
    - Monsters with massive single-target damage should be detected as **Strikers**.
    - The "Boss" threshold should be more sensitive to single-monster encounters.
4. **Finer DC Adjustments:** Change DC adjustment step from `1.0` to `0.5` for better granularity.
5. **Multiattack Awareness:** Ensure `finalize_adjustments` considers the number of attacks when reconstructing dice strings.
