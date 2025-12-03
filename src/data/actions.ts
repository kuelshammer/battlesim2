import { v4 as uuid } from 'uuid'
import { Action, FinalAction } from "../model/model"
import { ActionSlots, AllyTarget, EnemyTarget } from '../model/enums';

export const ActionTemplates = {
    'Bane': createTemplate({
        actionSlot: ActionSlots.Action,
        // New Phase 2 fields
        cost: [
            { type: 'Discrete', resourceType: 'Action', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 1 } // Would need level info here in full implementation
        ],
        requirements: [],
        tags: ['Spell', 'Necromancy', 'Concentration', 'Control'],
        // Legacy fields
        type: 'debuff',
        targets: 3,
        target: 'enemy with least HP',
        buff: {
            displayName: 'Bane',
            duration: 'entire encounter',
            toHit: '-1d4',
            save: '-1d4',
            concentration: true,
        },

        saveDC: 0, // Replaced later
    }),
    'Bless': createTemplate({
        actionSlot: ActionSlots.Action,
        // New Phase 2 fields
        cost: [
            { type: 'Discrete', resourceType: 'Action', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 1 } // Level 1 spell slot
        ],
        requirements: [],
        tags: ['Spell', 'Concentration', 'Support', 'Healing'],
        // Legacy fields
        type: 'buff',
        targets: 3,
        target: 'ally with the least HP',
        buff: {
            displayName: 'Bless',
            duration: 'entire encounter',
            toHit: '1d4',
            save: '1d4',
            concentration: true,
        },
    }),
    'Haste': createTemplate({
        actionSlot: ActionSlots.Action,
        cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
        requirements: [],
        tags: ['Spell', 'Concentration', 'Support'],
        type: 'buff',
        targets: 1,
        buff: {
            displayName: 'Haste',
            duration: 'entire encounter', // 1 minute
            ac: '2',
            condition: 'Hasted',
            concentration: true,
        },
    }),
    'Holy Weapon': createTemplate({
        actionSlot: ActionSlots['Bonus Action'], // Keep for backward compatibility, but define new costs
        cost: [
            { type: 'Discrete', resourceType: 'BonusAction', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 5 } // 5th-level spell, but can be higher. Use base level for template
        ],
        requirements: [],
        tags: ['Spell', 'Concentration', 'Buff', 'Radiant'],
        type: 'buff',
        targets: 1,
        target: 'self', // Usually weapon, simplifying to self for now
        buff: {
            displayName: 'Holy Weapon',
            duration: 'entire encounter', // 1 hour
            damage: '2d8',
            concentration: true,
        },
    }),
    'Hunter\'s Mark': createTemplate({
        actionSlot: ActionSlots['Bonus Action'], // Keep for backward compatibility, but define new costs
        cost: [
            { type: 'Discrete', resourceType: 'BonusAction', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 1 } // 1st-level spell
        ],
        requirements: [],
        tags: ['Spell', 'Concentration', 'Buff', 'Damage'],
        type: 'buff',
        targets: 1,
        target: 'enemy with least HP', // Mark the priority target
        buff: {
            displayName: 'Hunter\'s Mark',
            duration: 'entire encounter', // Simplified from 1 hour
            damage: '1d6', // Extra damage on each weapon attack hit
            concentration: true,
        },
    }),
    'Fireball': createTemplate({
        actionSlot: ActionSlots.Action,
        // New Phase 2 fields
        cost: [
            { type: 'Discrete', resourceType: 'Action', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 3 } // Level 3 spell slot
        ],
        requirements: [],
        tags: ['Spell', 'Evocation', 'Fire', 'AoE', 'Attack'],
        // Legacy fields
        type: 'atk',
        targets: 2,
        useSaves: true,
        dpr: '8d6',
        halfOnSave: true,

        toHit: 0,
    }),
    'Heal': createTemplate({
        actionSlot: ActionSlots.Action, // Keep for backward compatibility
        cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
        requirements: [],
        tags: ['Healing', 'Support'],
        type: 'heal',
        targets: 1,
        amount: 70,
    }),
    'Hypnotic Pattern': createTemplate({
        actionSlot: ActionSlots.Action, // Keep for backward compatibility
        cost: [
            { type: 'Discrete', resourceType: 'Action', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 3 }
        ],
        requirements: [],
        tags: ['Spell', 'Concentration', 'AoE', 'Enchantment', 'Control'],
        type: 'debuff',
        targets: 4, // Area of effect approximation
        saveDC: 0,
        buff: {
            displayName: 'Hypnotic Pattern',
            duration: 'entire encounter', // 1 minute
            condition: 'Incapacitated',
            concentration: true,
        },
    }),
    'Meteor Swarm': createTemplate({
        actionSlot: ActionSlots.Action, // Keep for backward compatibility
        cost: [
            { type: 'Discrete', resourceType: 'Action', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 9 }
        ],
        requirements: [],
        tags: ['Spell', 'AoE', 'Evocation', 'Fire', 'Attack', 'Damage'],
        type: 'atk',
        targets: 4,
        useSaves: true,
        dpr: '20d6 + 20d6',
        halfOnSave: true,
        toHit: 0,
    }),
    'Greater Invisibility': createTemplate({
        actionSlot: ActionSlots.Action, // Keep for backward compatibility
        cost: [
            { type: 'Discrete', resourceType: 'Action', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 4 }
        ],
        requirements: [],
        tags: ['Spell', 'Concentration', 'Buff', 'Illusion'],
        type: 'buff',
        targets: 1,
        buff: {
            displayName: 'Greater Invisibility',
            duration: 'entire encounter', // 1 minute
            condition: 'Invisible',
            concentration: true,
        },
    }),
    'Shield': createTemplate({
        actionSlot: ActionSlots.Reaction, // Keep for backward compatibility
        cost: [
            { type: 'Discrete', resourceType: 'Reaction', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 1 }
        ],
        requirements: [],
        tags: ['Spell', 'Abjuration', 'Defense'],
        type: 'buff',
        targets: 1,
        target: 'self',
        buff: {
            displayName: 'Shield',
            duration: '1 round',
            ac: '5',
        },
    }),
    'Mage Armour': createTemplate({
        actionSlot: -3, // Pre-combat by default
        cost: [
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 1 }
        ],
        requirements: [],
        tags: ['Spell', 'Abjuration', 'Buff'],
        type: 'buff',
        targets: 1,
        target: 'self', // Can be touch, but self for simplicity for now
        buff: {
            displayName: 'Mage Armour',
            duration: 'entire encounter', // 8 hours
            ac: '3',
        },
    }),
        'Armor of Agathys': createTemplate({
            actionSlot: -3, // Pre-combat by default
            cost: [
                { type: 'Discrete', resourceType: 'SpellSlot', amount: 1 }
            ],
            requirements: [],
            tags: ['Spell', 'Abjuration', 'Buff', 'TempHP', 'Defense'],
            type: 'heal', // Using heal for THP application
            targets: 1,
            target: 'self',
            amount: 0, // Base amount, scaled by level usually
            tempHP: true,
            // Note: Damage reflection needs a specific implementation or trigger,
            // for now just the THP part or we add a buff with damage reflection if supported
            // The current system supports damageTakenMultiplier but not reflection directly in simple buffs
            // We might need a trigger for the reflection part.
        }),    'False Life': createTemplate({
        actionSlot: -3,
        cost: [
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 1 }
        ],
        requirements: [],
        tags: ['Spell', 'Necromancy', 'Buff', 'TempHP'],
        type: 'heal',
        targets: 1,
        target: 'self',
        amount: '1d4 + 4',
        tempHP: true,
    }),
    'Shield of Faith': createTemplate({
        actionSlot: ActionSlots['Bonus Action'], // Keep for backward compatibility
        cost: [
            { type: 'Discrete', resourceType: 'BonusAction', amount: 1 },
            { type: 'Discrete', resourceType: 'SpellSlot', amount: 1 }
        ],
        requirements: [],
        tags: ['Spell', 'Concentration', 'Abjuration', 'Buff', 'Defense'],
        type: 'buff',
        targets: 1,
        target: 'ally with the least HP',
        buff: {
            displayName: 'Shield of Faith',
            duration: 'entire encounter', // 10 mins
            ac: '2',
            concentration: true,
        },
    }),
}

