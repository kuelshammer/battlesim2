# Bless & Bane Implementation Plan

## Current Status Analysis

### ✅ What's Working:
- **Bless casting is logged**: `"-> Casts Bless on Target"`
- **Debuff attempts with saves**: Detailed DC, save rolls, success/failure
- **Dice formula system**: Handles `1d4[Bless]` correctly for per-attack rolling
- **Buff application**: Basic application in trigger effects
- **Concentration breaking**: Logged when concentration drops

### ❌ What's Missing:
1. **Bane debuff implementation** - Missing entirely from Rust backend
2. **Bless bonus visibility** - +1d4 bonus application not shown in combat logs
3. **Buff effect details** - Mechanical effects (AC bonuses, to-hit bonuses) not displayed
4. **Buff duration information** - No indication how long buffs last
5. **Buff removal** - Expiration/removal not logged
6. **Multi-target details** - Individual applications for spells like Bless not shown

## Implementation Plan

### Phase 1: Bane Debuff Implementation
**Files to modify**: `simulation-wasm/src/resolution.rs`, `simulation-wasm/src/simulation.rs`

1. **Add Bane debuff logic**:
   - Implement -1d4 penalty to attack rolls for targets under Bane
   - Implement -1d4 penalty to saving throws for targets under Bane
   - Handle concentration mechanics (caster vs target concentration)

2. **Integration points**:
   - Modify attack roll calculation to check for Bane conditions
   - Modify saving throw calculation to check for Bane conditions
   - Ensure proper stack order with other modifiers

### Phase 2: Enhanced Combat Logging
**File to modify**: `simulation-wasm/src/resolution.rs`

1. **Buff Effect Logging**:
   - Log when Bless bonus is applied: `"Bless adds +1d4 to attack roll"`
   - Log when Bane penalty is applied: `"Bane subtracts 1d4 from attack roll"`
   - Log bonus/penalty results: `"Bless result: +3 (rolled 1d4=3)"`

2. **Detailed Casting Logs**:
   - Show duration: `"Bless (Concentration, 1 min) cast on [targets]"`
   - Show mechanical effects: `"Grants +1d4 to attack rolls and saving throws"`
   - Individual target applications for multi-target spells

3. **Buff Lifecycle Logging**:
   - Log when buffs expire: `"Bless effect expired on [target]"`
   - Log when concentration broken: `"Concentration broken - Bless ends"`
   - Log when caster dies: `"Bless ended - caster deceased"`

### Phase 3: Testing & Validation

1. **Create Test Scenarios**:
   - Acolyte casting Bless on multiple allies
   - Enemy casting Bane on party members
   - Concentration management under fire
   - Bless + Bane interaction testing

2. **Validation Points**:
   - Verify Bless +1d4 appears in attack logs
   - Verify Bane -1d4 appears in attack/save logs
   - Check concentration mechanics work correctly
   - Ensure proper bonus/penalty stacking

## Technical Details

### Bless Bonus Implementation (Already Working)
- Formula: `"base_attack_bonus + 1d4[Bless]"`
- Location: `dice.rs` lines 99-104 handle per-attack rolling
- Status: ✅ Functional, just needs logging enhancement

### Bane Penalty Implementation (Missing)
- Formula: `"base_attack_bonus - 1d4[Bane]"`
- Need to implement: Negative dice handling in formula system
- Integration points: Attack rolls and saving throws

### Logging Enhancement Strategy
- Add detailed logging at points where buffs are evaluated
- Use existing log infrastructure in `resolution.rs`
- Maintain backward compatibility with current log format

## Expected Outcomes

After implementation, combat logs will show:

```
-> Acolyte casts Bless (Concentration, 1 min) on [Player Fighter, Rogue]
   Bless grants +1d4 to attack rolls and saving throws

Player Fighter attacks Goblin with advantage:
   Base attack: +5, Bless bonus: +3 (rolled 1d4=3), Total: +8
   Hit! Goblin takes 12 damage

-> Enemy Mage casts Bane (DC 12) on [Player Fighter, Cleric]
   Player Fighter save: 8 - 1d4(Bane)=6 - FAILS
   Cleric save: 15 - 1d4(Bane)=13 - SUCCEEDS
   Player Fighter affected by Bane (-1d4 to attacks/saves)
```

This provides complete visibility into Bless/Bane mechanics while maintaining the existing dice rolling system that correctly handles per-attack randomization.