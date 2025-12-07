// src/model/spellSlots.ts
// D&D 5e 2014 Spell Slot Progression

export type CasterType = 'full' | 'half' | 'third' | 'pact' | 'artificer';

// Full caster spell slot table (Wizard, Cleric, Druid, Bard, Sorcerer)
const FULL_CASTER_SLOTS: Record<number, Record<number, number>> = {
    1: { 1: 2 },
    2: { 1: 3 },
    3: { 1: 4, 2: 2 },
    4: { 1: 4, 2: 3 },
    5: { 1: 4, 2: 3, 3: 2 },
    6: { 1: 4, 2: 3, 3: 3 },
    7: { 1: 4, 2: 3, 3: 3, 4: 1 },
    8: { 1: 4, 2: 3, 3: 3, 4: 2 },
    9: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 1 },
    10: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2 },
    11: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2, 6: 1 },
    12: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2, 6: 1 },
    13: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2, 6: 1, 7: 1 },
    14: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2, 6: 1, 7: 1 },
    15: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2, 6: 1, 7: 1, 8: 1 },
    16: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2, 6: 1, 7: 1, 8: 1 },
    17: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2, 6: 1, 7: 1, 8: 1, 9: 1 },
    18: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 3, 6: 1, 7: 1, 8: 1, 9: 1 },
    19: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 3, 6: 2, 7: 1, 8: 1, 9: 1 },
    20: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 3, 6: 2, 7: 2, 8: 1, 9: 1 },
};

// Half caster spell slot table (Paladin, Ranger)
// Artificer has its own table as it starts spellcasting at level 1
const HALF_CASTER_SLOTS: Record<number, Record<number, number>> = {
    // Level 1: No spells for Paladin/Ranger
    2: { 1: 2 },
    3: { 1: 3 },
    4: { 1: 3 },
    5: { 1: 4, 2: 2 },
    6: { 1: 4, 2: 2 },
    7: { 1: 4, 2: 3 },
    8: { 1: 4, 2: 3 },
    9: { 1: 4, 2: 3, 3: 2 },
    10: { 1: 4, 2: 3, 3: 2 },
    11: { 1: 4, 2: 3, 3: 3 },
    12: { 1: 4, 2: 3, 3: 3 },
    13: { 1: 4, 2: 3, 3: 3, 4: 1 },
    14: { 1: 4, 2: 3, 3: 3, 4: 1 },
    15: { 1: 4, 2: 3, 3: 3, 4: 2 },
    16: { 1: 4, 2: 3, 3: 3, 4: 2 },
    17: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 1 },
    18: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 1 },
    19: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2 },
    20: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2 },
};

// Artificer spell slot table (unique progression, starts at level 1)
const ARTIFICER_CASTER_SLOTS: Record<number, Record<number, number>> = {
    1: { 1: 2 },
    2: { 1: 2 },
    3: { 1: 3 },
    4: { 1: 3 },
    5: { 1: 4, 2: 2 },
    6: { 1: 4, 2: 2 },
    7: { 1: 4, 2: 3 },
    8: { 1: 4, 2: 3 },
    9: { 1: 4, 2: 3, 3: 2 },
    10: { 1: 4, 2: 3, 3: 2 },
    11: { 1: 4, 2: 3, 3: 3 },
    12: { 1: 4, 2: 3, 3: 3 },
    13: { 1: 4, 2: 3, 3: 3, 4: 1 },
    14: { 1: 4, 2: 3, 3: 3, 4: 1 },
    15: { 1: 4, 2: 3, 3: 3, 4: 2 },
    16: { 1: 4, 2: 3, 3: 3, 4: 2 },
    17: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 1 },
    18: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 1 },
    19: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2 },
    20: { 1: 4, 2: 3, 3: 3, 4: 3, 5: 2 },
};

// Third caster spell slot table (Eldritch Knight, Arcane Trickster)
const THIRD_CASTER_SLOTS: Record<number, Record<number, number>> = {
    3: { 1: 2 },
    4: { 1: 3 },
    5: { 1: 3 },
    6: { 1: 3 },
    7: { 1: 4, 2: 2 },
    8: { 1: 4, 2: 2 },
    9: { 1: 4, 2: 2 },
    10: { 1: 4, 2: 3 },
    11: { 1: 4, 2: 3 },
    12: { 1: 4, 2: 3 },
    13: { 1: 4, 2: 3, 3: 2 },
    14: { 1: 4, 2: 3, 3: 2 },
    15: { 1: 4, 2: 3, 3: 2 },
    16: { 1: 4, 2: 3, 3: 3 },
    17: { 1: 4, 2: 3, 3: 3 },
    18: { 1: 4, 2: 3, 3: 3 },
    19: { 1: 4, 2: 3, 3: 3, 4: 1 },
    20: { 1: 4, 2: 3, 3: 3, 4: 1 },
};

