# Critical Hit/Miss Implementation & Unexpected Low Damage Output

**Date:** 2025-11-27
**Status:** Resolved (Implemented Correct D&D 5e Combat Rules)

**Description:**
The simulation's damage output for a Lv5 Fighter with typical stats (+7 to hit, 2d6+4 damage, 2 attacks/round, AC17/40hp) was significantly lower than expected, leading to combat durations far exceeding D&D 5e norms (60+ rounds observed vs. expected 3-4 rounds).

**Observed Problems:**
1.  **Missing Critical Hit Damage:** The original simulation logic did not correctly implement critical hit damage (doubling damage dice). Critical hits were treated as normal hits.
2.  **Missing Critical Miss (Natural 1) Logic:** Natural 1s were not explicitly handled as automatic misses.
3.  **Missing Advantage/Disadvantage Logic:** Attack rolls were not correctly accounting for Advantage/Disadvantage conditions on hit/crit/miss determination.
4.  **Multi-Attack Targeting Flaw:** The multi-attack system prevented a combatant from hitting the same living target multiple times with a single action.

**Resolution:**
1.  **Implemented D&D 5e Critical Hit/Miss Rules:** Natural 1s are now automatic misses, and natural 20s are automatic hits that double the damage dice rolled.
2.  **Implemented Advantage/Disadvantage Logic:** Attack rolls now correctly account for Advantage/Disadvantage buffs/debuffs, rolling two d20s and applying the higher/lower result for the attack roll and critical hit/miss determination.
3.  **Fixed Multi-Attack Targeting:** Multi-attacks can now correctly target the same living enemy multiple times within a single action.
4.  **Increased Max Rounds:** The maximum rounds per encounter was increased to 100 to ensure even low-damage, high-variance fights conclude.

**Result:**
With these fixes, a Lv5 Fighter vs. Lv5 Fighter combat (with correct stats) now concludes in approximately 4-5 rounds, matching D&D 5e expectations. The original "10x lower damage" discrepancy was primarily due to these missing mechanics.

---

# Round 1 Aggregation Anomaly (Fast Fighter Appears Unharmed)

**Date:** 2025-11-27
**Status:** Open - Under Investigation

**Description:**
Despite core simulation mechanics now correctly reflecting D&D 5e rules (critical hits, multi-attacks, A/D), an anomaly persists in the aggregated GUI display for Round 1 of certain matchups. For a Fast Fighter vs. Slow Fighter setup, where both are expected to deal significant damage in Round 1, the GUI consistently shows:
*   Fast Fighter (Player 1) HP: **100.0/100** (unharmed)
*   Slow Fighter (Player 2) HP: **60.0/100** (took damage)

This implies that, in the aggregated view, the **Fast Fighter took 0 damage in Round 1**, even though deterministic and single-run debug traces confirm the Slow Fighter *does* deal damage in Round 1.

**Expected Behavior (based on single-run debug):**
*   End of Round 1: Fast Fighter HP ~60.0, Slow Fighter HP ~60.0 (both took significant damage).

**Observed Behavior (in aggregated GUI):**
*   End of Round 1: Fast Fighter HP 100.0, Slow Fighter HP 60.0.

**Investigation Notes:**
*   **Contradiction:** The GUI's aggregated display for Round 1 directly contradicts the behavior observed in a single, verified simulation run.
*   **Hypothesis:** The `aggregate_results` function, or the individual `SimulationResult`s it processes, contains a subtle logical flaw specific to Round 1, where the Fast Fighter's HP is either not correctly updated or is being read as its full HP (`c.creature.hp`) instead of its `current_hp` (`c.final_state.current_hp`) when calculating the Round 1 average.
*   **Possible Causes:**
    1.  A bug within `aggregate_results` where Player 1's Round 1 state is misinterpreted or overridden.
    2.  A highly improbable statistical bias in the 20% slice selection where almost all included runs have the Slow Fighter fail to deal damage to Player 1 in Round 1.
    3.  A very subtle bug in `run_single_simulation` or `run_encounter` where Player 1's HP is being reset or miscalculated for Round 1 in a way that is masked by the averaging process.

**Action Plan:**
1.  Add extensive `eprintln!` logging within the `aggregate_results` function to inspect the `final_state.current_hp` values of both fighters from *each individual `Round` object* that is fed into the averaging process for Round 1. This will confirm what exact HP values are being received and processed by the aggregation.
2.  If individual `Round` objects are consistently showing P1 at 100 HP, trace back further into `run_single_simulation` and `run_round` for Player 1's HP status after Slow Fighter's turn.

**Priority:** High (Distorts initial combat results and user understanding).