// @ts-nocheck - Temporarily excluded: legacy actions need migration to new schema (37 remaining)
import ClassOptions from '../model/classOptions'
import { Action, AtkAction, Creature, DiceFormula, HealAction } from "../model/model"
import { z } from 'zod'
import { getMonster } from './monsters'
import { v4 as uuid } from 'uuid'
import { ActionSlots } from '../model/enums'
import { calculateSpellSlots } from '../model/spellSlots'

// TODO: 
// 1) Add more options to the templates
// 2) Find a way to handle multiclasses

function artificer(level: number, options: z.infer<typeof ClassOptions.artificer>): Creature {
    const INT = scale(level, { 1: 4, 4: 5 })
    const CON = scale(level, { 1: 2, 8: 3, 12: 4, 16: 5 })
    const PB = pb(level)
    const AC = scale(level, { 1: 17, 3: 18, 5: 19, 8: 20, 11: 21, 16: 22 })
    const toHit = INT + PB
    const DC = 8 + PB + INT

    const fireBolt = `${cantrip(level)}d10`
    const arcaneFireArm = scale(level, { 1: '', 6: '+1d8[Arcane Firearm]' })

    // Artificer is a unique caster
    const spellSlots = calculateSpellSlots(level, 'artificer')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Flash of Genius (level 7): add INT mod to save/ability check a number of times per LR
    if (level >= 7) {
        classResources['Flash of Genius'] = scale(level, { 7: INT }) // INT modifier times per Long Rest
    }

    const result: Creature = {
        id: uuid(),
        name: name('Artificer', level),
        AC: AC,
        saveBonus: scale(level, { 1: PB, 20: PB + 6 }),
        hp: hp(level, 8, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        triggers: [
            {
                id: uuid(),
                condition: 'on being attacked',
                cost: ActionSlots.Reaction,
                action: {
                    id: uuid(),
                    type: 'template',
                    freq: { reset: 'lr', uses: scale(level, { 1: 1, 3: 2, 5: 3, 7: 4, 9: 5, 11: 6, 13: 7, 15: 8, 17: 9 }) },
                    condition: 'is under half HP',
                    templateOptions: { templateName: 'Shield' },
                }
            }
        ],
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Firebolt',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
                    requirements: [],
                    tags: ['Attack', 'Damage', 'Fire'],
                    targets: 1,
                    target: 'enemy with least HP',
                    toHit: toHit,
                    dpr: fireBolt + arcaneFireArm,
                    condition: 'default',
                },
            ],
            2: [{
                id: uuid(),
                name: 'Artificer Infusions',
                actionSlot: ActionSlots['Before the Encounter Starts'],
                type: 'buff',
                freq: '1/fight',
                cost: [],
                requirements: [],
                tags: ['Buff', 'Support'],
                condition: 'is available',
                targets: 100,
                target: 'ally with the most HP',
                buff: {
                    duration: 'entire encounter',
                    toHit: scale(level, { 1: 1, 12: 2 }),
                    ac: scale(level, { 1: 1, 12: 2 }),
                }
            }],
            3: [{
                id: uuid(),
                name: 'Shield Turret',
                actionSlot: ActionSlots['Bonus Action'],
                type: 'heal',
                freq: 'at will',
                cost: [{ type: 'Discrete', resourceType: 'BonusAction', amount: 1 }],
                requirements: [],
                tags: ['TempHP', 'Support'],
                condition: 'default',
                targets: 2,
                target: 'ally with the least HP',
                amount: `1d8+${INT}`,
                tempHP: true,
            }],
        }),
    }

    return result
}

function barbarian(level: number, options: z.infer<typeof ClassOptions.barbarian>): Creature {
    const STR = scale(level, { 1: 4, 4: 5, 20: 7 })
    const CON = scale(level, { 1: 2, 8: 3, 12: 4, 16: 5, 20: 7 })
    const DEX = 2
    const PB = pb(level)
    const RAGE_DAMAGE_BONUS = scale(level, { 1: 2, 9: 3, 16: 4 })

    // Build class resources
    const classResources: Record<string, number> = {}

    // Rage: uses per Long Rest
    classResources['Rage'] = scale(level, { 1: 2, 3: 3, 6: 4, 12: 5, 17: 6 })

    // Danger Sense (level 2): Advantage on DEX saves against effects you can see
    if (level >= 2) {
        classResources['Danger Sense'] = 1
    }

    // Feral Instinct (level 7): Advantage on Initiative, can act if surprised
    if (level >= 7) {
        classResources['Feral Instinct'] = 1
    }

    const result: Creature = {
        id: uuid(),
        name: name('Barbarian', level),
        AC: 10 + DEX + CON,
        saveBonus: PB,
        hp: hp(level, 12, CON),
        count: 1,
        mode: 'player',
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        initiativeAdvantage: level >= 7 ? true : undefined, // Feral Instinct (level 7)
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Rage',
                    actionSlot: BONUS_ACTION,
                    type: 'buff',
                    targets: 1,
                    target: 'self',
                    freq: 'at will', // Barbarian can rage at will, but consumes a resource.
                    condition: 'not used yet', // Will be managed by class resource check
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Rage', amount: 1 }],
                    buff: {
                        displayName: 'Rage',
                        duration: 'entire encounter',
                        damageTakenMultiplier: 0.5,
                    },
                },
                {
                    id: uuid(),
                    name: `Greatsword${level >= 5 ? ' x2' : ''}`,
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    targets: scale(level, { 1: 1, 5: 2 }),
                    condition: 'default',
                    freq: 'at will',
                    target: 'enemy with least HP',
                    toHit: `${PB}[PB] + ${STR}[STR]`
                        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                        + (options.gwm ? ` - 5[GWM]` : ''),
                    dpr: `2d6 + ${STR}[STR] + ${RAGE_DAMAGE_BONUS}[RAGE]`
                        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                        + (options.gwm ? ` + 10[GWM]` : ''),
                },
                ...(options.gwm ? [
                    {
                        id: uuid(),
                        name: 'GWM Extra Attack',
                        actionSlot: ActionSlots['When reducing an enemy to 0 HP'],
                        type: 'atk',
                        targets: 1,
                        condition: 'default',
                        freq: 'at will',
                        target: 'enemy with least HP',
                        toHit: `${PB}[PB] + ${STR}[STR]`
                            + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                            + (options.gwm ? ` - 5[GWM]` : ''),
                        dpr: `2d6 + ${STR}[STR] + ${RAGE_DAMAGE_BONUS}[RAGE]`
                            + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                            + (options.gwm ? ` + 10[GWM]` : ''),
                    } as Action
                ] : []),
                {
                    id: uuid(),
                    name: `Greatsword${level >= 5 ? ' x2' : ''}`,
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    targets: scale(level, { 1: 1, 5: 2 }),
                    condition: 'default',
                    freq: 'at will',
                    target: 'enemy with least HP',
                    toHit: `${PB}[PB] + ${STR}[STR]`
                        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                        + (options.gwm ? ` - 5[GWM]` : ''),
                    dpr: `2d6 + ${STR}[STR] + ${RAGE_DAMAGE_BONUS}[RAGE]`
                        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                        + (options.gwm ? ` + 10[GWM]` : ''),
                },
                ...(options.gwm ? [
                    {
                        id: uuid(),
                        name: 'GWM Extra Attack',
                        actionSlot: ActionSlots['When reducing an enemy to 0 HP'],
                        type: 'atk',
                        targets: 1,
                        condition: 'default',
                        freq: 'at will',
                        target: 'enemy with least HP',
                        toHit: `${PB}[PB] + ${STR}[STR]`
                            + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                            + (options.gwm ? ` - 5[GWM]` : ''),
                        dpr: `2d6 + ${STR}[STR] + ${RAGE_DAMAGE_BONUS}[RAGE]`
                            + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                            + (options.gwm ? ` + 10[GWM]` : ''),
                    } as Action
                ] : []),
            ],
            2: [
                {
                    id: uuid(),
                    name: 'Reckless Attack',
                    actionSlot: PASSIVE,
                    type: 'buff',
                    targets: 1,
                    condition: 'default',
                    freq: 'at will',
                    target: 'self',
                    buff: {
                        displayName: 'Reckless Attack',
                        duration: '1 round',
                        condition: 'Attacks and is attacked with Advantage',
                    }
                },
            ],
            11: [
                {
                    id: uuid(),
                    name: 'Relentless Rage',
                    actionSlot: ActionSlots['When Reduced to 0 HP'],
                    type: 'heal',
                    targets: 1,
                    target: 'self',
                    freq: { reset: 'sr', uses: scale(level, { 1: 3, 20: 4 }) },
                    condition: 'is available',
                    amount: '1',
                },
            ],
        })
    }

    return result
}