// Warlock Pact Magic slots (all slots at same level, refresh on short rest)
const PACT_MAGIC_SLOTS: Record<number, { slots: number; level: number }> = {
    1: { slots: 1, level: 1 },
    2: { slots: 2, level: 1 },
    3: { slots: 2, level: 2 },
    4: { slots: 2, level: 2 },
    5: { slots: 2, level: 3 },
    6: { slots: 2, level: 3 },
    7: { slots: 2, level: 4 },
    8: { slots: 2, level: 4 },
    9: { slots: 2, level: 5 },
    10: { slots: 2, level: 5 },
    11: { slots: 3, level: 5 },
    12: { slots: 3, level: 5 },
    13: { slots: 3, level: 5 },
    14: { slots: 3, level: 5 },
    15: { slots: 3, level: 5 },
    16: { slots: 3, level: 5 },
    17: { slots: 4, level: 5 },
    18: { slots: 4, level: 5 },
    19: { slots: 4, level: 5 },
    20: { slots: 4, level: 5 },
};

/**
 * Calculate spell slots for a given caster level and type
 * Returns a record of spell level -> number of slots
 * Keys are formatted as "level_X" to match the existing spellSlots schema
 */
export function calculateSpellSlots(
    casterLevel: number,
    casterType: CasterType = 'full'
): Record<string, number> {
    if (casterLevel < 1 || casterLevel > 20) {
        return {};
    }

    const result: Record<string, number> = {};

    if (casterType === 'pact') {
        const pactSlots = PACT_MAGIC_SLOTS[casterLevel];
        if (pactSlots) {
            result[`level_${pactSlots.level}`] = pactSlots.slots;
        }
        return result;
    }

    let slotTable: Record<number, Record<number, number>>;
    switch (casterType) {
        case 'half':
            slotTable = HALF_CASTER_SLOTS;
            break;
        case 'third':
            slotTable = THIRD_CASTER_SLOTS;
            break;
        case 'artificer': // Handle Artificer specifically
            slotTable = ARTIFICER_CASTER_SLOTS;
            break;
        default:
            slotTable = FULL_CASTER_SLOTS;
    }

    const slots = slotTable[casterLevel];
    if (slots) {
        for (const [level, count] of Object.entries(slots)) {
            result[`level_${level}`] = count;
        }
    }

    return result;
}

/**
 * Detect caster level from existing spell slots (reverse calculation)
 * Returns the detected level and type, or null if no exact match
 */
export function detectCasterLevel(
    spellSlots: Record<string, number> | undefined
): { level: number; type: CasterType } | null {
    if (!spellSlots || Object.keys(spellSlots).length === 0) {
        return null;
    }

    // Convert to numeric format for comparison
    const slots: Record<number, number> = {};
    for (const [key, value] of Object.entries(spellSlots)) {
        const level = parseInt(key.replace('level_', ''));
        if (!isNaN(level)) {
            slots[level] = value;
        }
    }

    // Try to match against full caster first (most common)
    for (let level = 20; level >= 1; level--) {
        const expectedSlots = FULL_CASTER_SLOTS[level];
        if (slotsMatch(slots, expectedSlots)) {
            return { level, type: 'full' };
        }
    }

    // Try half caster
    for (let level = 20; level >= 2; level--) { // Paladin/Ranger start spells at L2
        const expectedSlots = HALF_CASTER_SLOTS[level];
        if (expectedSlots && slotsMatch(slots, expectedSlots)) {
            return { level, type: 'half' };
        }
    }

    // Try artificer caster
    for (let level = 20; level >= 1; level--) { // Artificer starts spells at L1
        const expectedSlots = ARTIFICER_CASTER_SLOTS[level];
        if (expectedSlots && slotsMatch(slots, expectedSlots)) {
            return { level, type: 'artificer' };
        }
    }

    // Try third caster
    for (let level = 20; level >= 3; level--) {
        const expectedSlots = THIRD_CASTER_SLOTS[level];
        if (expectedSlots && slotsMatch(slots, expectedSlots)) {
            return { level, type: 'third' };
        }
    }

    return null;
}

function slotsMatch(
    actual: Record<number, number>,
    expected: Record<number, number> | undefined
): boolean {
    if (!expected) return false;

    const actualKeys = Object.keys(actual).map(Number).filter(n => actual[n] > 0);
    const expectedKeys = Object.keys(expected).map(Number);

    if (actualKeys.length !== expectedKeys.length) return false;

    return expectedKeys.every(level => actual[level] === expected[level]);
}

export const CASTER_TYPE_LABELS: Record<CasterType, string> = {
    'full': 'Full Caster (Wizard, Cleric, etc.)',
    'half': 'Half Caster (Paladin, Ranger)',
    'third': 'Third Caster (Eldritch Knight)',
    'pact': 'Pact Magic (Warlock)',
    'artificer': 'Artificer',
};
