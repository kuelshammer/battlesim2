# Legacy Action Migration Guide

## Overview

Three TypeScript files were temporarily excluded from type checking (`@ts-nocheck`) during Phase 5 because they contain actions using the legacy schema that's missing required fields. This document outlines the migration work needed.

## Files Requiring Migration

### 1. `src/data/data.ts`
- **Actions to migrate**: ~37 actions across multiple character classes
- **Characters affected**: Artificer, Barbarian, Bard, Cleric, Druid, Fighter, Monk, Paladin, Ranger, Rogue, Sorcerer, Warlock, Wizard
- **Status**: ❌ Excluded with `@ts-nocheck`

### 2. `src/data/monsters.ts`
- **Actions to migrate**: All monster actions (100+ entries)
- **Monsters affected**: All predefined monsters in the bestiary
- **Status**: ❌ Excluded with `@ts-nocheck`

### 3. `src/model/simulation.ts`
- **Type**: Legacy simulation engine
- **Status**: ❌ Excluded - replaced by WASM backend
- **Note**: This file can likely be deleted in the future, but kept for reference

## Required Schema Changes

Each action needs three new required fields:

### 1. `cost` Field
Convert `actionSlot` to a structured cost array.

**Mapping**:
```typescript
ActionSlots.Action           → [{ type: 'Discrete', resourceType: 'Action', amount: 1 }]
ActionSlots['Bonus Action']  → [{ type: 'Discrete', resourceType: 'BonusAction', amount: 1 }]
ActionSlots.Reaction         → [{ type: 'Discrete', resourceType: 'Reaction', amount: 1 }]
ActionSlots['Other 1']       → [{ type: 'Discrete', resourceType: 'SpellSlot', amount: 1 }]
ActionSlots['Other 2']       → [{ type: 'Discrete', resourceType: 'SpellSlot', amount: 1 }]
ActionSlots['Before the Encounter'] → []
ActionSlots['When reducing an enemy to 0 HP'] → []
ActionSlots['When Reduced to 0 HP'] → []
```

### 2. `requirements` Field
Add an empty array for now (can be populated later):
```typescript
requirements: []
```

### 3. `tags` Field
Add appropriate tags based on action type:

**By Type**:
- `atk` → `['Attack', 'Damage']`
- `heal` (normal) → `['Healing']`
- `heal` (tempHP) → `['TempHP']`
- `buff` → `['Buff', 'Support']`
- `debuff` → `['Debuff', 'Control']`
- `template` → `['Spell']`

**Additional Tags** (based on context):
- Fire damage → add `'Fire'`
- Spell actions → add `'Spell'`
- Concentration → add `'Concentration'`
- Abjuration/Evocation/etc → add school name

## Migration Examples

### Before (Legacy)
```typescript
{
    id: uuid(),
    name: 'Firebolt',
    actionSlot: ActionSlots.Action,
    type: 'atk',
    freq: 'at will',
    targets: 1,
    target: 'enemy with least HP',
    toHit: toHit,
    dpr: fireBolt + arcaneFireArm,
    condition: 'default',
}
```

### After (New Schema)
```typescript
{
    id: uuid(),
    name: 'Firebolt',
    actionSlot: ActionSlots.Action,
    type: 'atk',
    freq: 'at will',
    cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
    requirements: [],
    tags: ['Attack', 'Damage', 'Fire', 'Spell'],
    targets: 1,
    target: 'enemy with least HP',
    toHit: toHit,
    dpr: fireBolt + arcaneFireArm,
    condition: 'default',
}
```

### Template Action Example

**Before**:
```typescript
{
    id: uuid(),
    type: 'template',
    freq: { reset: 'lr', uses: 1 },
    condition: 'is under half HP',
    templateOptions: { templateName: 'Shield' },
}
```

**After**:
```typescript
{
    id: uuid(),
    type: 'template',
    freq: { reset: 'lr', uses: 1 },
    condition: 'is under half HP',
    cost: [{ type: 'Discrete', resourceType: 'Reaction', amount: 1 }],
    requirements: [],
    tags: ['Spell', 'Abjuration', 'Defense'],
    templateOptions: { templateName: 'Shield' },
}
```

## Migration Process

### Option 1: Manual Migration (Thorough)
1. Pick a character class in `data.ts`
2. For each action in that class:
   - Add `cost` field based on `actionSlot`
   - Add `requirements: []`
   - Add appropriate `tags` array
3. Test that the character loads without errors
4. Repeat for next character class

**Estimated Time**: ~2-3 hours for all characters

### Option 2: Semi-Automated (Faster)
1. Create a migration script that:
   - Parses each action object
   - Adds the three required fields
   - Preserves all existing fields
2. Run the script with manual verification
3. Fix any edge cases manually

**Estimated Time**: ~1 hour + script development

### Option 3: Incremental (As Needed)
1. Keep files excluded with `@ts-nocheck`
2. Migrate individual characters only when actively working with them
3. Remove `@ts-nocheck` once all actions in a file are migrated

**Estimated Time**: Ongoing, as needed

## Validation Checklist

After migrating a file:

- [ ] Remove `@ts-nocheck` comment
- [ ] Run `npm run build` - should compile without errors
- [ ] Load the character/monster in the UI
- [ ] Verify actions display correctly in ActionForm
- [ ] Check that StrategyBuilder shows all actions
- [ ] Ensure cost/requirements/tags appear in UI

## Known Issues

1. **Spell Slot Costs**: Current mapping uses generic `SpellSlot`, but spell actions should specify the level:
   ```typescript
   cost: [{ 
     type: 'Discrete', 
     resourceType: 'SpellSlot', 
     amount: 1,
     level: 3  // Add level field for spells
   }]
   ```

2. **Class Resources**: Some actions should consume class-specific resources (Ki, Bardic Inspiration, etc.) instead of generic action economy slots.

3. **Multiple Costs**: Some actions might have multiple costs (e.g., Action + Spell Slot), which the schema supports but legacy actions don't specify.

## Future Enhancements

Once migration is complete:

1. **Enhanced Tags**: Add more specific tags for better action categorization
2. **Requirements**: Populate actual requirements (e.g., "target must be concentrating")
3. **Proper Spell Costs**: Specify spell slot levels for spellcasting actions
4. **Class Resources**: Define Ki, Rage, Bardic Inspiration as proper resource types
5. **Delete Legacy**: Remove `simulation.ts` once confirmed WASM backend handles everything

## Questions?

If you encounter issues during migration:
- Check existing migrated actions in `src/data/actions.ts` for reference
- The new schema is defined in `src/model/model.ts` (search for `ActionSchema`)
- Cost structure is in `src/model/enums.ts` (search for `ActionCost`)
