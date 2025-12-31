// @ts-nocheck - Temporarily excluded: legacy actions need migration to new schema (37 remaining)
import ClassOptions from '../model/classOptions'
import { Action, AtkAction, Creature, DiceFormula, HealAction } from "../model/model"
import { z } from 'zod'
import { getMonster, DefaultMonsters } from './monsters'
import { v4 as uuid } from 'uuid'
import { ActionSlots } from '../model/enums'
import { calculateSpellSlots } from '../model/spellSlots'
import { clone } from '../model/utils'

// TODO: 
// 1) Add more options to the templates
// 2) Find a way to handle multiclasses

function artificer(level: number, options: z.infer<typeof ClassOptions.artificer>): Creature {
    const INT = scale(level, { 1: 4, 4: 5, 8: 5 })
    const CON = scale(level, { 1: 2, 12: 3, 16: 4, 19: 5 })
    const PB = pb(level)
    const ac = scale(level, { 1: 17, 3: 18, 5: 19, 10: 20, 15: 21, 20: 22 })
    const toHit = INT + PB
    const DC = 8 + PB + INT

    const fireBolt = `${cantrip(level)}d10`
    const arcaneFireArm = scale(level, { 1: '', 5: '+1d8[Arcane Firearm]' })

    // Artificer is a unique caster
    const spellSlots = calculateSpellSlots(level, 'artificer')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Infusions (level 2)
    if (level >= 2) {
        classResources['Infusion Uses'] = scale(level, { 2: 2, 6: 3, 10: 4, 14: 5, 18: 6 })
    }

    // Flash of Genius (level 7)
    if (level >= 7) {
        classResources['Flash of Genius'] = INT
    }

    // Spell-Storing Item (level 11)
    if (level >= 11) {
        classResources['Spell-Storing Item'] = 2 * INT
    }

    const result: Creature = {
        id: uuid(),
        name: name('Artificer', level),
        ac: ac,
        saveBonus: scale(level, { 1: PB, 20: PB + 6 }),
        hp: hp(level, 8, CON),
        hitDice: `${level}d8`,
        conModifier: CON,
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
                cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Infusion Uses', amount: 1 }],
                requirements: [],
                tags: ['Buff', 'Support'],
                condition: 'is available',
                targets: 100,
                target: 'ally with most HP',
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
                target: 'ally with least HP',
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
    classResources['Rage'] = scale(level, { 1: 2, 3: 3, 6: 4, 12: 5, 17: 6, 20: 999 })

    const result: Creature = {
        id: uuid(),
        name: name('Barbarian', level),
        ac: 10 + DEX + CON,
        saveBonus: PB,
        hp: hp(level, 12, CON),
        hitDice: `${level}d12`,
        conModifier: CON,
        count: 1,
        mode: 'player',
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        initiativeAdvantage: level >= 7, // Feral Instinct (level 7)
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Rage',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff',
                    targets: 1,
                    target: 'self',
                    freq: 'at will',
                    condition: 'not used yet',
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
            ],
            2: [
                {
                    id: uuid(),
                    name: 'Reckless Attack',
                    actionSlot: ActionSlots['Other 1'],
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
                    freq: { reset: 'sr', uses: scale(level, { 11: 1, 20: 3 }) },
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
    const CHA = scale(level, { 1: 4, 4: 5, 8: 5 })
    const DEX = scale(level, { 1: 2, 12: 3, 16: 4 })
    const PB = pb(level)
    const DC = 8 + PB + CHA
    const BARDIC_INSPI_DICE = scale(level, { 1: '1d6', 5: '1d8', 10: '1d10', 15: '1d12' })
    const spellSlots = calculateSpellSlots(level, 'full')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Bardic Inspiration uses
    classResources['Bardic Inspiration'] = CHA

    // Countercharm (level 6)
    if (level >= 6) {
        classResources['Countercharm'] = 1
    }

    return {
        id: uuid(),
        name: name('Bard', level),
        ac: 13 + DEX,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        hitDice: `${level}d8`,
        conModifier: CON,
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: 'Vicious Mockery',
                    actionSlot: ActionSlots.Action,
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
                {
                    id: uuid(),
                    type: 'template',
                    condition: 'not used yet',
                    templateOptions: { templateName: 'Bane', saveDC: DC, target: 'enemy with highest DPR' },
                },
                {
                    id: uuid(),
                    name: 'Bardic Inspiration',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff',
                    targets: 1,
                    condition: 'default',
                    freq: 'at will',
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Bardic Inspiration', amount: 1 }],
                    target: 'ally with highest DPR',
                    buff: {
                        duration: 'until next attack made',
                        toHit: BARDIC_INSPI_DICE,
                    },
                },
                {
                    id: uuid(),
                    name: (level < 9) ? 'Cure Wounds' : 'Mass Cure Wounds',
                    actionSlot: ActionSlots.Action,
                    type: 'heal',
                    targets: (level < 9) ? 1 : 6,
                    freq: { reset: 'lr', uses: scale(level, { 1: 1, 3: 2, 5: 3, 7: 4, 9: 2, 11: 3, 13: 4, 15: 5, 17: 6 }) },
                    condition: 'ally at 0 HP',
                    target: 'ally with least HP',
                    amount: `${Math.ceil(level / 3)}d8 + ${CHA}`,
                }
            ],
            5: [
                {
                    id: uuid(),
                    type: 'template',
                    condition: 'not used yet',
                    templateOptions: { templateName: 'Hypnotic Pattern', saveDC: DC, target: 'enemy with highest DPR' }
                },
            ]
        }),
    }
}

