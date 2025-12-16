# Battlesim2 Regression Test Suite - Comprehensive Quintile Analysis Report

**Date:** December 16, 2025  
**Test Environment:** Rust debug build, 1000 simulations per scenario  
**Analysis Tool:** Custom comprehensive regression test runner  

## Executive Summary

✅ **All regression tests PASSED** - The core functionality is working correctly.  
⚠️ **Reliability concerns detected** - High variance indicates simulation instability.  
✅ **-1000000 score issue RESOLVED** - No instances of critical scoring errors detected.  

## Test Results Overview

| Test Scenario | Expected Winner | Median Win Rate | Test Status | Reliability |
|---------------|----------------|-----------------|-------------|-------------|
| fast_init_PlayerA_wins.json | Player | 100.0% | ✅ PASSED | ⚠️ High Variance |
| damage_vs_precision_MonsterB_wins.json | Monster | 2.0% | ✅ PASSED | ⚠️ High Variance |
| heavy_vs_consistent_PlayerA_wins.json | Player | 100.0% | ✅ PASSED | ⚠️ High Variance |

## Detailed Analysis by Scenario

### 1. Fast Initiative Dominance Test (fast_init_PlayerA_wins.json)

**Expected Outcome:** Player should win due to initiative advantage  
**Actual Result:** ✅ PASSED

#### Quintile Breakdown:
| Quintile | Win Rate | Survivors | HP Lost % | Battle Duration | Status |
|----------|----------|-----------|-----------|-----------------|---------|
| Worst 20% | 0.0% | 0/1 | 100.0% | 17 rounds | ✗ FAIL |
| Below Average | 9.5% | 0/1 | 100.0% | 20 rounds | ✗ FAIL |
| **Median** | **100.0%** | **1/1** | **96.0%** | **19 rounds** | **✓ PASS** |
| Above Average | 100.0% | 1/1 | 91.0% | 20 rounds | ✓ PASS |
| Best 20% | 100.0% | 1/1 | 82.0% | 18 rounds | ✓ PASS |

#### Key Metrics:
- **Score Range:** -35.00 to 10,030.00
- **Average Score:** 6,612.67
- **Win Rate Variance:** 100.0% (extremely high)
- **Reliability Issues:** High variance indicates unstable simulation

#### Encounter Rating:
- **Risk Factor:** TPKRisk (Total Party Kill possible in worst cases)
- **Difficulty:** Grueling
- **Encounter Label:** Catastrophic

---

### 2. Damage vs Precision Test (damage_vs_precision_MonsterB_wins.json)

**Expected Outcome:** Monster should win (precision beats damage)  
**Actual Result:** ✅ PASSED

#### Quintile Breakdown:
| Quintile | Win Rate | Survivors | HP Lost % | Battle Duration | Status |
|----------|----------|-----------|-----------|-----------------|---------|
| Worst 20% | 0.0% | 0/1 | 100.0% | 13 rounds | ✓ PASS |
| Below Average | 0.0% | 0/1 | 100.0% | 19 rounds | ✓ PASS |
| **Median** | **2.0%** | **0/1** | **99.8%** | **16 rounds** | **✓ PASS** |
| Above Average | 100.0% | 1/1 | 85.4% | 12 rounds | ✗ FAIL |
| Best 20% | 100.0% | 1/1 | 60.1% | 12 rounds | ✗ FAIL |

#### Key Metrics:
- **Score Range:** -89.00 to 10,080.00
- **Average Score:** 4,087.31
- **Win Rate Variance:** 100.0% (extremely high)
- **Reliability Issues:** High variance indicates unstable simulation

#### Encounter Rating:
- **Risk Factor:** Suicidal (Party loses >50% even in typical scenarios)
- **Difficulty:** Grueling
- **Encounter Label:** Catastrophic

---

### 3. Heavy vs Consistent Test (heavy_vs_consistent_PlayerA_wins.json)

**Expected Outcome:** Player should win (burst damage beats consistent damage)  
**Actual Result:** ✅ PASSED

#### Quintile Breakdown:
| Quintile | Win Rate | Survivors | HP Lost % | Battle Duration | Status |
|----------|----------|-----------|-----------|-----------------|---------|
| Worst 20% | 0.0% | 0/1 | 100.0% | 15 rounds | ✗ FAIL |
| Below Average | 3.0% | 0/1 | 99.8% | 14 rounds | ✗ FAIL |
| **Median** | **100.0%** | **1/1** | **84.2%** | **14 rounds** | **✓ PASS** |
| Above Average | 100.0% | 1/1 | 65.1% | 11 rounds | ✓ PASS |
| Best 20% | 100.0% | 1/1 | 38.6% | 10 rounds | ✓ PASS |

#### Key Metrics:
- **Score Range:** -100.00 to 10,090.00
- **Average Score:** 6,145.67
- **Win Rate Variance:** 100.0% (extremely high)
- **Reliability Issues:** High variance indicates unstable simulation

#### Encounter Rating:
- **Risk Factor:** TPKRisk (Total Party Kill possible in worst cases)
- **Difficulty:** Grueling
- **Encounter Label:** Catastrophic

## Critical Findings

### ✅ **RESOLVED: -1000000 Score Issue**
- **Zero instances** of scores ≤ -1000000 across all 3,000 simulations
- Previous critical scoring bug appears to be completely fixed
- Score ranges are now within expected bounds (-100 to ~10,090)

### ⚠️ **RELIABILITY CONCERNS: High Simulation Variance**

All three scenarios exhibit **extreme variance** with 100.0% win rate spread between worst and best quintiles:

1. **Binary Outcome Pattern:** Results cluster at either 0% or 100% win rates
2. **Lack of Middle Ground:** Few simulations produce intermediate results
3. **Inconsistent Performance:** Same scenario produces drastically different outcomes

**Impact on Testing:**
- Regression tests still pass because median quintiles meet expectations
- However, the high variance suggests underlying simulation instability
- Makes balance tuning and difficulty assessment challenging

### 📊 **Performance Analysis**

#### Battle Duration Patterns:
- **Fast Initiative:** 17-20 rounds (consistent)
- **Damage vs Precision:** 12-19 rounds (variable)
- **Heavy vs Consistent:** 10-15 rounds (decreasing with better outcomes)

#### HP Remaining Patterns:
- Winning scenarios typically retain 38-96% HP
- Losing scenarios always lose 100% HP (as expected)
- HP efficiency correlates with battle duration

## Recommendations

### Immediate Actions:
1. **Investigate Variance Source:** Examine random number generation, dice mechanics, or action resolution
2. **Increase Sample Size:** Use 2000+ simulations for more stable statistical analysis
3. **Add Variance Metrics:** Include standard deviation in quintile analysis

### Medium-term Improvements:
1. **Balance Tuning:** All scenarios rated "Catastrophic" - consider adjusting encounter balance
2. **Reliability Testing:** Implement variance thresholds for simulation stability
3. **Enhanced Monitoring:** Add real-time variance tracking during simulations

### Long-term Considerations:
1. **Simulation Architecture:** Review core mechanics for sources of binary outcomes
2. **Difficulty Calibration:** Re-evaluate encounter rating system
3. **Statistical Framework:** Implement more sophisticated reliability metrics

## Conclusion

The regression test suite demonstrates **functional correctness** with all tests passing and the critical -1000000 scoring issue resolved. However, the **extreme variance** across all scenarios indicates underlying simulation instability that requires attention.

While the current implementation meets basic functionality requirements, the reliability concerns suggest that further refinement is needed before using the simulation for balance tuning or difficulty assessment in production environments.

**Overall Status:** ⚠️ **Functional with Reliability Concerns**