# The Quintile Method: Analyzing Combat Swinginess
### *Beyond Averages: How to Prevent Accidental TPKs*

## 1. The Problem with Averages
In D&D 5e, the "Average Result" is often a lie.
*   **Scenario:** A Monster deals massive damage but has low accuracy.
*   **The Average:** The simulator says the Party wins with **50 HP** remaining.
*   **The Reality:** In 40% of simulations, the Monster crits once and kills the Wizard instantly. In the other 60%, the Monster misses and dies easily.

If you only look at the average, you will design a **Trap Encounter**—one that looks fair on paper but has a high probability of causing a Total Party Kill (TPK) due to bad luck.

To fix this, we do not look at one "Average Combat." We look at **5 Distinct Timelines**.

---

## 2. The Methodology
We simulate the combat **1,005 times**. Instead of averaging them all together, we sort them based on a **Tiered Scoring System** and slice them into 5 equal blocks (Quintiles) of **201 runs** each.

### Step 1: The Tiered Scoring (Value of a Life)
To accurately sort "Bad" runs from "Good" runs, we must prioritize **Survival** over **Hit Points**. A run where 5 players survive with 1 HP each is *better* than a run where 4 players survive with full HP (because one player died).

We calculate the score for every run using this logic:

$$ \text{Score} = (\text{Survivors} \times 10,000) + \text{Total Party HP} - \text{Total Monster HP} $$

*   **The 10,000 Bonus:** This ensures that keeping a party member alive is mathematically more valuable than any amount of Hit Points.
*   **The Result:** The simulation sorts runs primarily by **Death Count**, and secondarily by **Resource Cost**.

### Implementation Status