type RealOmit<T, K extends keyof T> = { [P in keyof T as P extends K ? never : P]: T[P] };
type ActionTemplate = RealOmit<FinalAction, 'condition' | 'target' | 'freq' | 'name' | 'id'> & { target?: AllyTarget | EnemyTarget }

// For type safety
function createTemplate(action: ActionTemplate): ActionTemplate {
    return action
}

// Fetches the template if it's a TemplateAction
export function getFinalAction(action: Action): FinalAction {
    if (action.type !== 'template') return action

    const { freq, condition } = action
    const { toHit, saveDC, target, templateName } = action.templateOptions

    const template: ActionTemplate = ActionTemplates[templateName]

    const result = {
        ...template,
        id: action.id,
        name: templateName,
        actionSlot: action.actionSlot ?? template.actionSlot,
        freq,
        condition,
        target: template.target || target as any,
        templateName,
    }

    if (result.type === 'atk') {
        if (toHit !== undefined) result.toHit = toHit

        if (result.riderEffect && (saveDC !== undefined)) result.riderEffect.dc = saveDC
    }

    if (result.type === 'debuff') {
        if (saveDC !== undefined) result.saveDC = saveDC
    }

    if (result.type === 'heal') {
        if (action.templateOptions.amount !== undefined) result.amount = action.templateOptions.amount
    }

    return result
}