function bard(level: number, options: z.infer<typeof ClassOptions.bard>): Creature {
    const CON = 2
    const CHA = scale(level, { 1: 4, 4: 5 })
    const DEX = scale(level, { 1: 2, 12: 3, 16: 4 })
    const PB = pb(level)
    const DC = 8 + PB + CHA
    const BARDIC_INSPI_DICE = scale(level, { 1: '1d6', 5: '1d8', 10: '1d10', 15: '1d12' })
    const spellSlots = calculateSpellSlots(level, 'full')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Bardic Inspiration uses (resets on short/long rest at level 5+)
    // Before level 5: CHA modifier times per long rest.
    // At level 5 and beyond: Number of uses equal to your Charisma modifier (minimum of one). You regain all expended uses when you finish a long rest or a short rest.
    // For simplicity, let's use the CHA modifier for uses (capped at 5) for now.
    classResources['Bardic Inspiration'] = CHA // CHA modifier uses per (long/short) rest

    const result: Creature = {
        id: uuid(),
        name: name('Bard', level),
        AC: 13 + DEX,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Vicious Mockery',
                    actionSlot: ACTION,
                    type: 'atk',
                    targets: 1,
                    condition: 'default',
                    freq: 'at will',
                    target: 'enemy with highest DPR',
                    toHit: DC,
                    useSaves: true,
                    dpr: `${cantrip(level)}d4`,
                    riderEffect: {
                        dc: 100,
                        buff: {
                            displayName: 'Vicious Mockery',
                            duration: 'until next attack made',
                            condition: 'Attacks with Disadvantage',
                        },
                    }
                },
                scale<Action>(level, {
                    1: {
                        id: uuid(),
                        type: 'template',
                        condition: 'not used yet',
                        // freq and cost are handled by the template itself (Bane is a spell)
                        templateOptions: { templateName: 'Bane', saveDC: DC, target: 'enemy with highest DPR' },
                    },
                    5: {
                        id: uuid(),
                        type: 'template',
                        // freq and cost are handled by the template itself (Hypnotic Pattern is a spell)
                        condition: 'not used yet',
                        templateOptions: { templateName: 'Hypnotic Pattern', saveDC: DC, target: 'enemy with highest DPR' }
                    },
                }),
                {
                    id: uuid(),
                    name: 'Bardic Inspiration',
                    actionSlot: BONUS_ACTION,
                    type: 'buff',
                    targets: 1,
                    condition: 'default', // Managed by class resource check
                    freq: 'at will', // Can use at will, but consumes a resource
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Bardic Inspiration', amount: 1 }],
                    target: 'ally with the highest DPR',
                    buff: {
                        duration: 'until next attack made',
                        toHit: BARDIC_INSPI_DICE,
                    },
                },
                {
                    id: uuid(),
                    name: (level <= 9) ? 'Cure Wounds' : 'Mass Cure Wounds',
                    actionSlot: ACTION,
                    type: 'heal',
                    targets: (level <= 9) ? 1 : 6,
                    freq: { reset: 'lr', uses: scale(level, { 1: 1, 3: 2, 5: 3, 7: 4, 9: 2, 11: 3, 13: 4, 15: 5, 17: 6 }) },
                    condition: 'ally at 0 HP',
                    target: 'ally with the least HP',
                    amount: `${Math.ceil(level / 3)}d8 + ${CHA}`,
                }
            ],
        }),
    }

    return result
}

