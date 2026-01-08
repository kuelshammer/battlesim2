# BattleSim

<!-- AI AGENT INSTRUCTION: BEFORE STARTING WORK, YOU MUST READ AGENTS.md FOR BEHAVIORAL PROTOCOLS AND TASK MANAGEMENT RULES. -->
<!-- THIS REPO USES THE BEADS ISSUE TRACKER (bd command) FOR TASK MANAGEMENT. -->

This is a **Narrative-First** 5e encounter simulator. It goes beyond simple win/loss probabilities to analyze the *dramatic structure* of your battles.

## The Director's Cut
BattleSim doesn't just tell you if the party survives; it tells you **how it felt**. Using a high-fidelity deterministic engine, it measures:

*   **Narrative Arc:** Visualized via the "Heartbeat Graph," showing the emotional pacing of the fight.
*   **Director's Score:** A metric (0-100) rating the dramatic tension. Is it a boring slog (low score) or a nail-biting thriller (high score)?
*   **Archetype Classification:** Instead of generic grades, encounters are classified by their narrative role:
    *   *The Meat Grinder* (High Lethality, Low Variance)
    *   *The Cakewalk* (Low Lethality, Low Resource Cost)
    *   *The Coin Flip* (High Volatility)
*   **Doom Horizon:** Predicts exactly when the party will run out of steam across a full adventuring day.

## Key Features
*   **High-Fidelity Rules:** Actual d20 rolls, crits, and complex mechanics (e.g., Triple Advantage/Elven Accuracy).
*   **Magic Item Support:** Native support for complex items like *Cloak of Displacement*, *Armor of Agathys*, *Bracers of Defense*, and *Ring/Cloak of Protection*.
*   **Resource Tracking:** Tracks spell slots, Hit Dice, and class features across multi-encounter days.
*   **Deterministic Re-simulation:** Every run is seeded and reproducible. You can replay the exact "1 in 100" disaster scenario to see what went wrong.

---

## Two-Pass Simulation System

The simulator uses a **Two-Pass Deterministic Re-simulation** architecture for efficient memory usage and precise percentile analysis:

### Phase 1: Lightweight Survey Pass
- Runs **10,100 iterations** using lightweight simulation (no event collection)
- Each run stores only: seed, encounter scores, final score, deaths
- Memory: ~323 KB (32 bytes × 10,100)
- Time: ~10 seconds

### Phase 2: Seed Selection
Analyzes all 10,100 runs and identifies interesting seeds for re-simulation:

**1% Bucket Medians (100 seeds → Tier B)**
- Divides results into 100 equal-sized buckets
- Selects median from each 1% percentile (P0-1, P1-2, ..., P99-100)
- Enables true 1% granularity analysis (vs previous 10% buckets)

**Global Deciles (11 seeds → Tier A)**
- Selects P5, P15, P25, P35, P45, P50, P55, P65, P75, P85, P95
- These get full event logs for BattleCard playback

**Per-Encounter Extremes (59 seeds → Tier C)**
- Selects extreme runs for each encounter (P0, P50, P100)
- Used for encounter-specific analysis, no event logs needed

### Phase 3: Three-Tier Deep Dive
Re-runs only the ~170 selected seeds with appropriate event collection:

| Tier | Seeds | Event Type | Memory | Use Case |
|------|-------|------------|--------|----------|
| **A** | 11 | Full Events | 2.2 MB | Decile logs, full playback |
| **B** | 100 | Lean Events | 2 MB | 1% percentile medians |
| **C** | 59 | None (Phase 1 data) | ~2 KB | Per-encounter analysis |

**Total Phase 3 Memory:** ~4.2 MB (vs ~15-20 MB if all runs had full events)

### What Are "Lean Events"?
Lean events store aggregate statistics instead of per-attack events:

```rust
// Full Event Log: ~200-500 KB per run
Vec<Event> {
    AttackHit { ... },      // ×500+ (every attack)
    AttackMissed { ... },   // ×300+ (every miss)
    DamageTaken { ... },    // ×500+ (every damage instance)
}

// Lean Event Log: ~10-30 KB per run
LeanRunLog {
    round_summaries: [
        RoundSummary {
            total_damage: HashMap<CombatantID, TotalDamage>,  // Aggregated
            total_healing: HashMap<CombatantID, TotalHealing>, // Aggregated
            deaths: Vec<CombatantID>,                         // Who died
        },
        // ... one summary per round (10-20 rounds)
    ],
}
```

**Result:** 150-300× smaller than full event logs!

### Performance Characteristics

| Metric | Previous (2,511 runs) | Current (10,100 runs) | Change |
|--------|----------------------|----------------------|--------|
| Granularity | 10% buckets | 1% buckets | **10× better** |
| Total Memory | ~6.1 MB | ~4.5 MB | **-25%** |
| Total Time | ~5.5s | ~16s | +3× |
| Phase 1 Time | ~2.5s | ~10s | 4× |
| Phase 3 Time | ~3s | ~6s | 2× |

The 3× time increase is acceptable for:
1. **10× better granularity** (true 1% percentiles)
2. **Multi-modal distribution detection** (can see if results cluster)
3. **Exact confidence intervals** (no interpolation between 10% buckets)

### Why This Matters

**Previous 10% buckets:**
```
P0-10  │████████████████████████████████│  [251 runs → 1 median]
       ↑                                    ↑
    Wide spread within bucket           Only see median
```

**Current 1% buckets:**
```
P0-1   │█████████│  [101 runs → 1 median]
P1-2   │█████████│  [101 runs → 1 median]
...
       ↑
   True 1% granularity - can see distribution within each percentile
```

## Getting Started
* Install nodejs: https://nodejs.org/en
* Download node packages: `npm i`
* Run in dev mode: `npm run dev`
* Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

## Directory Structure
* `public`: images to be displayed on the website
* `simulation-wasm`: The Rust-based high-performance simulation engine.
* `src`
  * `components`: UI elements
    * `creatureForm`: the dialog which is shown when clicking adding/updating a creature
    * `simulation`: the components for showing the simulation's results on the home page
    * `utils`: general form elements
  * `data`: list of monsters & PC templates to populate the UI
  * `model`: type definitions, enums, and the core of the simulation
  * `pages`: HTTP URL endpoints
* `styles`: global CSS

## Contributing
To contribute, fork this repository and make pull requests.

This project's main goals are to:
* Streamline the process of inputting an encounter as much as possible without sacrificing the output's accuracy
* Give the user a clear understanding of what's happening, so they can decide whether or not the encounter is right for their table

Contributions that are likely to get accepted:
* Templates for multi-classed characters
* Streamlining of the UI (e.g. scanning a PC/monster on dndbeyond from just its URL, and making an educated guess about its gameplan)
* Improving the result's clarity (e.g. "luck sliders" to see multiple scenarios and quickly see how swingy an encounter is)
* Tightening the simulation algorithm to make it more accurate (e.g. making sure the order of the creatures does not matter)

Contribution checklist:
* Make sure the project compiles with `npm run build`
* Make sure the contents `src/data` are updated to reflect your changes
* Make sure your changes are backwards-compatible with the save files a user might already have in their local storage

Common reasons why a pull request might be denied:
* It makes the UI too tedious to use.
* It makes the result too confusing to read.
* It adds too much data to the monsters, and risks breaking the terms of the WotC Fan Content Policy

## Licence
<a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/4.0/"><img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by-nc-sa/4.0/88x31.png" /></a><br />This work is licensed under a <a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/4.0/">Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License</a>.

The license only covers commits on the `main` branch made after June 2nd, 2023, and only the contents of the `src` and `styles` directories, with the exception of:
* The `src/data` directory 
* The `src/components/utils/logo.tsx` file
* The `src/components/utils/logo.module.scss` file.

___

BattleSim is unofficial Fan Content permitted under the Fan Content Policy. Not approved/endorsed by Wizards. Portions of the materials used are property of Wizards of the Coast. ©Wizards of the Coast LLC.