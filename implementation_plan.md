# Implementation Plan - Battlesim Monte Carlo Upgrade

This document tracks the implementation of major features for the Battlesim Monte Carlo simulation engine.

## ✅ Completed Features

### 1. Monte Carlo Simulation Engine
- ✅ Rust/WASM backend implementation
- ✅ True RNG dice rolling system
- ✅ 1000+ simulation runs per execution
- ✅ Quintile-based result selection via "Luck" slider
- ✅ Deterministic combatant IDs for consistent aggregation

### 2. Concentration Mechanics
- ✅ Concentration tracking in creature state
- ✅ Automatic concentration breaking on damage (CON save)
- ✅ Concentration breaking on caster death
- ✅ Concentration conflict resolution (new spell replaces old)
- ✅ Duplicate concentration prevention (same spell on multiple targets)

### 3. Action Trigger System
- ✅ Defensive triggers (Shield spell on being attacked)
- ✅ Offensive triggers (Divine Smite on hit)
- ✅ Resource cost tracking for triggers
- ✅ Smart AC-based trigger activation
- ✅ Migrated Shield, Divine Smite, and Parry to trigger system

### 4. Pre-Combat Spell Casting ("Round 0")
- ✅ Actions with `actionSlot: -3` execute before initiative
- ✅ "Cast before combat" checkbox in UI
- ✅ Template support for Mage Armour, Armor of Agathys, False Life, Shield of Faith
- ✅ Pre-Combat Setup logging phase
- ✅ Amount override for template actions

### 5. Enhanced Combat Logging
- ✅ Bless/Bane detailed logging with bonus/penalty breakdowns
- ✅ Buff display name fallbacks (uses action name if no displayName)
- ✅ Save roll breakdowns showing base + buffs
- ✅ Concentration status in logs

### 6. Frontend Integration
- ✅ Direct single-run selection (removed aggregation)
- ✅ Luck slider selects from 1000+ real simulation runs
- ✅ Template action resolution before WASM call
- ✅ Stable combatant ID display

## 📋 Current Architecture

### Data Flow
1. **Frontend** (TypeScript) → Defines creatures, actions, triggers
2. **Template Resolution** → Converts template actions to final actions
3. **WASM Simulation** → Runs 1000+ Monte Carlo iterations
4. **Result Selection** → Luck slider picks one representative run
5. **Display** → Shows actual combat log from that specific run

### Key Files
- `simulation-wasm/src/simulation.rs` - Main simulation loop, pre-combat execution
- `simulation-wasm/src/resolution.rs` - Action resolution, triggers, enhanced logging
- `simulation-wasm/src/targeting.rs` - Target selection logic
- `src/data/actions.ts` - Action templates, including pre-combat spells
- `src/data/data.ts` - Class templates with triggers and pre-combat actions
- `src/components/creatureForm/actionForm.tsx` - Action editor with pre-combat checkbox

## 🎯 Design Decisions

### Monte Carlo vs Deterministic
- **Before**: Single deterministic run with weighted averages
- **After**: 1000+ real dice-rolled simulations, user selects representative outcome
- **Benefit**: More realistic, shows actual variance and edge cases

### Action Triggers
- **Pattern**: Hook-based system at critical points (pre-attack, post-hit)
- **Benefit**: Clean separation of reactive vs active actions
- **Future**: Extensible for opportunity attacks, counterspell, etc.

### Pre-Combat Spells
- **Approach**: Re-purposed existing `actionSlot: -3` constant
- **Benefit**: Minimal code changes, backward compatible
- **Future**: Could expand to multi-round pre-combat sequences

### Concentration
- **Rule**: One spell at a time, breaks on damage (CON save), death, or recasting
- **Special Case**: Same spell on multiple targets allowed (e.g., Bless on 3 allies)
- **Implementation**: Cleanup instructions pattern for deferred removal

## 🔧 Known Limitations

1. **Resource Tracking**: Spell slots not fully tracked (assumes infinite for templates)
2. **Movement**: No positioning or opportunity attacks
3. **Counterspell**: Requires OnCast trigger (not yet implemented)
4. **Lair Actions**: No multi-initiative system for complex encounters

## 📖 Reference Documents

For detailed historical context, see:
- `walkthrough.md` in `.gemini/antigravity/brain/` - Complete changelog of all fixes and features
- Git history for implementation details

---

**Last Updated**: 2025-12-02  
**Status**: Production-ready, all planned features implemented