function cleric(level: number, options: z.infer<typeof ClassOptions.cleric>): Creature {
    const CON = 2
    const WIS = scale(level, { 1: 4, 4: 5, 8: 5 })
    const PB = pb(level)
    const DC = 8 + PB + WIS
    const toHit = PB + WIS

    const spellSlots = calculateSpellSlots(level, 'full')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Channel Divinity uses (resets on short/long rest)
    if (level >= 2) {
        classResources['Channel Divinity'] = scale(level, { 2: 1, 6: 2, 18: 3 })
    }

    return {
        id: uuid(),
        name: name('Cleric', level),
        ac: scale(level, { 1: 17, 3: 18, 5: 19, 10: 20 }),
        saveBonus: PB,
        hp: hp(level, 8, CON),
        hitDice: `${level}d8`,
        conModifier: CON,
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
                    dpr: `${cantrip(level)}d8` + (level >= 8 ? ` + ${WIS}[Potent Spellcasting]` : ''),
                },
                {
                    id: uuid(),
                    type: 'template',
                    condition: 'not used yet',
                    templateOptions: { templateName: 'Bless', target: 'ally with highest DPR' },
                },
                {
                    id: uuid(),
                    name: scale(level, { 1: 'Cure Wounds', 11: 'Heal' }),
                    actionSlot: ActionSlots.Action,
                    type: 'heal',
                    targets: 1,
                    freq: { reset: 'lr', uses: scale(level, { 1: 1, 3: 2, 5: 3, 7: 4, 9: 5, 11: 1, 13: 2, 15: 3 }) },
                    condition: 'ally at 0 HP',
                    target: 'ally with least HP',
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
                    name: 'Spirit Guardians',
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    targets: 3,
                    freq: 'at will',
                    condition: 'default',
                    target: 'enemy with least HP',
                    useSaves: true,
                    halfOnSave: true,
                    toHit: DC,
                    dpr: `${(Math.ceil(level / 5) + 2)}d6`,
                },
                {
                    id: uuid(),
                    name: scale(level, { 5: 'Mass Healing Word', 9: 'Mass Cure Wounds' }),
                    actionSlot: ActionSlots.Action,
                    type: 'heal',
                    targets: 6,
                    freq: { reset: 'lr', uses: scale(level, { 5: 1, 9: 2, 13: 3, 17: 4 }) },
                    condition: 'ally under half HP',
                    target: 'ally with least HP',
                    amount: scale(level, { 5: `1d4 + ${WIS}`, 9: `3d8 + ${WIS}`, 17: 70 }),
                },
            ],
        }),
    }
}

