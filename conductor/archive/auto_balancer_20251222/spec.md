# Spec: Encounter Auto-Balancer Engine

## Overview
The Auto-Balancer engine transforms an unbalanced encounter into a "perfect" challenge by iteratively simulating combat and adjusting monster statistics until target safety and intensity thresholds are met.

## Targets
- **Safety Grade:** B (Fair) - Party survives in the 5th percentile (Disaster Timeline).
- **Intensity Tier:** 3-4 (Moderate to Heavy) - Party consumes 40-90% of resources in the median case.
- **Combat Length:** Median rounds <= 8.

## Tuning Strategy
1. **Role-Based Tuning:**
    - **Boss:** Adjusts Burst Damage (Nerf) or Legendary Res/HP (Buff).
    - **Brute:** Adjusts Damage Dice (Nerf) or HP Sponge (Buff).
    - **Striker:** Adjusts Accuracy/To-Hit.
    - **Controller:** Adjusts Save DCs.
    - **Minion:** Adjusts Count.

2. **Reconstruction:**
    - Modified values are never just flat numbers; they are back-calculated into D&D notation (e.g., 178 HP -> 17d10 + 85).

## Architecture
- **Rust Backend:** Handles the iterative Monte Carlo loops (2,510 runs per step) for maximum performance.
- **WASM Binding:** Exposes a single "Optimized Encounter" result to the frontend.