function cleric(level: number, options: z.infer<typeof ClassOptions.cleric>): Creature {
    const CON = 2
    const WIS = scale(level, { 1: 4, 4: 5 })
    const PB = pb(level)
    const DC = 8 + PB + WIS
    const toHit = PB + WIS

    const spellSlots = calculateSpellSlots(level, 'full')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Channel Divinity uses (resets on short/long rest)
    classResources['Channel Divinity'] = scale(level, { 1: 1, 6: 2 })

    return {
        id: uuid(),
        name: name('Cleric', level),
        AC: scale(level, { 1: 17, 3: 18, 5: 19, 8: 20 }),
        saveBonus: PB,
        hp: hp(level, 8, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Sacred Flame',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    targets: 1,
                    freq: 'at will',
                    condition: 'default',
                    target: 'enemy with least HP',
                    useSaves: true,
                    toHit: DC,
                    dpr: `${cantrip(level)}d8` + (level >= 8 ? ' + 1d8[Potent Spellcasting]' : ''),
                },
                scale(level, {
                    1: {
                        id: uuid(),
                        type: 'template',
                        // freq and cost are handled by the template itself (Bless is a spell)
                        condition: 'not used yet',
                        templateOptions: { templateName: 'Bless', target: 'ally with the highest DPR' },
                    },
                    5: {
                        id: uuid(),
                        name: 'Spirit Guardians',
                        actionSlot: ActionSlots.Action,
                        type: 'atk',
                        targets: 2,
                        freq: 'at will',
                        condition: 'default',
                        target: 'enemy with least HP',
                        useSaves: true,
                        halfOnSave: true,
                        toHit: DC,
                        dpr: `${(Math.ceil(level / 5) + 2)}d6`,
                    },
                }),
                {
                    id: uuid(),
                    name: scale(level, { 1: 'Cure Wounds', 11: 'Heal' }),
                    actionSlot: ActionSlots.Action,
                    type: 'heal',
                    targets: 1,
                    freq: { reset: 'lr', uses: scale(level, { 1: 1, 3: 2, 5: 3, 7: 4, 9: 5, 11: 1, 13: 2, 15: 3 }) },
                    condition: 'ally at 0 HP',
                    target: 'ally with the least HP',
                    amount: scale(level, { 1: `${Math.ceil(level / 3)}d8 + ${WIS}`, 11: 70 }),
                },
            ],
            3: [
                {
                    id: uuid(),
                    name: 'Spiritual Weapon',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'atk',
                    targets: 1,
                    freq: 'at will',
                    condition: 'default',
                    target: 'enemy with least HP',
                    toHit: toHit,
                    dpr: `${Math.ceil(level / 6)}d8 + ${WIS}`,
                },
            ],
            5: [
                {
                    id: uuid(),
                    name: scale(level, { 1: 'Mass Healing Word', 9: 'Mass Cure Wounds' }),
                    actionSlot: ActionSlots.Action,
                    type: 'heal',
                    targets: 6,
                    freq: { reset: 'lr', uses: scale(level, { 1: 1, 5: 1, 9: 2, 13: 3, 17: 4 }) },
                    condition: 'ally under half HP',
                    target: 'ally with the least HP',
                    amount: scale(level, { 1: `1d4 + ${WIS}`, 9: `3d8 + ${WIS}`, 17: 70 }),
                },
            ],
        }),
    }
}