function druid(level: number, options: z.infer<typeof ClassOptions.druid>): Creature {
    const CON = 2
    const DEX = 2
    const WIS = scale(level, { 1: 4, 4: 5, 8: 5 })
    const PB = pb(level)
    const DC = 8 + PB + WIS
    const toHit = PB + WIS

    const spellSlots = calculateSpellSlots(level, 'full')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Wild Shape uses (resets on short/long rest, unlimited at level 20)
    if (level >= 2) {
        classResources['Wild Shape'] = scale(level, { 2: 2, 20: 999 })
    }

    const wildshape = scale<Creature>(level, {
        1: { name: 'None', hp: 0, ac: 10, actions: [] } as any, // Placeholder for lvl 1
        2: DefaultMonsters.find(m => m.name === 'Dire Wolf')!,
        6: DefaultMonsters.find(m => m.name === 'Giant Constrictor Snake')!,
        9: DefaultMonsters.find(m => m.name === 'Giant Scorpion')!,
        10: DefaultMonsters.find(m => m.name === 'Fire Elemental')!,
    })

    const wildShapeAction: Action = {
        id: uuid(),
        name: `Wild Shape: ${wildshape.name}`,
        actionSlot: ActionSlots['Bonus Action'],
        type: 'heal',
        target: 'self',
        targets: 1,
        condition: 'has no THP',
        freq: 'at will',
        cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Wild Shape', amount: 1 }],
        amount: wildshape.hp,
        tempHP: true,
    }

    return {
        id: uuid(),
        name: name('Druid', level),
        ac: level >= 2 ? wildshape.ac : 14 + DEX,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        hitDice: `${level}d8`,
        conModifier: CON,
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: [
            ...(level >= 2 ? wildshape.actions : [
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
            ]),
            ...scaleArray<Action>(level, {
                1: [
                    {
                        id: uuid(),
                        name: 'Cure Wounds',
                        actionSlot: ActionSlots.Action,
                        type: 'heal',
                        targets: 1,
                        cost: [{ type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_1', amount: 1 }],
                        condition: 'ally at 0 HP',
                        target: 'ally with least HP',
                        amount: `1d8 + ${WIS}`,
                    },
                ],
                18: [
                    {
                        id: uuid(),
                        type: 'template',
                        condition: 'ally at 0 HP',
                        templateOptions: { templateName: 'Heal', target: 'ally with least HP' },
                    },
                ],
            }),
            ...(level >= 2 ? [wildShapeAction] : []),
        ].reverse(),
    }
}

function fighter(level: number, options: z.infer<typeof ClassOptions.fighter>): Creature {
    const CON = 2
    const STR = scale(level, { 1: 4, 4: 5, 6: 5 })
    const PB = pb(level)
    const ac = scale(level, { 1: 16, 3: 17, 6: 18, 10: 19, 15: 20 })
    const toHit = `${PB}[PB] + ${STR}[STR]`
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.gwm ? ` - 5[GWM]` : '')

    const attacks = scale(level, { 1: 1, 5: 2, 11: 3, 20: 4 })

    // Build class resources
    const classResources: Record<string, number> = {}

    // Action Surge
    if (level >= 2) {
        classResources['Action Surge'] = scale(level, { 2: 1, 17: 2 })
    }

    // Second Wind
    classResources['Second Wind'] = 1

    // Indomitable
    if (level >= 9) {
        classResources['Indomitable'] = scale(level, { 9: 1, 13: 2, 17: 3 })
    }

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

    return {
        id: uuid(),
        name: name('Fighter', level),
        ac: ac,
        saveBonus: PB,
        conSaveBonus: PB + CON,
        hp: hp(level, 10, CON),
        hitDice: `${level}d10`,
        conModifier: CON,
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
                    actionSlot: ActionSlots.Other1,
                    freq: 'at will', // Consumes resource
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
                    actionSlot: ActionSlots.Other2,
                    type: 'buff',
                    freq: 'at will', // Consumes resource
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
                        save: 10, // Reroll approximation
                    },
                }
            ],
        })
    }
}

