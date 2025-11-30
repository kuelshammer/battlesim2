# Buff/Debuff Cleanup Logic: Comprehensive Analysis

## Overview

The simulation has **THREE SEPARATE PATHS** for removing buffs when concentration is broken or a caster dies:

1. **During Combat (Per-Round Cleanup)** - in `simulation.rs`
2. **During Damage (Concentration Checks)** - in `simulation.rs` and `actions.rs`
3. **During Aggregation (Post-Processing)** - in `aggregation.rs`

---

## 1. Per-Round Cleanup (End of Round)

**Location**: [simulation.rs:239-248](file:///Users/max/Rust/Battlesim2/simulation-wasm/src/simulation.rs#L239-L248)

**When**: After all combatants have taken their turns, at the end of each round.

**Logic**:
```rust
// Remove buffs from dead sources (at end of round)
let mut dead_ids = HashSet::new();
for c in t1.iter().chain(t2.iter()) {
    if c.final_state.current_hp <= 0.0 {
        dead_ids.insert(c.id.clone());
    }
}

remove_dead_buffs(&mut t1, &dead_ids);
remove_dead_buffs(&mut t2, &dead_ids);
```

**Threshold**: `HP <= 0.0`

**Implementation**: [cleanup.rs:4-38](file:///Users/max/Rust/Battlesim2/simulation-wasm/src/cleanup.rs#L4-L38)
```rust
pub fn remove_dead_buffs(targets: &mut [Combattant], dead_source_ids: &HashSet<String>) {
    for target in targets.iter_mut() {
        target.final_state.buffs.retain(|buff_id, buff| {
            if let Some(source) = &buff.source {
                let should_keep = !dead_source_ids.contains(source);
                if !should_keep {
                    eprintln!("CLEANUP: Removing buff '{}' from {} (source {} is dead)",
                        buff_id, target.creature.name, source);
                }
                should_keep
            } else {
                // Buff with no source is always kept (might be innate effects)
                true
            }
        });
    }
}
```

**What it does**: Removes ALL buffs (concentration or not) from ALL targets if the source combatant has `HP <= 0.0`.

**Limitation**: Only runs at **END OF ROUND**, not when a caster dies mid-round.

---

## 2. Concentration Checks During Combat

### 2a. When Caster Takes Damage

**Location**: [simulation.rs:390-404](file:///Users/max/Rust/Battlesim2/simulation-wasm/src/simulation.rs#L390-L404) (enemy) and [simulation.rs:412-423](file:///Users/max/Rust/Battlesim2/simulation-wasm/src/simulation.rs#L412-L423) (ally)

**When**: Immediately after a combatant takes damage from an attack.

**Logic**:
```rust
// Concentration Check
if let Some(buff_id) = enemies[target_idx].final_state.concentrating_on.clone() {
    if enemies[target_idx].final_state.current_hp <= 0.0 {
        // If dead, automatically break concentration
        break_concentration(&enemies[target_idx].id.clone(), &buff_id, allies, enemies);
    } else {
        // If alive, roll Constitution save
        let con_save_bonus = enemies[target_idx].creature.con_save_bonus
            .unwrap_or(enemies[target_idx].creature.save_bonus);
        let dc = (dmg / 2.0).max(10.0);
        let roll = rand::thread_rng().gen_range(1..=20) as f64;
        if roll + con_save_bonus < dc {
            let caster_id = enemies[target_idx].id.clone();
            break_concentration(&caster_id, &buff_id, allies, enemies);
        }
    }
}
```

**Triggers**:
1. **Death**: If `HP <= 0.0` after damage
2. **Failed Save**: If alive but fails Constitution save (DC = max(10, damage/2))

### 2b. Break Concentration Function

**Location**: [actions.rs:86-113](file:///Users/max/Rust/Battlesim2/simulation-wasm/src/actions.rs#L86-L113)

**Logic**:
```rust
pub fn break_concentration(caster_id: &str, buff_id: &str, allies: &mut [Combattant], enemies: &mut [Combattant]) {
    // 1. Clear concentration on caster
    for c in allies.iter_mut().chain(enemies.iter_mut()) {
        if c.id == caster_id {
            c.final_state.concentrating_on = None;
        }
    }

    // 2. Remove the specific buff from all combatants
    for c in allies.iter_mut().chain(enemies.iter_mut()) {
        let should_remove = if let Some(buff) = c.final_state.buffs.get(buff_id) {
            buff.source.as_ref() == Some(&caster_id.to_string())
        } else {
            false
        };

        if should_remove {
            c.final_state.buffs.remove(buff_id);
            eprintln!("Removed {} from {}.", buff_id, c.creature.name);
        }
    }
}
```

**What it does**:
1. Clears `concentrating_on` field on the caster
2. Removes **ONLY THE SPECIFIC BUFF** that was being concentrated on
3. Only removes it if `buff.source == caster_id`

**Limitation**: Only removes the **single buff** being concentrated on, not all buffs from that caster.

### 2c. Concentration Replacement (New Spell)

**Location**: [simulation.rs:442-448](file:///Users/max/Rust/Battlesim2/simulation-wasm/src/simulation.rs#L442-L448) (Buff) and [simulation.rs:461-467](file:///Users/max/Rust/Battlesim2/simulation-wasm/src/simulation.rs#L461-L467) (Debuff)

**When**: When a caster casts a new concentration spell.

**Logic**:
```rust
if a.buff.concentration {
    let current_concentration = allies[attacker_idx].final_state.concentrating_on.clone();
    if let Some(old_buff) = current_concentration {
         let caster_id = allies[attacker_idx].id.clone();
         break_concentration(&caster_id, &old_buff, allies, enemies);
    }
    allies[attacker_idx].final_state.concentrating_on = Some(a.base().id.clone());
}
```

**What it does**: If already concentrating on something, breaks the old concentration before applying the new buff.

---

## 3. Aggregation Cleanup (Post-Processing)

**Location**: [aggregation.rs:197-299](file:///Users/max/Rust/Battlesim2/simulation-wasm/src/aggregation.rs#L197-L299)

**When**: After all simulation runs are complete, during the aggregation of results for display.

**Threshold**: `HP < 0.5` (different from per-round cleanup!)

**Logic** (Two-pass system):

### Pass 1: Dead Source Cleanup
```rust
// 1. Identify dead combatants (HP < 0.5)
let mut dead_source_ids = HashSet::new();
for c in t1.iter_mut().chain(t2.iter_mut()) {
    if c.final_state.current_hp < 0.5 {
        c.final_state.concentrating_on = None;
        dead_source_ids.insert(c.id.clone());
    }
}

// 2. Remove ALL buffs from dead sources
for c in t1.iter_mut().chain(t2.iter_mut()) {
    c.final_state.buffs.retain(|buff_id, buff| {
        if let Some(source) = &buff.source {
            if dead_source_ids.contains(source) {
                return false; // Remove buff
            }
        }
        true
    });
}
```

### Pass 2: Concentration Verification
```rust
// Build concentration map
let mut concentration_map: HashMap<String, Option<String>> = HashMap::new();
for c in t1.iter().chain(t2.iter()) {
    concentration_map.insert(c.id.clone(), c.final_state.concentrating_on.clone());
}

// Remove concentration buffs if source isn't concentrating on them
for c in t1.iter_mut().chain(t2.iter_mut()) {
    c.final_state.buffs.retain(|buff_id, buff| {
        if let Some(source) = &buff.source {
            if buff.concentration {
                if let Some(source_concentrating) = concentration_map.get(source) {
                    let is_concentrating_on_this = source_concentrating.as_ref() == Some(buff_id);
                    if !is_concentrating_on_this {
                        return false; // Remove buff
                    }
                }
            }
        }
        true
    });
}
```

**What it does**:
1. Removes ALL buffs from dead sources (`HP < 0.5`)
2. Removes concentration buffs if the source is not concentrating on that specific buff
3. Verifies concentration consistency

---

## Critical Issues Identified

### ❌ Issue 1: Threshold Mismatch
- **Per-round cleanup**: Uses `HP <= 0.0`
- **Aggregation cleanup**: Uses `HP < 0.5`
- **Result**: In aggregation, averaged HP might be 0.2 (nearly dead across runs), but per-round cleanup never triggered because individual runs had `HP = 0.0` (exactly)

### ❌ Issue 2: Mid-Round Death Not Handled
- **Per-round cleanup**: Only runs at **END** of round
- **Concentration check**: Only removes the **ONE** buff being concentrated on
- **Result**: If a caster dies mid-round and had **multiple** concentration spells active (e.g., from previous rounds), the **other** buffs persist until end of round

### ❌ Issue 3: Concentration Breaking is Too Narrow
- `break_concentration()` only removes **ONE SPECIFIC BUFF** (the one being concentrated on)
- **Expected**: When a caster dies, ALL their buffs should be removed
- **Actual**: Only the buff they were actively concentrating on at death is removed

### ❌ Issue 4: Non-Concentration Buffs from Dead Casters
- If a caster had both concentration and non-concentration buffs active, only concentration buffs are removed via `break_concentration()`
- Non-concentration buffs persist until end-of-round cleanup

---

## What SHOULD Happen (D&D 5e Rules)

### When a Caster Takes Damage:
1. If damage reduces HP to 0: **ALL spells end immediately** (death)
2. If damage but still alive: Make Constitution save (DC = max(10, damage/2))
   - Success: Maintain concentration
   - Failure: **ONLY the spell being concentrated on ends**

### When a Caster Casts a New Concentration Spell:
- Previous concentration spell ends
- New spell begins

### When Aggregating Results:
- Average HP across runs
- If averaged HP indicates "effectively dead" (< 0.5), remove all buffs
- Verify concentration consistency

---

## Recommended Fixes

### Fix 1: Immediate Death Handling
When `HP <= 0.0` after damage in `execute_turn`, call a new function `remove_all_buffs_from_source()` instead of just `break_concentration()`.

### Fix 2: Unified Threshold
Change per-round cleanup threshold from `HP <= 0.0` to `HP < 0.5` to match aggregation.

### Fix 3: New Function: `remove_all_buffs_from_source()`
```rust
pub fn remove_all_buffs_from_source(source_id: &str, allies: &mut [Combattant], enemies: &mut [Combattant]) {
    // Clear concentration
    for c in allies.iter_mut().chain(enemies.iter_mut()) {
        if c.id == source_id {
            c.final_state.concentrating_on = None;
        }
    }
    
    // Remove ALL buffs from this source
    for c in allies.iter_mut().chain(enemies.iter_mut()) {
        c.final_state.buffs.retain(|_, buff| {
            buff.source.as_ref() != Some(&source_id.to_string())
        });
    }
}
```

### Fix 4: Update Death Detection
Replace `break_concentration()` calls with `remove_all_buffs_from_source()` when `HP <= 0.0`.