function druid(level: number, options: z.infer<typeof ClassOptions.druid>): Creature {
    const CON = 2
    const DEX = 2
    const WIS = scale(level, { 1: 4, 4: 5 })
    const PB = pb(level)
    const DC = 8 + PB + WIS
    const toHit = PB + WIS

    const spellSlots = calculateSpellSlots(level, 'full')

    // Build class resources
    const classResources: Record<string, number> = {}

    if (level === 1) {
        return {
            id: uuid(),
            name: name('Druid', level),
            AC: 14 + DEX,
            saveBonus: PB,
            hp: hp(level, 8, CON),
            count: 1,
            mode: 'player',
            spellSlots: spellSlots, // Druids get spells at level 1
            classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
            actions: [
                {
                    id: uuid(),
                    name: 'Shillelagh',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    condition: 'default',
                    freq: 'at will',
                    targets: 1,
                    target: 'enemy with least HP',
                    toHit: toHit,
                    dpr: `1d8 + ${WIS}`,
                },
                {
                    id: uuid(),
                    name: 'Cure Wounds',
                    actionSlot: ActionSlots.Action,
                    type: 'heal',
                    targets: 1,
                    // Cure Wounds is a spell, its freq and cost are handled by spell slots
                    cost: [{ type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_1', amount: 1 }],
                    condition: 'ally at 0 HP',
                    target: 'ally with the least HP',
                    amount: `1d8 + ${WIS}`,
                },
            ],
        }
    }

    // Wild Shape uses (resets on short/long rest, unlimited at level 20)
    if (level >= 2) {
        classResources['Wild Shape'] = scale(level, { 2: 2, 20: 999 }) // 999 for "at will" or unlimited
    }

    const wildshape = scale<Creature>(level, {
        2: getMonster('Dire Wolf')!,
        6: getMonster('Giant Constrictor Snake')!,
        9: getMonster('Giant Scorpion')!,
        10: getMonster('Fire Elemental')!,
    })

    const wildShapeAction: Action = {
        id: uuid(),
        name: `Wild Shape: ${wildshape.name}`,
        actionSlot: ActionSlots['Bonus Action'],
        type: 'heal',
        target: 'self',
        targets: 1,
        condition: 'has no THP',
        freq: 'at will', // Consumes a class resource
        cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Wild Shape', amount: 1 }],
        amount: wildshape.hp,
        tempHP: true,
    }

    return {
        id: uuid(),
        name: name('Druid', level),
        AC: wildshape.AC,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: [
            ...wildshape.actions,
            ...scaleArray<Action>(level, {
                1: [],
                18: [
                    {
                        id: uuid(),
                        type: 'template',
                        condition: 'ally at 0 HP',
                        // Heal is a spell, its freq and cost are handled by the template itself
                        templateOptions: { templateName: 'Heal', target: 'ally with the least HP' },
                    },
                    {
                        id: uuid(),
                        name: 'Guardian of Nature',
                        actionSlot: ActionSlots['Bonus Action'],
                        type: 'buff',
                        target: 'self',
                        targets: 1,
                        condition: 'default',
                        // Guardian of Nature is a spell, its freq and cost are handled by spell slots
                        cost: [{ type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_2', amount: 1 }], // Level 2 spell
                        buff: {
                            displayName: 'Guardian of Nature',
                            duration: 'entire encounter',
                            condition: 'Attacks with Advantage',
                            damage: '2d6',
                        },
                    },
                ],
            }),
            wildShapeAction,
        ].reverse(),
    }
}

function fighter(level: number, options: z.infer<typeof ClassOptions.fighter>): Creature {
    const CON = 2
    const STR = scale(level, { 1: 4, 4: 5, 6: 5, 8: 5 })
    const PB = pb(level)
    const AC = scale(level, { 1: 16, 3: 17, 6: 18 })
    const toHit = `${PB}[PB] + ${STR}[STR]`
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.gwm ? ` - 5[GWM]` : '')

    const attacks = scale(level, { 1: 1, 5: 2, 11: 3, 20: 4 })

    // Main attack action
    const action: Action = {
        id: uuid(),
        name: `Greatsword${attacks > 1 ? ' x' + attacks : ''}`,
        actionSlot: ActionSlots.Action,
        type: 'atk',
        freq: 'at will',
        condition: 'default',
        target: 'enemy with least HP',
        targets: attacks,
        cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
        requirements: [],
        tags: ['Melee', 'Weapon', ...(options.gwm ? ['GWM' as const] : [])],
        toHit: toHit,
        dpr: `2d6 + ${STR}[STR]`
            + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
            + (options.gwm ? ' + 10[GWM]' : '')
    }

    // Build class resources
    const classResources: Record<string, number> = {}

    // Action Surge: 1 use at level 2, 2 uses at level 17+
    if (level >= 2) {
        classResources['Action Surge'] = level >= 17 ? 2 : 1
    }

    // Second Wind: Always 1/fight (short rest)
    if (level >= 1) {
        classResources['Second Wind'] = 1
    }

    // Indomitable: 1 at 9, 2 at 13, 3 at 17
    if (level >= 9) {
        classResources['Indomitable'] = scale(level, { 9: 1, 13: 2, 17: 3 })
    }

    return {
        id: uuid(),
        name: name('Fighter', level),
        AC: AC,
        saveBonus: PB,
        conSaveBonus: PB + CON, // Fighters can be proficient in CON saves
        hp: hp(level, 10, CON),
        count: 1,
        mode: 'player',
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                action,
                {
                    id: uuid(),
                    name: 'Second Wind',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'heal',
                    target: 'self',
                    freq: { reset: 'sr', uses: 1 },
                    condition: 'is under half HP',
                    targets: 1,
                    cost: [
                        { type: 'Discrete', resourceType: 'BonusAction', amount: 1 },
                        { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Second Wind', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Healing'],
                    amount: `1d10 + ${level}`,
                }
            ],
            2: [
                {
                    id: uuid(),
                    name: `Action Surge: ${action.name}`,
                    actionSlot: ActionSlots['Other 1'],
                    freq: { reset: 'sr', uses: 1 },
                    condition: 'is available',
                    type: 'atk',
                    target: 'enemy with least HP',
                    targets: attacks,
                    cost: [
                        { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Action Surge', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Melee', 'Weapon', ...(options.gwm ? ['GWM' as const] : [])],
                    toHit: toHit,
                    dpr: action.dpr,
                }
            ],
            9: [
                {
                    id: uuid(),
                    name: 'Indomitable',
                    actionSlot: ActionSlots['Other 2'],
                    type: 'buff',
                    freq: { reset: 'lr', uses: scale(level, { 9: 1, 13: 2, 17: 3 }) },
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [
                        { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Indomitable', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Defense'],
                    buff: {
                        displayName: 'Indomitable',
                        duration: 'until next attack taken',
                        save: 10, // Represents rerolling a failed save
                    },
                }
            ],
            17: [
                {
                    id: uuid(),
                    name: `Action Surge 2: ${action.name}`,
                    actionSlot: ActionSlots['Other 2'],
                    freq: { reset: 'sr', uses: 1 },
                    condition: 'is available',
                    type: 'atk',
                    target: 'enemy with least HP',
                    targets: attacks,
                    cost: [
                        { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Action Surge', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Melee', 'Weapon', ...(options.gwm ? ['GWM' as const] : [])],
                    toHit: toHit,
                    dpr: action.dpr,
                }
            ]
        })
    }
}

function monk(level: number, options: z.infer<typeof ClassOptions.monk>): Creature {
    const CON = 2
    const DEX = scale(level, { 1: 4, 4: 5 })
    const WIS = scale(level, { 1: 2, 8: 3, 12: 4, 16: 5 })
    const PB = pb(level)
    const toHit = PB + DEX
    const DC = 8 + PB + WIS
    const AC = 10 + DEX + WIS

    const martialArtsDie = scale(level, { 1: '1d4', 5: '1d6', 11: '1d8', 17: '1d10' })

    // Build class resources
    const classResources: Record<string, number> = {}

    // Ki Points (resets on short/long rest)
    if (level >= 2) {
        classResources['Ki Points'] = level
    }

    return {
        id: uuid(),
        name: name('Monk', level),
        AC: AC,
        saveBonus: scale(level, { 1: PB, 14: PB + 3 }),
        hp: hp(level, 8, CON),
        count: 1,
        mode: 'player',
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: scale(level, { 1: 'Quarterstaff', 5: 'Quarterstaff x2' }), // Stunning Strike will be a separate action/cost
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: scale(level, { 1: 1, 5: 2 }),
                    target: 'enemy with highest DPR',
                    toHit: toHit,
                    dpr: `1d10 + ${DEX}`,
                },
                {
                    id: uuid(),
                    name: 'Unarmed Strike (Bonus Action)',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'enemy with highest DPR',
                    toHit: toHit,
                    dpr: `${martialArtsDie} + ${DEX}`,
                },
            ],
            2: [
                {
                    id: uuid(),
                    name: 'Flurry of Blows',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: 2, // Two unarmed strikes
                    target: 'enemy with highest DPR',
                    toHit: toHit,
                    dpr: `${martialArtsDie} + ${DEX}`,
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Ki Points', amount: 1 }],
                },
                {
                    id: uuid(),
                    name: 'Patient Defense (Dodge)',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Ki Points', amount: 1 }],
                    buff: {
                        displayName: 'Patient Defense',
                        duration: '1 round',
                        condition: 'Is attacked with Disadvantage', // Implies enemy has disadvantage
                    },
                },
                {
                    id: uuid(),
                    name: 'Step of the Wind (Dash/Disengage)',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff', // Represents the movement benefit
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Ki Points', amount: 1 }],
                    buff: {
                        displayName: 'Step of the Wind',
                        duration: '1 round',
                        // Details could be added for movement speed or disengage effect
                    },
                },
            ],
            5: [
                {
                    id: uuid(),
                    name: 'Stunning Strike',
                    actionSlot: ActionSlots['Other 1'], // Activated on hit
                    type: 'debuff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'enemy with highest DPR',
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Ki Points', amount: 1 }],
                    saveDC: DC,
                    buff: {
                        displayName: 'Stunning Strike',
                        duration: '1 round',
                        condition: 'Stunned',
                    },
                },
            ],
        }),
    }
}

