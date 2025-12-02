# Critical Combat Simulation Bugs - Implementation Plan

## 🔴 Critical Issues (Priority: Immediate)

### 1. **CRITICAL: Resistance Math Verification Needed**
**Status:** ⚠️ Needs Investigation  
**Reported Issue:** Resistance to slashing damage (from Rage) may not be applying correctly in all cases.

**Evidence from User:**
- Kaelen starts with 90 HP, has Rage active (0.5x damage multiplier)
- Round 1: Takes 17 damage from Andreas
- Expected: Should take ~8-9 damage (17 / 2)
- HP at Round 2 start: 73 HP
- Actual damage taken: 17 (90 - 73 = 17)

**Note:** Recent implementation shows resistance working in logs:
```
Damage: 23 (Base) * 0.50 () = 11
```

**Action Required:**
- Verify resistance is consistently applied
- Check if there's a race condition or edge case
- Test with multiple damage sources

---

### 2. **CRITICAL: Illegal Bonus Actions (Rule Violation)**
**Status:** ❌ Broken  
**Impact:** Characters getting free extra attacks

**Issue:** Bonus Action economy is not enforced.
- Hunter's Mark costs: **1 Bonus Action**
- Crossbow Expert Attack costs: **1 Bonus Action**
- Andreas is using both in the same turn

**Evidence:**
```
Andreas... Uses Action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark
```

**Fix Required:**
- Add `bonus_action_used` boolean to turn state
- Reset at start of each turn
- Check before allowing bonus action spells/attacks
- Hunter's Mark and Offhand Attack must check this flag

**Files to Modify:**
- `simulation-wasm/src/simulation.rs` - Add bonus action tracking
- `simulation-wasm/src/model.rs` - Add to turn state
- `simulation-wasm/src/resolution.rs` - Check before bonus actions

---

### 3. **AI Loop: Rage Spam / Persistence**
**Status:** ❌ Broken  
**Impact:** Wasted actions, incorrect buff application

**Issue:** Rage is being cast multiple times when it should persist.
- Pre-Combat: Correctly skips Rage (condition check works)
- Round 1: Uses Reckless Attack (Rage should be active from pre-combat)
- Round 2: Casts Rage AGAIN (wastes bonus action)

**D&D 5e Rule:** Rage ends if you don't attack or take damage. If you attack, it persists.

**Fix Required:**
- Check if buff is already active before casting
- Verify buff duration logic ("entire encounter" should persist)
- Fix AI action selection to skip active buffs

**Files to Modify:**
- `simulation-wasm/src/actions.rs` - Add condition check for active buffs
- AI action selection logic

---

### 4. **Target Finder Bug (Ranged Attacks)**
**Status:** ❌ Broken  
**Impact:** Characters losing 66% of their attacks

**Issue:** `get_targets` returns Empty/Null for ranged characters mid-turn.

**Evidence:**
```
Round 3: Andreas attacks Acoyth Debuff, misses
  -> No targets available (x2)
```
But `Acoyth Debuff` was alive (HP 6). Van hits it immediately after.

**Fix Required:**
- Debug `get_targets` function for ranged weapons
- Check if target filtering differs for ranged vs melee
- Ensure targets aren't removed from pool mid-turn

**Files to Modify:**
- `simulation-wasm/src/targeting.rs` - Fix ranged target selection

---

### 5. **AI Loop: Concentration Spell Spam**
**Status:** ❌ Broken  
**Impact:** Wasted turns, poor tactical decisions

**Issue:** AI casts concentration spells repeatedly, overwriting previous instances.

**Evidence:**
```
Round 2: Acoyth Debuff casts Bane
Round 3: Acoyth Debuff casts Bane AGAIN
```

**Fix Required:**
- Check `is_concentrating` before casting concentration spell
- AI should prioritize other actions if already concentrating

**Files to Modify:**
- `simulation-wasm/src/actions.rs` - Add concentration check to condition
- AI action selection logic

---

## 📋 Implementation Checklist

- [ ] **Bug 1: Resistance Verification**
  - [ ] Add comprehensive test cases
  - [ ] Verify rage/resistance interaction
  - [ ] Check edge cases (multiple damage sources, crits)

- [ ] **Bug 2: Bonus Action Economy**
  - [ ] Add `bonus_action_used` to turn state
  - [ ] Implement checks in action resolution
  - [ ] Update Hunter's Mark logic
  - [ ] Update Crossbow Expert logic

- [ ] **Bug 3: Rage Persistence**
  - [ ] Fix AI to check active buffs before casting
  - [ ] Verify "entire encounter" duration works correctly
  - [ ] Test rage ending conditions

- [ ] **Bug 4: Target Finder**
  - [ ] Debug ranged attack targeting
  - [ ] Add logging to `get_targets`
  - [ ] Test with multiple ranged attackers

- [ ] **Bug 5: Concentration Checks**
  - [ ] Add `is_concentrating` check to AI
  - [ ] Prevent duplicate concentration spell casts
  - [ ] Test with multiple concentration users

---

## 🔧 Verification Plan

After fixes:
1. Run simulation with Kaelen battle scenario
2. Verify resistance calculations in logs
3. Check bonus action usage (should be 1 per turn max)
4. Verify Rage only casts once
5. Check ranged attacks hit all intended targets
6. Verify concentration spells aren't duplicated

---

**Last Updated:** 2025-12-02  
**Status:** Critical bugs documented, ready for implementation