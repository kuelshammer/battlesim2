# The Decile Method: Analyzing Combat Swinginess
### *Beyond Averages: How to Prevent Accidental TPKs*

## 1. The Problem with Averages
In D&D 5e, the "Average Result" is often a lie.
*   **Scenario:** A Monster deals massive damage but has low accuracy.
*   **The Average:** The simulator says the Party wins with **50 HP** remaining.
*   **The Reality:** In 40% of simulations, the Monster crits once and kills the Wizard instantly. In the other 60%, the Monster misses and dies easily.

If you only look at the average, you will design a **Trap Encounter**—one that looks fair on paper but has a high probability of causing a Total Party Kill (TPK) due to bad luck.

To fix this, we do not look at one "Average Combat." We look at **5 Distinct Timelines** picked from a statistically significant sample size.

---

## 2. The Methodology
We simulate the combat **2,510 times**. This "Magic Number" is mathematically elegant for specific reasons:
*   **The Math:** $2,510 / 10 = \mathbf{251}$.
*   **The Median Trick:** Since **251** is an **odd number**, every 10% slice (Decile) has an *exact* center (the 126th run in that slice). We never have to average two runs together; we can always point to a real, specific simulation.
*   **The Precision:** It provides a Margin of Error $\approx 2\%$, high enough for reliable "Deadly" labels while remaining fast enough for chained combat calculations.

### Step 1: The Tiered Scoring (Value of a Life & Efficiency)
To accurately sort "Bad" runs from "Good" runs, we prioritize **Survival** over **Resource Efficiency**, and **Resource Efficiency** over **Hit Points**.

$$ \text{Score} = (\text{Survivors} \times 1,000,000) + \text{Total Party HP} - \text{Resource Penalty} - \text{Total Monster HP} $$

*   **The 1,000,000 Bonus:** Ensures keeping a party member alive is mathematically more valuable than any amount of Hit Points or resource conservation.
*   **The Resource Penalty:** Factors in the "cost" of victory (Spell Slots, Class Features, Potions).
*   **The Result:** Simulation results are sorted primarily by **Death Count**, and secondarily by **Resource Cost** (Efficiency).

#### Resource Penalty Weights:
- **Spell Slots:** $15 \times (\text{Level}^{1.6})$ (e.g., Lvl 1 = 15, Lvl 3 = 87, Lvl 9 = 500)
- **Short Rest Class Features:** 20 points (e.g., Action Surge, Ki)
- **Long Rest Class Features:** 40 points (e.g., Rage, Indomitable)
- **Consumables (Potions):** 20 points

### Step 2: Mapping 10 Slices to 5 Battle Cards
We divide the 2,510 sorted runs into 10 deciles (slices of 251 runs) but display only **5 Rows** to maintain a clean UI. We pick the **Medians** of specific slices to recreate a **Statistical Box Plot**:

| UI Row Label | Statistical Meaning | Which Slice? | **Run Index** (0-2509) |
| :--- | :--- | :--- | :--- |
| **1. Disaster** | **5th Percentile** (Worst 10% Median) | Slice 1 | **#125** |
| **2. Struggle** | **25th Percentile** (Bad Luck Median) | Slice 3 | **#627** |
| **3. Typical** | **50th Percentile** (Global Median) | (Global) | **#1,255** |
| **4. Heroic** | **75th Percentile** (Good Luck Median) | Slice 8 | **#1,882** |
| **5. Legend** | **95th Percentile** (Best 10% Median) | Slice 10 | **#2,384** |

---

## 3. The Quality Assurance System

We use the data from these specific runs to provide a **Safety Grade** and an **Intensity Tier**.

### Part 1: The Safety Grade (The "Floor" Check)
*This answers: "Is it fair?"

We check the **Disaster Run (#125)** and the **Struggle Run (#627)**.

| Grade | Label | Criteria (Logic) | What it means |
| :--- | :--- | :--- | :--- |
| **A** | **Secure** | Disaster Run has >10% HP remaining. | Party is safe even with terrible luck. |
| **B** | **Fair** | Disaster Run is Alive (>0 HP). | Bad luck hurts, but doesn't kill. **The Gold Standard.** |
| **C** | **Risky** | Disaster Run is TPK, but Struggle Run is Alive. | Bottom 10% kills the party. Use for climactic bosses. |
| **D** | **Unstable** | Struggle Run is a TPK. | Bottom 25% kills the party. **Avoid this.** |
| **F** | **Broken** | Typical Run is a TPK. | Mathematically impossible/guaranteed TPK. |

### Part 2: The Intensity Tier (The "Drain" Check)
*This answers: "Is it fun/challenging?"

We check the **Typical Run (#1,255)** for **Resources Remaining** (HP + Spell Slots proxy).

| Tier | Label | Typical Run Ending State | Player Feeling |
| :--- | :--- | :--- | :--- |
| **1** | **Trivial** | > 90% Resources Left | "Was that it?" |
| **2** | **Light** | 70% – 90% Resources Left | "Warm up." |
| **3** | **Moderate** | 40% – 70% Resources Left | "Good fight." |
| **4** | **Heavy** | 10% – 40% Resources Left | "We need a rest." |
| **5** | **Extreme** | < 10% Resources Left | "We almost died!" |

---

## 4. Combined Labels (The "Sweet Spot")

| Label | Grade + Tier | Narrative Logic |
| :--- | :--- | :--- |
| **The Epic Challenge** | **B + 4** | Average case is high tension (<40% HP), but Disaster run survives. Perfect for **Bosses**. |
| **The Tactical Grinder**| **A + 3** | Average case drains ~40% resources, but worst case is totally safe. Great for **Mini-Bosses**. |
| **The Action Movie** | **B + 2** | Low resource drain, but Disaster run gets scary close to 0 HP. **Glass Cannons**. |
| **The Trap** | **C + 2** | Average case looks easy, but Disaster case is a TPK. One bad breath weapon = End of Campaign. |
| **The Slog** | **A + 5** | Totally safe, but drains 95% of resources. Monster has high HP but low damage. Boring. |

---

## 5. Adventuring Day Labeling

When simulating a full day, we apply this logic to the **Final State of the Day**.

**The Ideal Day:** **Safety Grade B + Intensity Tier 5**
*   **Tier 5:** Players end the day with empty tanks (0-10% resources), validating their resource management.
*   **Grade B:** Even with "Nat 1" luck all day (Disaster Timeline), the party survived (barely).

### Day Rating Algorithm
1.  Check **Safety**: If Run #125 is TPK $\rightarrow$ Grade C/D. If Run #125 is Alive but <5% HP $\rightarrow$ Grade B. Else $\rightarrow$ Grade A.
2.  Check **Intensity**: Based on Typical Run (#1,255) resources.
    *   > 60% $\rightarrow$ Under-tuned
    *   30% - 60% $\rightarrow$ Standard Day
    *   5% - 30% $\rightarrow$ Perfect Challenge
    *   < 5% $\rightarrow$ Overwhelming