function paladin(level: number, options: z.infer<typeof ClassOptions.paladin>): Creature {
    const CON = 2
    const STR = scale(level, { 1: 4, 4: 5 })
    const CHA = scale(level, { 1: 2, 8: 3, 12: 5 })
    const PB = pb(level)
    const AC = scale(level, { 1: 17, 3: 18, 5: 19, 8: 20, 11: 21, 16: 22 })
    const toHit = `${PB}[PB] + ${STR}[STR]`
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.gwm ? ` - 5[GWM]` : '')

    const spellSlots = calculateSpellSlots(level, 'half')

    const classResources: Record<string, number> = {
        'Lay on Hands': 5 * level,
    }
    if (level >= 2) {
        // Assuming Dueling fighting style for simplicity, +2 damage
        classResources['Fighting Style (Dueling)'] = 1
    }

    return {
        id: uuid(),
        name: name('Paladin', level),
        AC: AC,
        saveBonus: PB,
        hp: hp(level, 10, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        triggers: [
            {
                id: uuid(),
                condition: 'on hit',
                action: {
                    id: uuid(),
                    name: 'Divine Smite',
                    actionSlot: ActionSlots['Other 1'],
                    type: 'atk',
                    // Divine Smite consumes a spell slot
                    cost: [{ type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_1', amount: 1 }],
                    // The frequency is 'at will' as a reaction to hitting.
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'enemy with least HP',
                    toHit: 100, // Auto-hit, as it's a rider effect
                    dpr: `${scale(level, { 1: 2, 5: 3, 11: 4, 17: 5 })}d8`,
                }
            }
        ],
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: scale(level, { 1: 'Longsword', 5: 'Longsword x2' }),
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: scale(level, { 1: 1, 5: 2 }),
                    target: 'enemy with least HP',
                    toHit: toHit,
                    dpr: `1d8 + ${STR}[STR]`
                        + (level > 1 ? ' + 2[Fighting Style]' : '') // This is still hardcoded, ideally should be a buff
                        + (options.weaponBonus ? ` + ${options.weaponBonus}` : '')
                        + (options.gwm ? ' + 10[GWM]' : '')
                        + (level >= 10 ? ' + 1d8[IDS]' : ''),
                },
                {
                    id: uuid(),
                    name: 'Lay on Hands',
                    actionSlot: ActionSlots.Action,
                    type: 'heal',
                    freq: 'at will',
                    condition: 'any ally injured',
                    targets: 1,
                    target: 'ally with the least HP',
                    amount: 5, // Standard dose
                    cost: [
                        {
                            type: 'Discrete',
                            resourceType: 'ClassResource',
                            resourceVal: 'Lay on Hands',
                            amount: 5,
                        }
                    ]
                },
            ],
            6: [
                {
                    id: uuid(),
                    name: 'Aura of Protection',
                    actionSlot: ActionSlots['Other 2'],
                    type: 'buff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 2,
                    target: 'ally with the least HP',
                    buff: {
                        displayName: 'Aura of Protection',
                        duration: '1 round',
                        save: CHA,
                    },
                },
            ],
        }),
    }
}

function ranger(level: number, options: z.infer<typeof ClassOptions.ranger>): Creature {
    const CON = 2
    const DEX = scale(level, { 1: 4, 4: 5, 8: 5 })
    const WIS = scale(level, { 1: 2, 8: 3, 12: 4, 16: 5 })
    const PB = pb(level)
    const AC = DEX + scale(level, { 1: 12, 5: 13, 11: 14 }) // Studded Leather progression
    const toHit = `${PB}[PB] + ${DEX}[DEX]`
        + (level > 1 ? ' + 2[ARCHERY]' : '')
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.ss ? ` - 5[SS]` : '')
    const DC = 8 + PB + WIS

    // Ranger is a half-caster (spells at level 2+)
    const spellSlots = level >= 2 ? calculateSpellSlots(level, 'half') : undefined

    // Number of attacks
    const attacks = scale(level, { 1: 1, 5: 2 })

    // Build main weapon attack with optional Hunter's Mark damage
    const mainWeaponDpr = `1d6 + ${DEX}[DEX]`
        + (level >= 2 ? ' + 1d6[HM]' : '') // Hunter's Mark at level 2
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.ss ? ' + 10[SS]' : '')

    const classResources: Record<string, number> = {
        'Fighting Style (Archery)': 1,
        'Favored Enemy': 1,
        'Natural Explorer': 1,
    }

    if (level >= 3) {
        classResources['Primeval Awareness'] = 1 // Represents the feature
    }

    return {
        id: uuid(),
        name: name('Ranger', level),
        AC: AC,
        saveBonus: PB,
        dexSaveBonus: PB + DEX, // Dexterity save proficiency
        hp: hp(level, 10, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: scale(level, {
                        1: "Longbow",
                        5: "Longbow x2",
                    }),
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: attacks,
                    target: 'enemy with least HP',
                    cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
                    requirements: [],
                    tags: ['Ranged', 'Weapon', ...(options.ss ? ['Sharpshooter' as const] : [])],
                    toHit: toHit,
                    dpr: mainWeaponDpr,
                },
            ],
            2: [
                {
                    id: uuid(),
                    name: "Hunter's Mark",
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff',
                    freq: { reset: 'lr', uses: scale(level, { 2: 2, 5: 3, 9: 4, 13: 5, 17: 6 }) },
                    condition: 'not used yet',
                    targets: 1,
                    target: 'self',
                    cost: [
                        { type: 'Discrete', resourceType: 'BonusAction', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_1', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Concentration', 'Buff'],
                    buff: {
                        displayName: "Hunter's Mark",
                        duration: 'entire encounter',
                        damage: '1d6', // Already factored into main attack, but shown for clarity
                        concentration: true,
                    },
                },
            ],
            3: [
                // Hunter subclass: Colossus Slayer (extra 1d8 damage 1/turn)
                {
                    id: uuid(),
                    name: 'Colossus Slayer',
                    actionSlot: ActionSlots['Other 1'],
                    type: 'buff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [],
                    requirements: [],
                    tags: ['Damage'],
                    buff: {
                        displayName: 'Colossus Slayer',
                        duration: 'until next attack made',
                        damage: '1d8',
                    },
                },
            ],
            5: [
                // Multiattack is already handled by targets: 2
            ],
            9: [
                // Land's Stride - ignore difficult terrain (not combat relevant)
            ],
            11: [
                // Volley/Whirlwind Attack for Hunter
                {
                    id: uuid(),
                    name: 'Volley',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: { reset: 'sr', uses: 1 },
                    condition: 'enemy count multiple',
                    targets: 4, // AoE attack
                    target: 'enemy with least HP',
                    cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
                    requirements: [],
                    tags: ['Ranged', 'Weapon', 'AoE'],
                    toHit: toHit,
                    dpr: `1d8 + ${DEX}[DEX]` + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : ''),
                },
            ],
            14: [
                // Vanish - can Hide as bonus action + can't be tracked
                {
                    id: uuid(),
                    name: 'Vanish',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff',
                    freq: 'at will',
                    condition: 'is under half HP',
                    targets: 1,
                    target: 'self',
                    cost: [{ type: 'Discrete', resourceType: 'BonusAction', amount: 1 }],
                    requirements: [],
                    tags: ['Defense'],
                    buff: {
                        displayName: 'Hidden',
                        duration: 'until next attack made',
                        condition: 'Attacks with Advantage',
                    },
                },
            ],
            18: [
                // Feral Senses - can sense invisible creatures within 30 ft
                {
                    id: uuid(),
                    name: 'Feral Senses',
                    actionSlot: ActionSlots['Other 2'],
                    type: 'buff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [],
                    requirements: [],
                    tags: ['Buff'],
                    buff: {
                        displayName: 'Feral Senses',
                        duration: 'entire encounter',
                        // Negates disadvantage from not seeing target
                    },
                },
            ],
            20: [
                // Foe Slayer - add WIS to attack or damage once per turn
                {
                    id: uuid(),
                    name: 'Foe Slayer',
                    actionSlot: ActionSlots['Other 1'],
                    type: 'buff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [],
                    requirements: [],
                    tags: ['Damage'],
                    buff: {
                        displayName: 'Foe Slayer',
                        duration: 'until next attack made',
                        damage: WIS,
                    },
                },
            ],
        }),
    }
}

