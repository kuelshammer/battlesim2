# Additional Action Trigger Implementation Plan

## Objective
Implement a system for "Reaction" and "Conditional" actions (Triggers) that occur outside the standard "Choose Action -> Execute" flow. This enables features like **Divine Smite** (On Hit), **Shield Spell** (On Being Attacked), and **Opportunity Attacks** (future scope).

## 1. Data Model Changes (`model.rs` / `enums.rs`)

We need a new structure to hold these "conditional actions" distinct from the main `actions` list which is used for the active turn.

### 1.1 New Enums
Add `TriggerCondition` to `enums.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerCondition {
    #[serde(rename = "on hit")]
    OnHit, // e.g. Divine Smite
    #[serde(rename = "on being attacked")]
    OnBeingAttacked, // e.g. Shield Spell, Cutting Words
    #[serde(rename = "on being damaged")]
    OnBeingDamaged, // e.g. Absorb Elements
    #[serde(rename = "on ally attacked")]
    OnAllyAttacked, // e.g. Sentinel
}
```

### 1.2 New Structs
Add `ActionTrigger` to `model.rs`. This wraps an existing `Action` but defines *when* it is used.
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTrigger {
    pub id: String,
    pub condition: TriggerCondition,
    pub action: Action, // The action to execute when triggered
    pub cost: Option<ActionSlot>, // e.g. Reaction (4)
}
```

Add `triggers` field to `Creature` struct in `model.rs`:
```rust
pub struct Creature {
    // ... existing fields
    pub triggers: Vec<ActionTrigger>,
}
```

## 2. Simulation Logic Updates (`simulation.rs`)

We need to inject "Hook Points" into the `execute_turn` function.

### 2.1 Hook 1: Pre-Attack Hit Check (Defense/Reaction)
*Location:* Inside `execute_turn`, after `Action::Atk` target selection, but *before* calculating `hits`.

**Logic:**
1. Identify the target.
2. Iterate through target's `triggers`.
3. Filter for `TriggerCondition::OnBeingAttacked`.
4. Check if trigger is usable (resource cost, reaction slot available).
5. **Execute** the trigger action (e.g., apply `Shield` buff).
   - *Note:* This requires a mini-execution context. If the action is a Buff, apply it immediately.
6. Recalculate `total_ac` for the attack using the new state.

### 2.2 Hook 2: Post-Hit Confirmation (Offense/Rider)
*Location:* Inside `execute_turn`, inside the `if hits { ... }` block, *before* damage application.

**Logic:**
1. Iterate through attacker's `triggers`.
2. Filter for `TriggerCondition::OnHit`.
3. Check if usable (resource cost, slots).
4. **Execute** trigger action.
   - If it's a Damage action (like Smite), add its damage to the current attack's total.
   - If it's a Debuff action (like Smite spell effects), apply it.

## 3. AI / Decision Logic

For the MVP, the AI will be "Eager":
- **OnHit (Smite):** Always use if resource available (and maybe check if target is not already dead/low HP optimization later).
- **OnBeingAttacked (Shield):** Only use if `roll >= AC` AND `roll < AC + Bonus`. (Smart usage).

## 4. Implementation Phases

### Phase 1: Data Structure & Loading
- Update `enums.rs` and `model.rs`.
- Update `data.ts` (frontend) to support defining these triggers (or just manual JSON for testing first).
- **Deliverable:** Compiling code with new fields.

### Phase 2: The Defensive Hook (Shield Spell)
- Implement `check_defensive_triggers` in `simulation.rs`.
- Test case: Wizard with Shield spell vs Fighter.
- **Deliverable:** Wizard AC increases dynamically when hit.

### Phase 3: The Offensive Hook (Divine Smite)
- Implement `check_offensive_triggers` in `simulation.rs`.
- Test case: Paladin Smites on hit.
- **Deliverable:** Paladin deals extra damage only on hits.

## 5. Future Considerations
- **Opportunity Attacks:** Requires "Movement" simulation (out of scope currently).
- **Counterspell:** Requires "OnCast" trigger.