**Current Backend Status (as of [Today's Date])**
The backend for quintile analysis has undergone significant improvements to ensure the accuracy and reliability of the data presented for the "5 Distinct Timelines".

*   **Sorted Simulation Results:** All 1,005 simulation runs are now explicitly sorted from worst to best performance using the described tiered scoring system *before* being divided into the 5 quintile blocks. This ensures that each quintile genuinely represents a specific range of outcomes (e.g., the worst 20% or best 20%).
*   **Median Run Selection:** The "median run" for each quintile is accurately identified as the 101st run within its 201-run sorted slice. This run serves as the representative "snapshot of reality" for that quintile.
*   **Detailed Visualization Data Population:** The `CombatantVisualization` data for each median run is now correctly populated, capturing the `max_hp`, `current_hp`, `is_dead` status, `is_player` flag, `hp_percentage`, and `battle_duration_rounds`. This ensures the backend provides all necessary information for a rich frontend visualization.
*   **Critical Bug Fix:** A significant bug was identified and resolved where the `creature.hp` (intended to represent maximum HP) of combatants was erroneously being overwritten with their `current_hp` during the simulation's result conversion. This led to incorrect `max_hp: 0.0` values in the visualization data and caused `median_run_visualization` to appear empty or incorrect for some quintiles. This has been rectified, ensuring `max_hp` accurately reflects the creature's original maximum health.
*   **Backend Verified:** The corrected backend data generation has been thoroughly verified through CLI execution and detailed inspection of the JSON output, confirming that all quintiles now provide accurate and complete median run visualization data.

### Step 2: The Slicing
Sort the 1,005 runs from **Lowest Score** (Worst Case) to **Highest Score** (Best Case). Split them into 5 blocks:

1.  **The Disaster Timeline** (Bottom 20%) $\rightarrow$ Runs 1–201
2.  **The Struggle Timeline** (20%–40%) $\rightarrow$ Runs 202–402
3.  **The Typical Timeline** (40%–60%) $\rightarrow$ Runs 403–603
4.  **The Heroic Timeline** (60%–80%) $\rightarrow$ Runs 604–804
5.  **The Legend Timeline** (Top 20%) $\rightarrow$ Runs 805–1,005

---

## 3. The 5 Scenarios Explained

When analyzing an encounter, we check the **Median Result** (the majority outcome) inside each block.

### 💀 1. The Disaster Timeline (Worst 20%)
*   **Contains:** TPKs, Multiple PC Deaths, or Extreme Near-Death scenarios.
*   **Narrative:** The players roll Nat 1s. The Monster crits. Tactics fail.
*   **Analysis Goal:** **Safety Floor.**
*   **The Question:** *Does bad luck guarantee a death?*
*   **Red Flag:** If the "Survivors" count in this block is less than the Party Size, the encounter has a high risk of lethality.

### ⚠️ 2. The Struggle Timeline (Bad Luck)
*   **Contains:** Single PC Deaths or Heavy Resource Drain.
*   **Narrative:** The dice are cold. The fight drags on. Healing potions are chugged.
*   **Analysis Goal:** **Resource Cost.**
*   **The Question:** *If things go wrong, do we lose a party member?*
*   **Red Flag:** If the Median HP here is <10% of max, players will panic.

### 📊 3. The Typical Timeline (Average)
*   **Contains:** The statistical average outcome.
*   **Narrative:** Hits and misses balance out.
*   **Analysis Goal:** **Intended Difficulty.**
*   **The Question:** *Is this the challenge rating I intended?*

### ⚔️ 4. The Heroic Timeline (Good Luck)
*   **Contains:** Clean wins with minimal resource usage.
*   **Narrative:** Players roll well. The Paladin lands a Smite early.
*   **Analysis Goal:** **Pacing.**
*   **The Question:** *Does the monster die too fast to be memorable?*

### 👑 5. The Legend Timeline (Best Case)
*   **Contains:** "Stomps" and Nova rounds.
*   **Narrative:** Everything connects. The boss dies in Round 1 or 2.
*   **Analysis Goal:** **Triviality Check.**
*   **The Question:** *Is the boss too weak?*

---

## 4. Interpreting "Swinginess" (Volatility)

The most important use of this tool is to detect **Swinginess**.

### ✅ The Stable Encounter (Good Design)
*   **Disaster Block:** 0 Deaths, Low HP.
*   **Typical Block:** 0 Deaths, Medium HP.
*   **Verdict:** The players are in control. Bad luck hurts, but doesn't kill.

### ❌ The "Glass Cannon" Trap (Bad Design)
*   **Disaster Block:** **TPK or 1-2 Deaths.**
*   **Typical Block:** 0 Deaths, High HP.
*   **Verdict:** **High Swinginess.**
    *   The monster likely has **Low HP** (dies fast in Typical runs) but **High Damage** (kills fast in Disaster runs).
    *   *Fix:* Increase Monster HP, Decrease Monster Damage.

### ❌ The "Meat Grinder" (Bad Design)
*   **Disaster Block:** 1 Death.
*   **Typical Block:** 0 Deaths, but 5 HP remaining.
*   **Verdict:** **Overtuned.** Even average luck results in a near-death experience.

---

## 5. Example Data Output

Here is how you should visualize the data for the user.

**Encounter: Level 5 Party (4 Players) vs. Homebrew Boss**

| Scenario | Survivors (Median) | Party HP (Median) | Verdict |
| :--- | :--- | :--- | :--- |
| **1. Disaster** | **0 / 4** | 0 (Dead) | 💀 **TPK Risk** |
| **2. Struggle** | **3 / 4** | 15 HP | ⚠️ **Lethal Risk** |
| **3. Typical** | 4 / 4 | 85 HP | ✅ Fair |
| **4. Heroic** | 4 / 4 | 120 HP | ✅ Easy |
| **5. Legend** | 4 / 4 | 150 HP | ✅ Trivial |

**Analysis:**
*   **The Average (Typical)** looks fine (Everyone alive, 85 HP).
*   **The Reality:** In the "Struggle" timeline (20-40% of cases), a player dies. In the "Disaster" timeline, everyone dies.
*   **Conclusion:** This encounter is a **Trap**. The DM must nerf the monster's burst damage to prevent the deaths in the lower quintiles.</content>
<parameter name="filePath">QUINTILE_METHODOLOGY.md