function rogue(level: number, options: z.infer<typeof ClassOptions.rogue>): Creature {
    const CON = 2
    const DEX = scale(level, { 1: 4, 4: 5 })
    const PB = pb(level)
    const AC = DEX + scale(level, { 1: 12, 5: 13, 11: 14 })
    const sneakAttack = `${Math.ceil(level / 2)}d6`
    const toHit = `${PB}[PB] + ${DEX}[DEX]`
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.ss ? ` - 5[SS]` : '')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Stroke of Luck (level 20)
    if (level >= 20) {
        classResources['Stroke of Luck'] = 1 // 1 use per short/long rest
    }

    return {
        id: uuid(),
        name: name('Rogue', level),
        AC: AC,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        count: 1,
        mode: 'player',
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: scale(level, {
                        1: "Hand Crossbow",
                        4: "Hand Crossbow + Crossbow Expert"
                    }),
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: scale(level, { 1: 1, 4: 2 }),
                    target: 'enemy with least HP',
                    toHit: toHit,
                    dpr: `1d6 + ${DEX}[DEX]`
                        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                        + (options.ss ? ' + 10[SS]' : ''),
                },
                {
                    id: uuid(),
                    name: 'Sneak Attack',
                    actionSlot: ActionSlots['Other 1'], // This is a damage rider that applies once per turn
                    type: 'buff', // It's a buff that modifies damage
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    buff: {
                        displayName: 'Sneak Attack',
                        duration: 'until next attack made',
                        damage: sneakAttack
                    }
                },
            ],
            2: [
                {
                    id: uuid(),
                    name: 'Cunning Action: Dash',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff', // Represents the movement benefit
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [{ type: 'Discrete', resourceType: 'BonusAction', amount: 1 }],
                    buff: {
                        displayName: 'Dash',
                        duration: '1 round',
                        // Details could be added for movement speed
                    },
                },
                {
                    id: uuid(),
                    name: 'Cunning Action: Disengage',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff', // Represents the movement benefit
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [{ type: 'Discrete', resourceType: 'BonusAction', amount: 1 }],
                    buff: {
                        displayName: 'Disengage',
                        duration: '1 round',
                        condition: 'Does not provoke Opportunity Attacks',
                    },
                },
                {
                    id: uuid(),
                    name: 'Cunning Action: Hide',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff', // Represents the defensive benefit
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [{ type: 'Discrete', resourceType: 'BonusAction', amount: 1 }],
                    buff: {
                        displayName: 'Hidden',
                        duration: 'until next attack made',
                        condition: 'Attacks with Advantage',
                    },
                },
            ],
            5: [
                {
                    id: uuid(),
                    name: 'Uncanny Dodge',
                    actionSlot: ActionSlots['Other 2'],
                    type: 'buff',
                    target: 'self',
                    targets: 1,
                    freq: 'at will',
                    condition: 'default',
                    buff: {
                        displayName: 'Uncanny Dodge',
                        duration: 'until next attack taken',
                        damageTakenMultiplier: 0.5,
                    },
                }
            ],
        }),
    }
}

