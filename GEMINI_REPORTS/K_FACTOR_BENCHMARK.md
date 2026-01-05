# Simulation Performance Benchmark (K-Factor)

This report examines the simulation time for a complete adventuring day (`fighters_3_encounters.json`) across various K-factors.

**Formula:** $N = (2K-1) 	imes 100$ runs.

| K-Factor | Total Runs (N) | Duration (ms) | Avg ms/Run |
|:---------|:---------------|:--------------|:-----------|
| 1        | 100            | 156           | 1.560      |
| 2        | 300            | 445           | 1.483      |
| 3        | 500            | 756           | 1.512      |
| 4        | 700            | 1063          | 1.519      |
| 5        | 900            | 1469          | 1.632      |
| 6        | 1100           | 1648          | 1.498      |
| 7        | 1300           | 1953          | 1.502      |
| 8        | 1500           | 2268          | 1.512      |
| 9        | 1700           | 2587          | 1.522      |
| 10       | 1900           | 2932          | 1.543      |
| 20       | 3900           | 6084          | 1.560      |
| 30       | 5900           | 10611         | 1.798      |
| 40       | 7900           | 16016         | 2.027      |
| 50       | 9900           | 21388         | 2.160      |

**Observations:**
- Scaling is roughly linear with total runs.
- Average time per run increases slightly at higher K values (potentially due to memory pressure or cache effects in the large results vector).
- K=51 (current "Precise" default) would result in 10,100 runs, taking approximately 22 seconds for this 3-encounter scenario.