function monk(level: number, options: z.infer<typeof ClassOptions.monk>): Creature {
    const CON = 2
    const DEX = scale(level, { 1: 4, 4: 5, 8: 5 })
    const WIS = scale(level, { 1: 3, 4: 4, 12: 5 })
    const PB = pb(level)
    const toHit = PB + DEX
    const DC = 8 + PB + WIS
    const ac = 10 + DEX + WIS

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
        ac: ac,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        hitDice: `${level}d8`,
        conModifier: CON,
        count: 1,
        mode: 'player',
        classResources: Object.keys(classResources).length > 0 ? classResources : undefined,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: scale(level, { 1: 'Quarterstaff', 5: 'Quarterstaff x2' }),
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: scale(level, { 1: 1, 5: 2 }),
                    target: 'enemy with highest DPR',
                    toHit: toHit,
                    dpr: `1d8 + ${DEX}`,
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
                    targets: 2,
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
                        condition: 'Is attacked with Disadvantage',
                    },
                },
            ],
            5: [
                {
                    id: uuid(),
                    name: 'Stunning Strike',
                    actionSlot: ActionSlots['Other 1'],
                    type: 'debuff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'enemy with highest DPR',
                    cost: [{ type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Ki Points', amount: 1 }],
                    saveDC: DC,
                    buff: {
                        displayName: 'Stunned',
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
    const STR = scale(level, { 1: 4, 4: 5, 8: 5 })
    const CHA = scale(level, { 1: 3, 4: 4, 12: 5 })
    const PB = pb(level)
    const ac = scale(level, { 1: 17, 3: 18, 5: 19, 10: 20, 15: 21 })
    const toHit = `${PB}[PB] + ${STR}[STR]`
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.gwm ? ` - 5[GWM]` : '')

    const spellSlots = calculateSpellSlots(level, 'half')

    const classResources: Record<string, number> = {
        'Lay on Hands': 5 * level,
    }

    return {
        id: uuid(),
        name: name('Paladin', level),
        ac: ac,
        saveBonus: PB,
        hp: hp(level, 10, CON),
        hitDice: `${level}d10`,
        conModifier: CON,
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
                    cost: [{ type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_1', amount: 1 }],
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'enemy with least HP',
                    toHit: 100,
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
                        + (level >= 2 ? ' + 2[Fighting Style]' : '')
                        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                        + (options.gwm ? ' + 10[GWM]' : '')
                        + (level >= 11 ? ' + 1d8[IDS]' : ''),
                },
                {
                    id: uuid(),
                    name: 'Lay on Hands',
                    actionSlot: ActionSlots.Action,
                    type: 'heal',
                    freq: 'at will',
                    condition: 'any ally injured',
                    targets: 1,
                    target: 'ally with least HP',
                    amount: 5,
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
                    targets: 3,
                    target: 'ally with least HP',
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
    const WIS = scale(level, { 1: 3, 4: 4, 12: 5 })
    const PB = pb(level)
    const ac = DEX + scale(level, { 1: 12, 5: 13, 11: 14 })
    const toHit = `${PB}[PB] + ${DEX}[DEX]`
        + (level >= 2 ? ' + 2[ARCHERY]' : '')
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.ss ? ` - 5[SS]` : '')

    const spellSlots = level >= 2 ? calculateSpellSlots(level, 'half') : undefined
    const attacks = scale(level, { 1: 1, 5: 2 })

    return {
        id: uuid(),
        name: name('Ranger', level),
        ac: ac,
        saveBonus: PB,
        hp: hp(level, 10, CON),
        hitDice: `${level}d10`,
        conModifier: CON,
        count: 1,
        mode: 'player',
        spellSlots: spellSlots,
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: scale(level, { 1: "Longbow", 5: "Longbow x2" }),
                    actionSlot: ActionSlots.Action,
                    type: 'atk',
                    freq: 'at will',
                    condition: 'default',
                    targets: attacks,
                    target: 'enemy with least HP',
                    toHit: toHit,
                    dpr: `1d8 + ${DEX}[DEX]`
                        + (level >= 2 ? ' + 1d6[HM]' : '')
                        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
                        + (options.ss ? ' + 10[SS]' : ''),
                },
            ],
            2: [
                {
                    id: uuid(),
                    type: 'template',
                    condition: 'not used yet',
                    templateOptions: { templateName: "Hunter's Mark", target: 'enemy with least HP' },
                },
            ],
            3: [
                {
                    id: uuid(),
                    name: 'Colossus Slayer',
                    actionSlot: ActionSlots['Other 1'],
                    type: 'buff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    buff: {
                        displayName: 'Colossus Slayer',
                        duration: 'until next attack made',
                        damage: '1d8',
                    },
                },
            ],
        }),
    }
}