function sorcerer(level: number, options: z.infer<typeof ClassOptions.sorcerer>): Creature {
    const CON = 2
    const DEX = 2
    const CHA = scale(level, { 1: 4, 4: 5 })
    const PB = pb(level)
    const AC = 13 + DEX
    const toHit = PB + CHA
    const DC = 8 + PB + CHA

    const spellSlots = calculateSpellSlots(level, 'full')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Sorcery Points (gained at level 2)
    if (level >= 2) {
        classResources['Sorcery Points'] = level // 1 sorcery point per level
    }

    return {
        id: uuid(),
        name: name('Sorcerer', level),
        AC: AC,
        saveBonus: PB,
        hp: hp(level, 6, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Fire Bolt',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'enemy with least HP',
                    toHit: toHit,
                    dpr: `${cantrip(level)}d10`,
                },
            ],
            3: [
                ...scale<Action[]>(level, {
                    1: [],
                    5: [
                        {
                            id: uuid(),
                            name: 'Quickened Fireball',
                            actionSlot: ActionSlots['Bonus Action'],
                            type: 'atk',
                            freq: 'at will', // Consumes Sorcery Points
                            cost: [
                                { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_3', amount: 1 }, // Fireball spell slot
                                { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Sorcery Points', amount: 2 } // Quickened Spell costs 2 Sorcery Points
                            ],
                            condition: 'is available',
                            targets: 2,
                            target: 'enemy with least HP',
                            useSaves: true,
                            halfOnSave: true,
                            toHit: DC,
                            dpr: '8d6',
                        },
                    ]
                }),
                {
                    id: uuid(),
                    name: 'Quickened Mirror Image',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff',
                    freq: 'at will', // Consumes Sorcery Points
                    cost: [
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_2', amount: 1 }, // Mirror Image is a 2nd level spell
                        { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Sorcery Points', amount: 2 } // Quickened Spell costs 2 Sorcery Points
                    ],
                    condition: 'not used yet',
                    targets: 1,
                    target: 'self',
                    buff: {
                        displayName: 'Mirror Image',
                        duration: 'entire encounter',
                        damageTakenMultiplier: 0.25,
                    },
                },
            ],
        }),
    }
}
function warlock(level: number, options: z.infer<typeof ClassOptions.warlock>): Creature {
    const CON = 2
    const DEX = 2
    const CHA = scale(level, { 1: 4, 4: 5 })
    const PB = pb(level)
    const AC = 13 + DEX
    const DC = 8 + PB + CHA
    const toHit = PB + CHA

    const spellSlots = calculateSpellSlots(level, 'pact')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Mystic Arcanum (Level 11+): One use of each spell level 6-9
    if (level >= 11) {
        classResources['Mystic Arcanum (6th Level)'] = 1
    }
    if (level >= 13) {
        classResources['Mystic Arcanum (7th Level)'] = 1
    }
    if (level >= 15) {
        classResources['Mystic Arcanum (8th Level)'] = 1
    }
    if (level >= 17) {
        classResources['Mystic Arcanum (9th Level)'] = 1
    }

    return {
        id: uuid(),
        name: name('Warlock', level),
        AC: AC,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Eldritch Blast + Hex',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: cantrip(level),
                    target: 'enemy with least HP',
                    toHit: toHit,
                    dpr: `1d10 + 1d6[HEX]`
                        + (level > 1 ? ` + ${CHA}[AB]` : ''),
                },
                {
                    id: uuid(),
                    type: 'template',
                    // freq and cost are handled by the template itself (Armor of Agathys is a spell)
                    condition: 'is available',
                    templateOptions: {
                        templateName: 'Armor of Agathys',
                        amount: (5 * level).toString()
                    },
                },
            ],
            5: [
                {
                    id: uuid(),
                    type: 'template',
                    // freq and cost are handled by the template itself (Hypnotic Pattern is a spell)
                    condition: 'not used yet',
                    templateOptions: { templateName: 'Hypnotic Pattern', saveDC: DC, target: 'enemy with highest DPR' },
                },
            ],
        }),
    }
}

