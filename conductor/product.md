# Product Guide - BattleSim

## Target Users
- **Dungeon Masters:** Use the tool to verify encounter balance for their specific parties, ensuring combat is challenging but fair.
- **Theorycrafters:** Test character builds and team compositions to optimize tactics and understand performance under "average" conditions.

## Core Goals
- **Encounter Balancing:** Move beyond Challenge Rating (CR) to provide a high-fidelity simulation of how an encounter will actually play out.
- **Strategy Optimization:** Provide a platform to test complex decision-making, helping users determine the optimal use of limited resources.
- **Simulation Accuracy:** Maintain a balance between the speed of probability-based "average" results and the nuance of tactical play.

## Key Features
- **Complex Tactical Logic:** Simulation of advanced resource management (e.g., "Should the Fighter Action Surge now or save it for a later round based on encounter difficulty?").
        - **5e.tools Integration:** Rapidly import monsters and NPCs from 5e.tools JSON data to pre-fill custom creature forms, allowing for easy adjustment before adding to combat.
        - **Timeline-Based Simulation:** Sequence multiple combat encounters and short rests in a linear timeline to simulate a full adventuring day, accounting for resource recovery and Hit Die consumption.
        - **Dual-Gauge Pacing Dashboard:** Visualize resource attrition via separate metrics for "Vitality" (Physical reserves) and "Power" (Resource potential) to provide a precise tactical picture.
        - **Modular Action System:** Leverage the existing flexible action system to support complex creature abilities and unique gameplans.
        - **Tactically-Aware Auto-Balancer:** Automatically optimizes encounter difficulty, choosing between HP and Damage adjustments based on the party's Vitality/Power split.

## Visualization & Reporting
- **Human-Readable Combat Logs:** Narrative-driven event logs that transform raw simulation data into natural language using smart ID resolution for combatant and action names. Features a two-tier viewing experience: a concise summary card for quick review, and an expandable detailed view showing raw dice rolls, modifiers (e.g., Bless, Bane, Proficiency), and explicit AC comparisons for deep technical verification.
- **The Descent Graph:** A custom SVG chart visualizing party net worth attrition and risk across the entire adventuring day.
- **Assistant Voice Summary:** Provides human-readable assessments of the day's pacing (e.g., \"Balanced\", \"Minor Drift\", or \"Impossible Day\").
- **Performance Metrics:** Rich data including Damage Per Round (DPR), resource consumption, and kill-turn distributions.