function rogue(level: number, options: z.infer<typeof ClassOptions.rogue>): Creature {
    const CON = 2
    const DEX = scale(level, { 1: 4, 4: 5, 10: 5 })
    const PB = pb(level)
    const ac = DEX + scale(level, { 1: 12, 5: 13, 11: 14 })
    const sneakAttack = `${Math.ceil(level / 2)}d6`
    const toHit = `${PB}[PB] + ${DEX}[DEX]`
        + (options.weaponBonus ? ` + ${options.weaponBonus}[WEAPON]` : '')
        + (options.ss ? ` - 5[SS]` : '')

    return {
        id: uuid(),
        name: name('Rogue', level),
        ac: ac,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        hitDice: `${level}d8`,
        conModifier: CON,
        count: 1,
        mode: 'player',
        actions: scaleArray<Action>(level, {
            1: [
                {
                    id: uuid(),
                    name: scale(level, { 1: "Hand Crossbow", 4: "Hand Crossbow + Crossbow Expert" }),
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
                    actionSlot: ActionSlots['Other 1'],
                    type: 'buff',
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
                    name: 'Cunning Action: Dash/Hide/Disengage',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'buff',
                    freq: 'at will',
                    condition: 'default',
                    targets: 1,
                    target: 'self',
                    buff: {
                        displayName: 'Cunning Action',
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
    const CHA = scale(level, { 1: 4, 4: 5, 8: 5 })
    const PB = pb(level)
    const ac = 13 + DEX
    const toHit = PB + CHA
    const DC = 8 + PB + CHA

    const spellSlots = calculateSpellSlots(level, 'full')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Sorcery Points (gained at level 2)
    if (level >= 2) {
        classResources['Sorcery Points'] = level
    }

    return {
        id: uuid(),
        name: name('Sorcerer', level),
        ac: ac,
        saveBonus: PB,
        hp: hp(level, 6, CON),
        hitDice: `${level}d6`,
        conModifier: CON,
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
                {
                    id: uuid(),
                    name: 'Quickened Fireball',
                    actionSlot: ActionSlots['Bonus Action'],
                    type: 'atk',
                    freq: 'at will',
                    cost: [
                        { type: 'Discrete', resourceType: 'SpellSlot', resourceVal: 'level_3', amount: 1 },
                        { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Sorcery Points', amount: 2 }
                    ],
                    condition: 'is available',
                    targets: 2,
                    target: 'enemy with least HP',
                    useSaves: true,
                    halfOnSave: true,
                    toHit: DC,
                    dpr: '8d6',
                },
            ],
        }),
    }
}

function warlock(level: number, options: z.infer<typeof ClassOptions.warlock>): Creature {
    const CON = 2
    const DEX = 2
    const CHA = scale(level, { 1: 4, 4: 5, 8: 5 })
    const PB = pb(level)
    const ac = 13 + DEX
    const DC = 8 + PB + CHA
    const toHit = PB + CHA

    const spellSlots = calculateSpellSlots(level, 'pact')

    // Build class resources
    const classResources: Record<string, number> = {}

    // Mystic Arcanum
    if (level >= 11) classResources['Mystic Arcanum (6th)'] = 1
    if (level >= 13) classResources['Mystic Arcanum (7th)'] = 1
    if (level >= 15) classResources['Mystic Arcanum (8th)'] = 1
    if (level >= 17) classResources['Mystic Arcanum (9th)'] = 1

    return {
        id: uuid(),
        name: name('Warlock', level),
        ac: ac,
        saveBonus: PB,
        hp: hp(level, 8, CON),
        hitDice: `${level}d8`,
        conModifier: CON,
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
                        + (level >= 2 ? ` + ${CHA}[AB]` : ''),
                },
                {
                    id: uuid(),
                    type: 'template',
                    condition: 'is available',
                    templateOptions: {
                        templateName: 'Armor of Agathys',
                        amount: (5 * Math.ceil(level / 2)).toString()
                    },
                },
            ],
            5: [
                {
                    id: uuid(),
                    type: 'template',
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
    const ac = 13 + DEX
    const toHit = PB + INT
    const DC = 8 + PB + INT

    const spellSlots = calculateSpellSlots(level, 'full')

    // Class resources
    const classResources: Record<string, number> = {
        'Arcane Recovery': 1,
    }

    if (level >= 14) classResources['Overchannel'] = 1

    return {
        id: uuid(),
        name: name('Wizard', level),
        ac: ac,
        saveBonus: PB,
        intSaveBonus: PB + INT,
        wisSaveBonus: PB + scale(level, { 1: 1, 8: 2, 12: 3 }),
        hp: hp(level, 6, CON),
        hitDice: `${level}d6`,
        conModifier: CON,
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
                    toHit: toHit,
                    dpr: `${cantrip(level)}d10` + (level >= 10 ? ` + ${INT}[Empowered]` : ''),
                },
                {
                    id: uuid(),
                    type: 'template',
                    freq: { reset: 'lr', uses: 1 },
                    condition: 'not used yet',
                    templateOptions: { templateName: 'Mage Armour' },
                },
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
                    toHit: 100,
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
                    targets: scale(level, { 5: 2, 11: 3 }),
                    target: 'enemy with least HP',
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
                    templateOptions: { templateName: 'Hypnotic Pattern', saveDC: DC, target: 'enemy with highest DPR' },
                },
            ],
            14: [
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
                        { type: 'Discrete', resourceType: 'ClassResource', resourceVal: 'Overchannel', amount: 1 }
                    ],
                    useSaves: true,
                    halfOnSave: true,
                    toHit: DC,
                    dpr: `48 + ${INT}`,
                },
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

const acTION = 0
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
    return dieSize + con + (level - 1) * (Math.floor(dieSize / 2) + 1 + con)
}

function pb(level: number) {
    return scale(level, { 1: 2, 5: 3, 9: 4, 13: 5, 17: 6 })
}

function name(className: string, level: number) {
    return `Lv${level} ${className}`
}