function wizard(level: number, options: z.infer<typeof ClassOptions.wizard>): Creature {
    const CON = 2
    const DEX = 2
    const INT = scale(level, { 1: 4, 4: 5, 8: 5 })
    const PB = pb(level)
    const AC = 13 + DEX // Base with Mage Armor
    const toHit = PB + INT
    const DC = 8 + PB + INT

    // Wizard is a full caster
    const spellSlots = calculateSpellSlots(level, 'full')

    // Class resources
    const classResources: Record<string, number> = {}

    // Arcane Recovery: recover spell slots on short rest (levels = 1/2 wizard level, rounded up)
    if (level >= 1) {
        classResources['Arcane Recovery'] = 1
    }

    // Overchannel uses (level 14+): 1/LR for free, then take damage
    if (level >= 14) {
        classResources['Overchannel'] = 1
    }

    return {
        id: uuid(),
        name: name('Wizard', level),
        AC: AC,
        saveBonus: PB,
        intSaveBonus: PB + INT, // Intelligence save proficiency
        wisSaveBonus: PB + scale(level, { 1: 1, 8: 2, 12: 3 }), // Wisdom save proficiency
        hp: hp(level, 6, CON),
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        triggers: [
            {
                id: uuid(),
                condition: 'on being attacked',
                cost: ActionSlots.Reaction,
                action: {
                    id: uuid(),
                    type: 'template',
                    freq: { reset: 'lr', uses: Math.ceil(level / 2) },
                    condition: scale(level, { 1: 'is under half HP', 8: 'default' }),
                    templateOptions: { templateName: 'Shield' },
                }
            }
        ],
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Fire Bolt',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'enemy with least HP',
                    cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
                    requirements: [],
                    tags: ['Spell', 'Evocation', 'Fire', 'Attack'],
                    toHit: toHit,
                    // Evocation Wizard gets Empowered Evocation at level 10 (+INT to one evocation damage roll)
                    dpr: `${cantrip(level)}d10` + (level >= 10 ? ` + ${INT}[Empowered]` : ''),
                },
                {
                    id: uuid(),
                    type: 'template',
                    freq: { reset: 'lr', uses: 1 },
                    condition: 'not used yet',
                    cost: [{ type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_1', amount: 1 }],
                    requirements: [],
                    tags: ['Spell', 'Abjuration', 'Buff'],
                    templateOptions: { templateName: 'Mage Armour' },
                },
                {
                    id: uuid(),
                    name: 'Arcane Recovery',
                    actionSlot: ActionSlots['Other 2'],
                    type: 'buff',
                    freq: { reset: 'lr', uses: 1 },
                    condition: 'is available',
                    targets: 1,
                    target: 'self',
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Arcane Recovery', amount: 1 }],
                    requirements: [],
                    tags: ['Utility'],
                    buff: {
                        displayName: 'Arcane Recovery',
                        duration: 'instant',
                        description: `Recovers spell slots equal to ${Math.ceil(level / 2)} spell levels.`,
                    },
                },
            ],
            2: [
                // Level 2: Arcane Tradition (School of Evocation)
                // Sculpt Spells - allies auto-succeed on evocation spell saves
            ],
            3: [
                {
                    id: uuid(),
                    name: 'Magic Missile',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: { reset: 'lr', uses: scale(level, { 3: 2, 5: 3, 7: 4, 9: 5 }) },
                    condition: 'default',
                    targets: 3,
                    target: 'enemy with least HP',
                    cost: [
                        { type: 'Discrete', resourceType: 'Action', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_1', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Evocation', 'Force', 'Attack'],
                    toHit: 100, // Auto-hit
                    dpr: '1d4+1',
                },
            ],
            5: [
                {
                    id: uuid(),
                    name: 'Fireball',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: { reset: 'lr', uses: scale(level, { 5: 2, 9: 3, 13: 4, 17: 5 }) },
                    condition: 'is available',
                    targets: scale(level, { 5: 2, 11: 3 }), // Sculpt Spells protects allies
                    target: 'enemy with least HP',
                    cost: [
                        { type: 'Discrete', resourceType: 'Action', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_3', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Evocation', 'Fire', 'AoE', 'Attack'],
                    useSaves: true,
                    halfOnSave: true,
                    toHit: DC,
                    dpr: '8d6' + (level >= 10 ? ` + ${INT}[Empowered]` : ''),
                },
                {
                    id: uuid(),
                    type: 'template',
                    freq: { reset: 'lr', uses: scale(level, { 5: 1, 9: 2, 13: 3 }) },
                    condition: 'not used yet',
                    cost: [
                        { type: 'Discrete', resourceType: 'Action', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_3', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Enchantment', 'Control', 'AoE', 'Concentration'],
                    templateOptions: { templateName: 'Hypnotic Pattern', saveDC: DC, target: 'enemy with highest DPR' },
                },
            ],
            7: [
                {
                    id: uuid(),
                    name: 'Greater Invisibility',
                    actionSlot: ActionSlots.Action,
                    type: 'buff',
                    freq: { reset: 'lr', uses: scale(level, { 7: 1, 11: 2, 15: 3 }) },
                    condition: 'is under half HP',
                    targets: 1,
                    target: 'self',
                    cost: [
                        { type: 'Discrete', resourceType: 'Action', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_4', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Illusion', 'Concentration', 'Buff'],
                    buff: {
                        displayName: 'Greater Invisibility',
                        duration: 'entire encounter',
                        condition: 'Invisible',
                        concentration: true,
                    },
                },
            ],
            9: [
                {
                    id: uuid(),
                    name: 'Cone of Cold',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: { reset: 'lr', uses: scale(level, { 9: 1, 13: 2, 17: 3 }) },
                    condition: 'enemy count multiple',
                    targets: 3,
                    target: 'enemy with least HP',
                    cost: [
                        { type: 'Discrete', resourceType: 'Action', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_5', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Evocation', 'Cold', 'AoE', 'Attack'],
                    useSaves: true,
                    halfOnSave: true,
                    toHit: DC,
                    dpr: '8d8' + (level >= 10 ? ` + ${INT}[Empowered]` : ''),
                },
            ],
            10: [
                // Empowered Evocation - +INT to evocation spell damage (already applied above)
            ],
            11: [
                {
                    id: uuid(),
                    name: 'Disintegrate',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: { reset: 'lr', uses: scale(level, { 11: 1, 15: 2, 19: 3 }) },
                    condition: 'is available',
                    targets: 1,
                    target: 'enemy with least HP',
                    cost: [
                        { type: 'Discrete', resourceType: 'Action', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_6', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Transmutation', 'Force', 'Attack'],
                    useSaves: true,
                    halfOnSave: false, // All or nothing
                    toHit: DC,
                    dpr: '10d6+40',
                },
            ],
            14: [
                // Overchannel - maximize evocation spell damage (1-5th level)
                {
                    id: uuid(),
                    name: 'Overchannel (Fireball)',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: { reset: 'lr', uses: 1 },
                    condition: 'is available',
                    targets: 3,
                    target: 'enemy with least HP',
                    cost: [
                        { type: 'Discrete', resourceType: 'Action', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_3', amount: 1 },
                        { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Overchannel', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Evocation', 'Fire', 'AoE', 'Attack'],
                    useSaves: true,
                    halfOnSave: true,
                    toHit: DC,
                    dpr: `48 + ${INT}`, // Max damage: 8*6=48
                },
            ],
            17: [
                {
                    id: uuid(),
                    type: 'template',
                    freq: '1/day',
                    condition: 'is available',
                    cost: [
                        { type: 'Discrete', resourceType: 'Action', amount: 1 },
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_9', amount: 1 }
                    ],
                    requirements: [],
                    tags: ['Spell', 'Evocation', 'Fire', 'AoE', 'Attack'],
                    templateOptions: { templateName: 'Meteor Swarm', toHit: DC, target: 'enemy with least HP' },
                },
            ],
            18: [
                // Spell Mastery - can cast a 1st and 2nd level spell at will
                {
                    id: uuid(),
                    name: 'Spell Mastery: Shield (at will)',
                    actionSlot: ActionSlots.Reaction,
                    type: 'buff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    cost: [{ type: 'Discrete', resourceType: 'Reaction', amount: 1 }],
                    requirements: [],
                    tags: ['Spell', 'Abjuration', 'Defense'],
                    buff: {
                        displayName: 'Shield',
                        duration: '1 round',
                        ac: '5',
                    },
                },
            ],
            20: [
                // Signature Spells - two 3rd-level spells always prepared, cast once each without slot
            ],
        }),
    }
}

export const PlayerTemplates = {
    artificer,
    barbarian,
    bard,
    cleric,
    druid,
    fighter,
    monk,
    paladin,
    ranger,
    rogue,
    sorcerer,
    warlock,
    wizard,
} as const


/*  UTILS   */

const ACTION = 0
const BONUS_ACTION = 1
const REACTION = 4
const PASSIVE = 5
const RIDER_EFFECT = 6

function scale<T>(currentLevel: number, levelScale: { [minLevel: number]: T }): T {
    const keys = Object.keys(levelScale).map(Number);
    const applicableLevels = keys.filter(scaleLevel => (scaleLevel <= currentLevel));

    if (applicableLevels.length === 0) {
        // If currentLevel is below all defined minLevels, use the smallest minLevel.
        // Assumes levelScale is not empty itself.
        const minLevel = Math.min(...keys);
        return levelScale[minLevel];
    }

    const effectiveLevel = applicableLevels.reduce((a, b) => Math.max(a, b));
    return levelScale[effectiveLevel];
}

function scaleArray<T>(currentLevel: number, minLevelScale: { [minLevel: number]: T[] }): T[] {
    return [
        ...Object.keys(minLevelScale)
            .map(Number)
            .filter(level => (level <= currentLevel))
            .flatMap(level => minLevelScale[level])
            .reverse(),
    ]
}

function multiattack(level: number, expr: DiceFormula) {
    if (level < 5) return expr

    return `(${expr})*2`
}

const ADVANTAGE = 4.5
const DISADVANTAGE = -4.5
const d4 = 2.5
const d6 = 3.5
const d8 = 4.5
const d10 = 5.5
const d12 = 6.5
const cantrip = (level: number) => scale(level, { 1: 1, 5: 2, 11: 3, 17: 4 })

function hp(level: number, dieSize: number, con: number) {
    return (level) * (con + dieSize / 2) + (dieSize / 2)
}

function pb(level: number) {
    return scale(level, { 1: 2, 5: 3, 9: 4, 13: 5, 17: 6 })
}

function name(className: string, level: number) {
    return `Lv${level} ${className}`
}